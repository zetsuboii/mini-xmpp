use std::sync::Arc;

use crate::{conn::Connection, handlers::HandleRequest, state::ServerState};
use color_eyre::eyre;
use parsers::{
    constants::{NAMESPACE_BIND, NAMESPACE_SASL, NAMESPACE_TLS},
    from_xml::{ReadXmlString, WriteXmlString},
    jid::Jid,
    stanza::{
        iq::{self, Iq, IqPayload},
        Stanza,
    },
    stream::{
        auth::{AuthRequest, AuthSuccess, PlaintextCredentials},
        features::{
            Bind, Features, Mechanism, Mechanisms, StartTls, StartTlsResponse, StartTlsResult,
        },
        initial::InitialHeader,
    },
};
use sqlx::{Pool, Sqlite};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug)]
pub struct Session {
    pub pool: Pool<Sqlite>,
    pub connection: Connection,
}

impl Session {
    pub fn new(pool: Pool<Sqlite>, connection: Connection) -> Self {
        Self { pool, connection }
    }

    pub fn get_resource(&self) -> Option<String> {
        self.connection
            .get_jid()
            .and_then(|jid| jid.resource_part().map(|s| s.to_string()))
    }

    /// Resets the session by receiving a new stream header
    async fn reset(&mut self) -> eyre::Result<()> {
        // Receive the header
        let request = self.connection.read().await?;
        let mut header = InitialHeader::read_xml_string(&request)?;

        // Generate a new id
        let new_id = Uuid::new_v4().to_string();
        header.id = Some(new_id);

        // Send the header
        self.connection.send(header.write_xml_string()?).await
    }

    async fn validate_credentials(
        &mut self,
        credentials: &PlaintextCredentials,
    ) -> eyre::Result<bool> {
        let mut db_conn = self.pool.acquire().await?;

        // Check if user exists
        let users = sqlx::query!(
            "SELECT password FROM users WHERE email = $1",
            credentials.username
        )
        .fetch_all(&mut *db_conn)
        .await?;

        // If user does not exist, create it
        // If user exists, check if password matches
        if users.len() == 0 {
            sqlx::query!(
                "INSERT INTO users(email, password) VALUES($1, $2)",
                credentials.username,
                credentials.password
            )
            .execute(&mut *db_conn)
            .await?;
            Ok(true)
        } else {
            let user = &users[0];
            Ok(user.password == credentials.password)
        }
    }

    /// Negotiates features with the client
    async fn negotiate_features(&mut self, features: Features) -> eyre::Result<()> {
        // Send features
        self.connection.send(features.write_xml_string()?).await?;

        // If TLS is required, negotiate it
        if let Some(tls) = features.start_tls {
            if tls.required {
                let request = self.connection.read().await?;
                StartTls::read_xml_string(&request)?;

                let proceed = StartTlsResponse {
                    xmlns: NAMESPACE_TLS.into(),
                    result: StartTlsResult::Proceed,
                };
                self.connection.send(proceed.write_xml_string()?).await?;
            }
        }

        Ok(())
    }

    pub async fn handshake(&mut self) -> eyre::Result<()> {
        // Receive initial header
        self.reset().await?;

        // Send features
        let features = Features {
            mechanisms: Some(Mechanisms {
                xmlns: NAMESPACE_SASL.into(),
                mechanisms: vec![Mechanism::Plain],
            }),
            start_tls: Some(StartTls {
                xmlns: NAMESPACE_TLS.into(),
                required: true,
            }),
            ..Default::default()
        };
        self.negotiate_features(features).await?;
        self.reset().await?;

        // Authenticate client
        let request = self.connection.read().await?;
        let auth = AuthRequest::read_xml_string(&request)?;
        let credentials = PlaintextCredentials::from_base64(auth.value)?;
        let valid = self.validate_credentials(&credentials).await?;
        if !valid {
            eyre::bail!("Invalid credentials");
        }
        let jid = Jid::try_from(credentials.username)?;
        let success = AuthSuccess {
            xmlns: NAMESPACE_SASL.into(),
        };
        self.connection.send(success.write_xml_string()?).await?;
        self.reset().await?;

        // Bind resource
        let bind_features = Features {
            bind: Some(Bind::new(NAMESPACE_BIND.into())),
            ..Default::default()
        };
        self.negotiate_features(bind_features).await?;

        // Get resource request
        let request = self.connection.read().await?;
        let iq_req = Iq::read_xml_string(&request)?;
        let bind = match &iq_req.payload {
            Some(IqPayload::Bind(bind)) => bind,
            _ => eyre::bail!("Expected bind payload"),
        };

        // Generate resource
        let resource = match &bind.resource {
            Some(resource) => resource.clone(),
            None => Uuid::new_v4().to_string(),
        };
        let jid = jid.with_resource(resource);

        // Send resource response
        let mut iq_res = iq_req;
        iq_res.from = None;
        iq_res.type_ = Some("result".into());
        iq_res.payload = Some(IqPayload::Bind(iq::Bind {
            xmlns: NAMESPACE_BIND.into(),
            jid: Some(jid.clone()),
            resource: None,
        }));
        self.connection.send(iq_res.write_xml_string()?).await?;
        self.connection.set_jid(jid);

        Ok(())
    }

    pub async fn listen_stanza(&mut self, state: Arc<RwLock<ServerState>>) -> eyre::Result<()> {
        let request = self.connection.read_timeout(10).await;

        match request {
            Ok(request) => {
                let stanza = match Stanza::read_xml_string(&request) {
                    Ok(stanza) => stanza,
                    Err(e) => {
                        eyre::bail!("error reading stanza: {}", e);
                    }
                };
                stanza.handle_request(self, state.clone()).await?;
            }
            Err(e) => match e.to_string().as_str() {
                "timeout" => {}
                _ => eyre::bail!("connection closed"),
            },
        }

        Ok(())
    }
}
