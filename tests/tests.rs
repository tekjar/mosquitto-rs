extern crate mosquitto;
use mosquitto::{Client, Qos};

/* The linked code creates a client that connects to a broker at
 * localhost:1883, subscribes to the topics "tick", "control".
 * When received a message on 'tick', it'll be forwarded to tock
 */

/* 1. Start the broker --> mosquitto -c /etc/mosquitto/mosquitto.conf -d
   2. cargo run
   3. mosquitto_sub -t "tock"
   4. mosquitto_pub -t "tick" -m "Hello World"
   5. mosquitto_pub -t "control" -m "halt" --> stop
*/

// #[test]
fn all_ok() {

	/* Set before connect */
	//let client = Client::new("test").keep_alive(30).clean_session(true).auth("root", "admin");

	// let mut clients: Vec<Client> = vec![];

	// for i in 0..1000{

	// 	let mut client = Client::new("test".to_string())
	// 				.keep_alive(5)
	// 				.clean_session(false)
	// 				.will("goodbye", "my last words");
	// 	clients.push(client);
	// }

	let mut client = Client::new("test")
					.keep_alive(5)
					.clean_session(false)
					.will("goodbye", "my last words");
				//	.auth("admin", "admin");



	
	let i = 100;

	client.onconnect_callback(move |a: i32|{
									println!("i = {:?}", i);
									println!("@@@ On connect callback {}@@@", a)
						  		});
	
	match client.connect("localhost"){
		Ok(_) => println!("Connection successful --> {:?}", "client"),
		Err(n) => panic!("Connection error = {:?}", n)
	}

	client.onsubscribe_callback(move|mid|println!("@@@ Subscribe request received for message mid = {:?}", mid));
	client.subscribe("hello/world", Qos::AtMostOnce);

	let mut count = 0; //TODO: Weird count print in closure callback
	client.onmesssage_callback(move |s|{									
									println!("@@@ Message = {:?}, Count = {:?}", s, count);							   
								   });
	

	client.onpublish_callback(move |mid|println!("@@@ Publish request received for message mid = {:?}", mid));

	for i in 0..5{
	 	client.publish("hello/world", "Hello", Qos::AtMostOnce);
	}

	client.loop_forever();
}

#[test]
/* Tests idle connections */
fn connect_stress(){
	Client::init();
	let id_prefix: String = "ath".to_string();
	let mut clients: Vec<Client> = Vec::new();

	for i in 0..10{
		let id = format!("{}-{}", id_prefix, i);
		//println!("{:?}", id);
		let mut client = Client::new(id)
					.keep_alive(30)
					.clean_session(true)
					.will("goodbye", "my last words");
		clients.push(client);
		match clients[i].connect("ec2-52-77-220-182.ap-southeast-1.compute.amazonaws.com"){
			Ok(_) => println!("Connection successful --> {:?}", "client"),
			Err(n) => panic!("Connection error = {:?}", n)
		}
	}
	clients[0].loop_forever();
	Client::cleanup();
	
/************************ Stress observations **************************
  Broker should receive a PINGREQ message every 'keep alive' time if no other messages are exchanged in this time
  Network loop should kick in at right time and send PINGREQ or else broker will disconnect the client. If there are
  a lot of connections, choose 'keep alive' in such a way that all the network threads (per client) are hit in 
  'keep alive' seconds

  - [ ] 'keep alive' of 5 seems to be ok for 100 client connections
  - [ ] 'keep alive' of 30 seems to be ok for 300 client connections
*/
}
