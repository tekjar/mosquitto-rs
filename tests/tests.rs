extern crate mosquitto;

use mosquitto::{MqttClientOptions, Qos};
use std::thread;
use std::time::Duration;
#[macro_use]
extern crate log;
extern crate env_logger;

/// Check for handshake successes and pingreqs
#[test]
fn synchronous_connect() {
    // USAGE: RUST_LOG=mosquitto cargo test -- --nocapture
    env_logger::init().unwrap();
    let mut opts = MqttClientOptions::new();
    opts.set_keep_alive(5);

    let client = opts.connect("localhost:1883");
    thread::sleep(Duration::new(10, 0));
}
