use mini_jabber::*;

// Using this as a playground
fn main() {
    let features = StreamFeatures {
        start_tls: None,
        mechanisms: Some(
            Mechanisms {
                xmlns: "hello".to_string(),
                mechanisms: vec![
                    Mechanism("PLAIN".into()),
                    Mechanism("SCRAM-SHA-1".into()),
                ]
            }
        ),
        bind: None
    };

    let result = features.to_string();
    println!("result: {result}");
}