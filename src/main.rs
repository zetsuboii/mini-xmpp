use mini_jabber::*;
use xmlserde::xml_serialize;

// Using this as a playground
fn main() {
    let features = StreamFeatures {
        start_tls: None,
        mechanisms: Some(
            Mechanisms {
                xmlns: "hello".to_string(),
                mechanisms: vec![
                    Mechanism { value: "PLAIN".to_string() },
                    Mechanism { value: "SCRAM-SHA-1".to_string() }
                ]
            }
        )
    };

    let result = xml_serialize(features);
    println!("result: {result}");
}