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

#[test]
fn all_ok() {

	/* Set before connect */
	//let client = Client::new("test").keep_alive(30).clean_session(true).auth("root", "admin");
	let mut client = Client::new("test")
					.keep_alive(5)
					.clean_session(false)
					.will("goodbye", "my last words");
				//	.auth("admin", "admin");			

	
	let i = 100;

	client.onconnect_callback(|a:i32|println!("@@@ On connect callback {}@@@", a + i));
	match client.connect("localhost"){
		Ok(_) => println!("Connection successful --> {:?}", client),
		Err(n) => panic!("Connection error = {:?}", n)
	}

	client.onsubscribe_callback(|mid|println!("@@@ Subscribe request received for message mid = {:?}", mid));
	client.subscribe("hello/world", Qos::AtMostOnce);

	let mut count = 0; //TODO: Weird count print in closure callback
	client.onmesssage_callback(|s|{									
									count += 1;
									println!("@@@ Message = {:?}, Count = {:?}", s, count);							   
								   });
	

	client.onpublish_callback(|mid|println!("@@@ Publish request received for message mid = {:?}", mid));

	for i in 0..5{
		client.publish("hello/world", "Hello", Qos::AtMostOnce);
	}

	client.loop_forever();
}
