use std::io::{BufRead, Write};

use color_eyre::eyre;
use parsers::{
    constants::{NAMESPACE_BIND, NAMESPACE_SASL, NAMESPACE_TLS},
    empty::IsEmpty,
    from_xml::{ReadXmlString, WriteXmlString},
    jid::Jid,
    stanza::{
        iq::{Bind, Iq, Payload},
        message, Stanza,
    },
    stream::{
        auth::{AuthRequest, AuthSuccess, PlaintextCredentials},
        features::{Features, Mechanism, StartTls, StartTlsResponse, StartTlsResult},
        initial::InitialHeader,
    },
};
use quick_xml::escape::unescape;
use uuid::Uuid;

use crate::conn::Connection;

#[derive(Debug)]
pub struct Session {
    id: Option<String>,
    jid: Jid,
    credentials: PlaintextCredentials,
    connection: Connection,
}

impl Session {
    pub fn new(jid: Jid, credentials: PlaintextCredentials, connection: Connection) -> Self {
        Self {
            id: None,
            jid,
            credentials,
            connection,
        }
    }

    /// Resets the session by sending a new stream header
    /// After connection is established again, id of the session is updated
    async fn reset(&mut self) -> eyre::Result<()> {
        // Build initial header
        let mut initial_header = InitialHeader::new();
        initial_header.id = self.id.clone();
        initial_header.from = Some(self.jid.to_string());
        initial_header.to = Some("localhost".into());
        initial_header.version = Some("1.0".to_string());
        initial_header.xmlns = Some("jabber:client".to_string());
        initial_header.xmlns_stream = Some("http://etherx.jabber.org/streams".to_string());
        initial_header.xml_lang = Some("en".to_string());

        // Send to the stream
        self.connection
            .send(initial_header.write_xml_string()?)
            .await
            .unwrap();

        // Get response
        let response = self.connection.recv().await?;
        let header = InitialHeader::read_xml_string(&response)?;

        self.id = header.id;

        Ok(())
    }

    /// Negotiates features with the server
    /// For now, we only support PLAIN mechanism
    /// And we skip TLS negotiation even when it is required
    async fn negotiate_features(&mut self) -> eyre::Result<()> {
        // Get features from server
        let response = self.connection.recv().await?;
        let features = Features::read_xml_string(&response)?;

        // If no features, no need to negotiate
        if features.is_empty() {
            return Ok(());
        }

        // Evaluate features
        if let Some(mechanisms) = &features.mechanisms {
            if !mechanisms.mechanisms.contains(&Mechanism::Plain) {
                eyre::bail!("PLAIN mechanism not supported")
            }
        }

        if let Some(tls) = &features.start_tls {
            // If TLS is required, we need to negotiate it
            if tls.required {
                let mut tls_feature = StartTls::new(NAMESPACE_TLS.to_string());
                tls_feature.required = true;

                // Send TLS feature
                self.connection
                    .send(tls_feature.write_xml_string()?)
                    .await?;

                // Get response
                let response = self.connection.recv().await?;
                let tls_response = StartTlsResponse::read_xml_string(response.as_str());

                // TODO: Server doesn't add xmlns attribute to the response
                match tls_response {
                    Ok(response) => {
                        if let StartTlsResult::Failure = response.result {
                            eyre::bail!("TLS negotiation failed")
                        }
                    }
                    Err(e) => {
                        eprintln!("{}, ignoring", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Binds a resource to the session
    async fn bind_resource(&mut self) -> eyre::Result<()> {
        // Get stream features from server and check if bind option is available
        let response = self.connection.recv().await?;
        let features = Features::read_xml_string(&response)?;
        features
            .bind
            .ok_or_else(|| eyre::eyre!("bind feature not available"))?;

        // Send bind request IQ
        let request_id = Uuid::new_v4().to_string();
        let mut iq = Iq::new(request_id);
        iq.type_ = Some("set".to_string());

        // We don't know if the server supports resource binding
        // So we separate the resource part from the JID
        let mut bind = Bind::new(NAMESPACE_BIND.into());
        bind.resource = self.jid.resource_part.take();
        bind.jid = Some(self.jid.clone());
        iq.payload = Some(Payload::Bind(bind));

        self.connection.send(iq.write_xml_string()?).await?;

        // Get response and save the resource
        let response = self.connection.recv().await?;
        let iq = Iq::read_xml_string(response.as_str())?;

        if let Some(Payload::Bind(bind)) = iq.payload {
            self.jid.resource_part = bind.jid.and_then(|jid| jid.resource_part);
        } else {
            eyre::bail!("invalid bind response")
        }

        Ok(())
    }

    pub async fn handshake(&mut self) -> eyre::Result<()> {
        // Start by sending initial header
        self.reset().await?;

        // Negotiate features
        self.negotiate_features().await?;
        self.reset().await?;

        // Authenticate
        let auth = AuthRequest::new(
            NAMESPACE_SASL.to_string(),
            Mechanism::Plain,
            self.credentials.to_base64(),
        );
        self.connection.send(auth.write_xml_string()?).await?;

        // Get response and assert that it is success
        let response = self.connection.recv().await?;
        AuthSuccess::read_xml_string(response.as_str())?;
        self.reset().await?;

        // Bind resource
        self.bind_resource().await?;

        Ok(())
    }

    /// Sends a stanza to server
    pub async fn send_stanza(&mut self, stanza: impl WriteXmlString) -> eyre::Result<()> {
        self.connection.send(stanza.write_xml_string()?).await?;
        Ok(())
    }

    /// Waits for a stanza from server
    pub async fn recv_stanza(&mut self) -> eyre::Result<Stanza> {
        let response = self.connection.recv().await?;
        Stanza::read_xml_string(response.as_str())
    }

    /// Start sending and receving messages
    pub async fn start_messaging(self) -> eyre::Result<()> {
        let (mut reader, mut writer) = self.connection.split();

        // Start listening for messages
        let receiver = tokio::spawn(async move {
            loop {
                let response = reader.recv().await.unwrap();
                let stanza = Stanza::read_xml_string(response.as_str()).unwrap();
                match stanza {
                    Stanza::Message(message) => {
                        let from = message.from.unwrap_or("unknown".into());
                        let body = message.body.unwrap_or("".into());

                        println!("\rfrom: {}", from);
                        println!("< {}", unescape(body.as_ref()).unwrap());
                        print!("{}\nto: ", "=".repeat(32));
                        std::io::stdout().lock().flush().expect("failed to flush");
                    }
                    Stanza::Presence(presence) => {
                        let from = presence.from.unwrap_or("unknown".to_string());

                        println!("\r< {} now online", from);
                        print!("{}\nto: ", "=".repeat(32));
                        std::io::stdout().lock().flush().expect("failed to flush");
                    }
                    _ => continue,
                }
            }
        });

        // Start getting user input and sending messages
        let sender = tokio::spawn(async move {
            loop {
                // Make a new line
                print!("to: ");
                std::io::stdout().lock().flush().expect("failed to flush");
                let to = get_user_input();

                // Make a new line
                print!("> ");
                std::io::stdout().lock().flush().expect("failed to flush");
                let input = get_user_input();

                // Send user input
                let message = Stanza::Message(message::Message {
                    id: Uuid::new_v4().to_string().into(),
                    from: self.jid.to_string().into(),
                    to: to.into(),
                    body: input.into(),
                    xml_lang: "en".to_string().into(),
                });
                writer
                    .send(message.write_xml_string().unwrap())
                    .await
                    .unwrap();
            }
        });

        receiver.await?;
        sender.await?;
        Ok(())
    }
}

fn get_user_input() -> String {
    let mut input = String::new();

    // Read user input
    std::io::stdin()
        .lock()
        .read_line(&mut input)
        .expect("failed to read to string");

    while input.ends_with("\n") {
        input.truncate(input.len() - 1);
    }

    input
}
