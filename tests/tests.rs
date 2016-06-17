extern crate mosquitto;

use mosquitto::{MqttClientOptions, Qos};
use std::thread;
use std::time::Duration;
#[macro_use]
extern crate log;
extern crate env_logger;

/// Check for handshake successes and pingreqs
// #[test]
fn synchronous_connect() {
    // USAGE: RUST_LOG=mosquitto cargo test -- --nocapture
    env_logger::init().unwrap();
    let mut opts = MqttClientOptions::new();
    opts.set_keep_alive(5);

    let mut client = opts.connect("localhost:1883").unwrap();
    client.subscribe("hello/world", Qos::AtLeastOnce);
    client.onmesssage_callback(move |s| {
        println!("@@@ Message = {:?}", s);
    });

    // callbacks are working even if there is a sleep here
    // so callbacks are being invoked on a different thread
    thread::sleep(Duration::new(30, 0));
}

#[test]
fn publish_blocks() {
    // USAGE: RUST_LOG=mosquitto cargo test -- --nocapture
    env_logger::init().unwrap();
    let mid = 0;
    let mut opts = MqttClientOptions::new();
    opts.set_keep_alive(5);

    let mut client = opts.connect("localhost:1883").unwrap();

    for i in 0..100 {
        client.publish(Some(&mid),
                       "hello/world",
                       &"hello rust".to_string().into_bytes(),
                       Qos::AtLeastOnce);
        thread::sleep(Duration::new(1, 0));
    }

    // callbacks are working even if there is a sleep here
    // so callbacks are being invoked on a different thread
    thread::sleep(Duration::new(30, 0));
}
