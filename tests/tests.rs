extern crate mosquitto;

use mosquitto::{Client, Qos};
use std::thread;
use std::time::Duration;

///#TESTCASES TO CHECK 
///
///- [ ] Clent ram persistence. When broker goes down, client should keep track of all its publishes (with QoS 1,2)
///- [ ] Client disk persistance. Broker went down/ Scooter internet down. Client crashed. Broker up. Client up. 
///      Now client should resend all the publishes after broker crash - before client crash.
///- [ ] Broker ram persistence. Broker should save messages that are supposed to be sent to disconnected persistent clients
///      (one's connected with clean_session = false).
///- [ ] Broker disk persistence. If broker went down before publishing all the messages (let's say a persisent client 
///      which is supposed to receive the publish is down), it should retry sending that message when it is back up again. 
///      I.e all the broker state should be written to permanent storage
///      Disconnect client, publish message, disconnect broker, connect broker, connect client.
///      Check is there is a way to periodically update the disk database incase of unexpected broker crashes.
///- [ ] Broker should remember client subscriptions for persistent clients (clean_session = false)even after disconnections 
///      and should directly handle publishes to them after reconnections. 
///      Disconnect and connect back broker and see if subscriptions persist. 
///      Note: Set 'persist = true' in mosquitto.conf
///- [ ] Disconnection handling. If broker goes down, client should reconnect automatically when broker comes up
///- [ ] Reconnection handling when both broker and client are crashed.
// TODO: Check why cargo test some times is not waiting at loop_forever()
// #[test]
// fn all_ok() {
//     // Set before connect
//     let client = Client::new("test", true)
//                      .unwrap()
//                      .keep_alive(30);
//
//     let mut clients: Vec<Client> = vec![];
//
//     for i in 0..1000 {
//
//         let mut client = Client::new(&"test".to_string(), true)
//                              .unwrap()
//                              .keep_alive(5)
//                              .will("goodbye", "my last words");
//         clients.push(client);
//     }
//
//     let mut client = Client::new("test", true)
//                          .unwrap()
//                          .keep_alive(5)
//                          .will("goodbye", "my last words");
//
//
//
//
//     let i = 100;
//
//     client.onconnect_callback(move |a: i32| {
//         println!("i = {:?}", i);
//         println!("@@@ On connect callback {}@@@", a)
//     });
//
//     match client.connect("ec2-52-77-220-182.ap-southeast-1.compute.amazonaws.com") {
//         Ok(_) => println!("Connection successful --> {:?}", "client"),
//         Err(n) => panic!("Connection error = {:?}", n),
//     }
//
//     client.onsubscribe_callback(move |mid| {
//         println!("@@@ Subscribe request received for message mid = {:?}", mid)
//     });
//     client.subscribe("hello/world", Qos::AtMostOnce);
//
//     let mut count = 0; //TODO: Weird count print in closure callback
//     client.onmesssage_callback(move |s| {
//         println!("@@@ Message = {:?}, Count = {:?}", s, count);
//     });
//
//
//     client.onpublish_callback(move |mid| {
//         println!("@@@ Publish request received for message mid = {:?}", mid)
//     });
//
//     for i in 0..5 {
//         client.publish("hello/world".to_string(),
//                        "Hello".to_string(),
//                        Qos::AtMostOnce);
//     }
//
//     client.loop_forever();
// }
///###ANALYSIS
/// -[X] Auto reconnect working
/// 
/// -[X] Client RAM persistance working
/// Testcase: All the scooter clients will start publishing and AWS client receives it. 
///           At some point in between, broker goes down and hence AWS client will stop receiving
///           AWS client should receive all the messages when broker comes up again. Total count should be = 100
///            
#[test]
fn client_persistance() {
    let client_test = Client::new("test", true).unwrap().keep_alive(30);

    let mut clients: Vec<Client> = vec![];

    for i in 0..10 {
        let id = format!("client-{}", i);
        let mut client = Client::new(&id, true)
                             .unwrap()
                             .keep_alive(5)
                             .will("goodbye", "my last words");
        clients.push(client);
    }

    // for client in clients.iter_mut() {
    //     client.onconnect_callback(move |a: i32| {
    //         println!("@@@ {} - On connect callback {} @@@", &client.id, a);
    //     });
    // }

    // for client in clients.iter_mut() {
    //     match client.connect("localhost") {
    //         Ok(_) => println!("Connection successful --> {:?}", client.id),
    //         Err(n) => panic!("Connection error = {:?}", n),
    //     }
    // }

    for client in clients.iter_mut() {
        match client.secure_connect("localhost",
                                    "/home/raviteja/Desktop/certs/ca.crt",
                                    Some(("/home/raviteja/Desktop/certs/scooter.crt",
                                          "/home/raviteja/Desktop/certs/scooter.key"))) {
            Ok(_) => println!("Connection successful --> {:?}", client.id),
            Err(n) => panic!("Connection error = {:?}", n),
        }
    }

    let mut count = 0;
    for client in clients.iter_mut() {
        for i in 0..10 {
            thread::sleep(Duration::from_millis(100));
            let message = format!("{}...{:?} - Message {}", count, client.id, i);
            client.publish("ather/log-ship", &message, Qos::AtLeastOnce);
            count += 1;
        }
    }


    // client_test.loop_forever();
}

// #[test]
fn idle_connect() {
    let id_prefix: String = "ath".to_string();
    let mut clients: Vec<Client> = Vec::new();

    for i in 0..10 {
        let id = format!("{}-{}", id_prefix, i);
        // println!("{:?}", id);
        let mut client = Client::new(&id, true)
                             .unwrap()
                             .keep_alive(30)
                             .will("goodbye", "my last words");
        clients.push(client);
        match clients[i].connect("ec2-52-77-220-182.ap-southeast-1.compute.amazonaws.com") {
            Ok(_) => println!("Connection successful --> {:?}", "client"),
            Err(n) => panic!("Connection error = {:?}", n),
        }
    }
    clients[0].loop_forever();
    mosquitto::cleanup();
}


// /************************ Stress observations **************************
//   Broker should receive a PINGREQ message every 'keep alive' time if no other messages are exchanged in this time
//   Network loop should kick in at right time and send PINGREQ or else broker will disconnect the client. If there are
//   a lot of connections, choose 'keep alive' in such a way that all the network threads (per client) are hit in
//   'keep alive' seconds

//   - [ ] 'keep alive' of 5 seems to be ok for 100 client connections
//   - [ ] 'keep alive' of 30 seems to be ok for 300 client connections
// */
// }
