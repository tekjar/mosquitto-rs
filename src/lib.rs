extern crate libc;

use std::ptr;
use std::ffi::{CString, CStr};

mod bindings;
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


//#[derive(Default)]
#[derive(Debug)]
pub struct Client<'b, 'c>{
	pub host: String,
	pub id: String,
	pub user_name: Option<&'b str>,
	pub password: Option<&'c str>,
	pub keep_alive: i32,
	pub clean: bool,
	mosquitto: * mut bindings::Struct_mosquitto
}

impl<'b, 'c> Client<'b, 'c>{

    pub fn new(id: &str) -> Client{
        let name = CString::new(id).unwrap().as_ptr();
        let mosquitto: * mut bindings::Struct_mosquitto;
        unsafe{
            mosquitto = bindings::mosquitto_new(name, 1, ptr::null_mut());
        }
        //TODO: Implement default for mosquitto and remove this ugly default host
        Client{
        	host: "test.mosquitto.org".to_string(),
        	id: id.to_string(),
        	user_name: None,
        	password: None,
        	keep_alive: 10,
        	clean: true,
        	mosquitto:mosquitto,
        }
    }

    pub fn keep_alive(mut self, keepalive: i32) -> Self{
    	self.keep_alive = keepalive;
    	self
    } 

    pub fn connect(&self, host: &'b str) -> Result<&Self, i32>{
        let host = CString::new(host).unwrap().as_ptr();
        let nRet;

        unsafe{
            nRet = bindings::mosquitto_connect(self.mosquitto, host, 1883, self.keep_alive);
        }

        if nRet == 0{
        	Ok(self)
        }
        else{
        	Err(nRet)
        }
    }

    pub fn register_onconnect_callback<F>(&self, mut callback: F) where F: Fn(i32){

    	/* Convert the rust closure into void* to be used as user_data. This will
    	   be passed to call back automatically by the library */
    	let cb = &callback as *const _ as *mut libc::c_void;
       	 
        unsafe{
        	bindings::mosquitto_user_data_set(self.mosquitto, cb); /* Set our closure as user data */
            bindings::mosquitto_connect_callback_set(self.mosquitto, Some(onconnect_wrapper::<F>)); /* Register callback */
        }
        
        /* Registered callback. user data is our closure */
        unsafe extern "C" fn onconnect_wrapper<F>(mqtt: *mut bindings::Struct_mosquitto, closure: *mut libc::c_void, val: libc::c_int)
        where F:Fn(i32){
        	let closure = closure as *mut F;
        	(*closure)(val as i32);
      		
        }
    }

    pub fn loop_forever(&self){
        unsafe{
            bindings::mosquitto_loop_forever(self.mosquitto, 1000, 1000);
        }
    }
}

#[test]
fn it_works() {
	let client = Client::new("test").keep_alive(30);
	
	match client.connect("test.mosquitto.org"){
		Ok(_) => println!("Connection successful"),
		Err(n) => panic!("Connection error = {:?}", n)
	}

	let i = 100;

	client.register_onconnect_callback(|a:i32|println!("@@@ On connect callback {}@@@", a + i));
	client.loop_forever();
}
