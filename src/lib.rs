//! This crate provides rustful wrappers for (unsafe) mosquitto mqtt library.
//! With these wrappers you can write safe, superfast, concurrent mqtt code.
//! Since mosquitto libraries are low level and avalilable on almost all the platforms,
//! this crate is super portable
//!


extern crate libc;
extern crate mosquitto_sys as bindings;

use std::ptr;
use std::mem;
use std::ffi::{CString, CStr, NulError};
use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref INSTANCES: Mutex<usize> = Mutex::new(0);
}


// #[derive(Default)]
// #[derive(Debug)]
pub struct Client<'b, 'c, 'd> {
    pub id: String,
    pub user_name: Option<&'b str>,
    pub password: Option<&'c str>,
    pub host: Option<&'d str>,
    pub keep_alive: i32,
    pub clean_session: bool,
    pub icallbacks: HashMap<String, Box<Fn(i32)>>,
    pub scallbacks: HashMap<String, Box<Fn(&str)>>,
    pub mosquitto: *mut bindings::Struct_mosquitto,
}

pub enum Qos {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}


fn cleanup() {
    unsafe {
        bindings::mosquitto_lib_cleanup();
    }
}

impl<'b, 'c, 'd> Client<'b, 'c, 'd> {
    ///Creates a new mosquitto mqtt client
    ///
    ///**id**: ID of the new client  
    ///**clean**: Clean session or not. If not, broker will remember this client(useful during connection drops)
    ///
    ///```ignore
    ///let mut client = Client::new(&id, true).unwrap()
    ///``
    pub fn new(id: &str, clean: bool) -> Result<Client<'b, 'c, 'd>, i32> {
        let icallbacks: HashMap<String, Box<Fn(i32)>> = HashMap::new();
        let scallbacks: HashMap<String, Box<Fn(&str)>> = HashMap::new();

        let mut client = Client {
            id: id.to_string(),
            user_name: None,
            password: None,
            host: None,
            keep_alive: 10,
            clean_session: clean,
            icallbacks: icallbacks, // integer callbacks
            scallbacks: scallbacks, // string callbacks
            mosquitto: ptr::null_mut(),
        };

        let id = CString::new(id);

        // TODO: Replace all 'unwrap().as_ptr() as *const _' with 'unwrap().as_ptr()' in rust 1.6
        unsafe {
            client.mosquitto = bindings::mosquitto_new(id.unwrap().as_ptr() as *const libc::c_char,
                                                       clean as u8,
                                                       ptr::null_mut());
        }

        if client.mosquitto != ptr::null_mut() {

            let mut instances = INSTANCES.lock().unwrap();
            *instances += 1;
            println!("mosq client instance {:?} created", *instances);
            if *instances == 1 {
                unsafe {
                    println!("@@@ Initializing mosquitto library @@@");
                    bindings::mosquitto_lib_init();
                }
            }


            Ok(client)
        } else {
            Err(-1)
        }
    }

    ///The number of seconds after which the broker should 
    ///send a PING message to the client if no other messages 
    ///have been exchanged in that time. This is necessary for
    ///keeping the connection alive
    ///
    ///```ignore
    ///let mut client = Client::new(&id, true)
    ///                         .unwrap()
    ///                         .keep_alive(5)
    ///```
    ///
    pub fn keep_alive(mut self, keepalive: i32) -> Self {
        self.keep_alive = keepalive;
        self
    }



    ///Will topic and message on behalf of the client.
    ///Broker will take the responsibility of publishing this
    ///after the client dies
    ///
    ///```ignore
    ///let mut client = Client::new(&id, true)
    ///                         .unwrap()
    ///                         .keep_alive(5)
    ///                         .will("goodbye", "my last words");
    ///```
    ///
    pub fn will(self, topic: &str, message: &str) -> Self {

        let msg_len = message.len();
        let topic = CString::new(topic);
        let message = CString::new(message);

        unsafe {
            // Publish will with Qos 2
            bindings::mosquitto_will_set(self.mosquitto,
                                         topic.unwrap().as_ptr() as *const libc::c_char,
                                         msg_len as i32,
                                         message.unwrap().as_ptr() as *mut libc::c_void,
                                         2,
                                         0);
        }
        self

    }


    ///Connects the client to broker. Connects to port 1883 by default (TODO)
    ///Speciy in `HOST:PORT` format if you want to connect to a different port.
    ///
    ///```ignore
    /// match client.connect("localhost") {
    ///     Ok(_) => println!("Connection successful --> {:?}", client.id),
    ///     Err(n) => panic!("Connection error = {:?}", n),
    /// }
    ///```
    ///
    pub fn connect(&mut self, host: &'d str, port: i32) -> Result<&Self, i32> {

        self.host = Some(host);

        let host = CString::new(host);

        let n_ret;
        // Connect to broker
        // TODO: Take optional port number in the string and split it
        unsafe {
            n_ret = bindings::mosquitto_connect(self.mosquitto,
                                                host.unwrap().as_ptr() as *const libc::c_char,
                                                port,
                                                self.keep_alive);
            if n_ret == 0 {
                // TODO: What happens to this thread if there is a problem if error is reported in callback (n_ret == 0 and error in callback (is this possible?))
                // Start a thread to process network traffic. All the callbacks are handled by this thread
                // Seems like this needs to be called per client. Or else callbacks are not working.
                bindings::mosquitto_loop_start(self.mosquitto);
                Ok(self)
            } else {
                Err(n_ret)
            }
        }
    }

    pub fn reconnect(&self) -> Result<&Self, i32> {

        let n_ret;
        // Connect to broker
        unsafe {
            n_ret = bindings::mosquitto_reconnect(self.mosquitto);
            if n_ret == 0 {
                Ok(self)
            } else {
                Err(n_ret)
            }
        }
    }


    ///Connects the client to broker using certificate based TLS authentication. 
    ///Connects to port 8884 by default (TODO).
    ///Speciy in `HOST:PORT` format if you want to connect to a different port.
    ///
    ///```ignore
    /// match client.connect("localhost") {
    ///     Ok(_) => println!("Connection successful --> {:?}", client.id),
    ///     Err(n) => panic!("Connection error = {:?}", n),
    /// }
    ///```
    ///
    pub fn secure_connect(&mut self,
                          host: &'d str,
                          port: i32,
                          ca_cert: &str,
                          client_cert: Option<(&str, &str)>)
                          -> Result<&Self, i32> {
        // TODO: Remove all the unwraps and panics from the code
        let c_ca_cert = CString::new(ca_cert);
        let c_client_cert: Result<CString, NulError>;
        let c_client_key: Result<CString, NulError>;

        let tls_ret: i32;
        match client_cert {
            Some((cert, key)) => {
                c_client_cert = CString::new(cert);
                c_client_key = CString::new(key);
                unsafe {
                    bindings::mosquitto_tls_insecure_set(self.mosquitto, 1 as u8);
                    tls_ret = bindings::mosquitto_tls_set(self.mosquitto,
                                                          c_ca_cert.unwrap().as_ptr() as *const libc::c_char,
                                                          ptr::null_mut(),
                                                          c_client_cert.unwrap().as_ptr() as *const libc::c_char,
                                                          c_client_key.unwrap().as_ptr() as *const libc::c_char,
                                                          None);
                }

                if tls_ret != 0 {
                    cleanup();
                    Err(tls_ret)
                } else {
                    self.connect(host, port)
                }

            }
            None => {
                unsafe {
                    tls_ret = bindings::mosquitto_tls_set(self.mosquitto,
                                                          c_ca_cert.unwrap().as_ptr() as *const libc::c_char,
                                                          ptr::null_mut(),
                                                          ptr::null_mut(),
                                                          ptr::null_mut(),
                                                          None);
                }

                if tls_ret != 0 {
                    cleanup();
                    Err(tls_ret)
                } else {
                    self.connect(host, port)
                }
            }
        }
    }


    ///Registered callback is called when the broker sends a CONNACK message in response
    ///to a connection. Will be called even incase of failure. All your sub/pub stuff
    ///should ideally be done in this callback when connection is successful
    ///Callback argument specifies the connection state
    ///```ignore
    /// let i = 100;
    ///
    /// client.onconnect_callback(move |a: i32| {
    ///         println!("i = {:?}", i);
    ///         println!("@@@ On connect callback {}@@@", a)
    ///     });
    ///```
    pub fn onconnect_callback<F>(&mut self, callback: F)
        where F: Fn(i32),
              F: 'static
    {
        self.icallbacks.insert("on_connect".to_string(), Box::new(callback));
        // setting client object as userdata. Setting 'callback' as userdata is buggy because by the
        // time the actual callback is invoked, other callbacks like 'on_subscribe' callback is overwriting
        // the userdata and wrong closure is getting invoked for on_connect callback
        let cb = self as *const _ as *mut libc::c_void;
        unsafe {
            // Set our closure as user data
            bindings::mosquitto_user_data_set(self.mosquitto, cb);
            // Register callback
            bindings::mosquitto_connect_callback_set(self.mosquitto, Some(onconnect_wrapper));
        }

        // Registered callback. user data is our closure
        unsafe extern "C" fn onconnect_wrapper(mqtt: *mut bindings::Struct_mosquitto,
                                               closure: *mut libc::c_void,
                                               val: libc::c_int) {
            let client: &mut Client = mem::transmute(closure);
            match client.icallbacks.get("on_connect") {
                Some(cb) => cb(val as i32),
                _ => panic!("No callback found"),
            }
        }
    }


    ///Subscibe to a topic with a Qos
    ///
    ///```ignore
    /// client.subscribe("hello/world", Qos::AtMostOnce);
    ///```
    pub fn subscribe(&self, topic: &str, qos: Qos) {
        let topic = CString::new(topic);

        let qos = match qos {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2,
        };

        unsafe {
            bindings::mosquitto_subscribe(self.mosquitto,
                                          ptr::null_mut(),
                                          topic.unwrap().as_ptr() as *const libc::c_char,
                                          qos);
        }
    }

    ///Registered callback will be called when broker responds to a subscription
    ///
    ///```ignore
    /// client.onsubscribe_callback(move |mid| {
    ///            println!("@@@ Subscribe request received for message mid = {:?}", mid)
    ///        });
    ///```
    pub fn onsubscribe_callback<F>(&mut self, callback: F)
        where F: Fn(i32),
              F: 'static
    {
        self.icallbacks.insert("on_subscribe".to_string(), Box::new(callback));
        let cb = self as *const _ as *mut libc::c_void;

        unsafe {
            bindings::mosquitto_user_data_set(self.mosquitto, cb);
            bindings::mosquitto_subscribe_callback_set(self.mosquitto, Some(onsubscribe_wrapper));
        }

        unsafe extern "C" fn onsubscribe_wrapper(mqtt: *mut bindings::Struct_mosquitto,
                                                 closure: *mut libc::c_void,
                                                 mid: libc::c_int,
                                                 qos_count: libc::c_int,
                                                 qos_list: *const ::libc::c_int) {
            let client: &mut Client = mem::transmute(closure);
            match client.icallbacks.get("on_subscribe") {
                Some(cb) => cb(mid as i32),
                _ => panic!("No callback found"),
            }
        }
    }


    ///Publish a message with a Qos
    ///
    ///```ignore
    /// let message = format!("{}...{:?} - Message {}", count, client.id, i);
    /// client.publish("hello/world", &message, Qos::AtLeastOnce);
    ///```
    pub fn publish(&self, topic: &str, message: &str, qos: Qos) -> Result<(), i32> {

        // CString::new(topic).unwrap().as_ptr() is wrong.
        // topic String gets destroyed and pointer is invalidated
        // Whem message is created, it will allocate to destroyed space of 'topic'
        // topic is now pointing to it and publish is happening on the same message String.
        //
        // Try let topic = CString::new(topic).unwrap().as_ptr(); instead of let topic = CString::new(topic)
        //


        // If inputs are of type &str, Convert them to String
        // let message = message.into();
        // let topic = topic.into();

        let msg_len = message.len();

        let topic = CString::new(topic);
        let message = CString::new(message);

        let qos = match qos {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2,
        };

        let n_ret: i32;
        unsafe {
            n_ret = bindings::mosquitto_publish(self.mosquitto,
                                                ptr::null_mut(),
                                                topic.unwrap().as_ptr() as *const libc::c_char,
                                                msg_len as i32,
                                                message.unwrap().as_ptr() as *mut libc::c_void,
                                                qos,
                                                0);
        }

        if n_ret == 0 {
            Ok(())
        } else {
            Err(n_ret)
        }
    }



    ///Registered callback is called when a message initiated with `publish` has been 
    ///sent to the broker successfully.
    ///
    ///```ignore
    ///client.onpublish_callback(move |mid| {
    ///         println!("@@@ Publish request received for message mid = {:?}", mid)
    ///     });
    ///```
    pub fn onpublish_callback<F>(&mut self, callback: F)
        where F: Fn(i32),
              F: 'static
    {
        self.icallbacks.insert("on_publish".to_string(), Box::new(callback));
        let cb = self as *const _ as *mut libc::c_void;

        unsafe {
            bindings::mosquitto_user_data_set(self.mosquitto, cb);
            bindings::mosquitto_publish_callback_set(self.mosquitto, Some(onpublish_wrapper));
        }

        unsafe extern "C" fn onpublish_wrapper(mqtt: *mut bindings::Struct_mosquitto,
                                               closure: *mut libc::c_void,
                                               mid: libc::c_int) {
            let client: &mut Client = mem::transmute(closure);
            match client.icallbacks.get("on_publish") {
                Some(cb) => cb(mid as i32),
                _ => panic!("No callback found"),
            }
        }
    }


    ///Registered callback will be called when a message is received from the broker
    ///
    ///```ignore
    ///client.onmesssage_callback(move |s| {
    ///         println!("@@@ Message = {:?}, Count = {:?}", s, count);
    ///     });
    ///```
    pub fn onmesssage_callback<F>(&mut self, callback: F)
        where F: Fn(&str),
              F: 'static
    {
        self.scallbacks.insert("on_message".to_string(), Box::new(callback));
        let cb = self as *const _ as *mut libc::c_void;
        unsafe {
            bindings::mosquitto_user_data_set(self.mosquitto, cb); /* Set our closure as user data */
            bindings::mosquitto_message_callback_set(self.mosquitto, Some(onmessage_wrapper));
        }


        unsafe extern "C" fn onmessage_wrapper(mqtt: *mut bindings::Struct_mosquitto, closure: *mut libc::c_void, mqtt_message: *const bindings::Struct_mosquitto_message)
        {

            let mqtt_message = (*mqtt_message).payload as *const i8;
            let mqtt_message = CStr::from_ptr(mqtt_message).to_bytes();
            let mqtt_message = std::str::from_utf8(mqtt_message).unwrap();

            let client: &mut Client = mem::transmute(closure);
            match client.scallbacks.get("on_message") {
                Some(cb) => cb(mqtt_message),
                _ => panic!("No callback found"),
            }
        }
    }

    pub fn loop_forever(&self) {
        unsafe {
            bindings::mosquitto_loop_forever(self.mosquitto, 2000, 1);
        }
    }
}


impl<'b, 'c, 'd> Drop for Client<'b, 'c, 'd> {
    fn drop(&mut self) {

        unsafe {
            bindings::mosquitto_disconnect(self.mosquitto);
            bindings::mosquitto_loop_stop(self.mosquitto, true as u8);
            bindings::mosquitto_destroy(self.mosquitto);
        }

        let mut instances = INSTANCES.lock().unwrap();
        println!("mosq client instance {:?} desroyed", *instances);
        *instances -= 1;


        if *instances == 0 {
            println!("@@@ All clients dead. Cleaning mosquitto library @@@");
            cleanup();
        }
    }
}

// NOTE:
// mosquitto_lib_init() calls everything that is needed by the internals of the library.
// If you're on Windows nothing will work without it for example.
// On linux, for TLS, mosquitto_lib_init() is necessary.
// Multiple calls - it depends whether anything else is using the same libraries (e.g. openssl).
// If you call lib_cleanup() then everything using openssl will stop working.
// So don't call it at destruction of each client
//
