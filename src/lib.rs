//! This crate provides rustful wrappers for (unsafe) mosquitto mqtt library.
//! With these wrappers you can write safe, superfast, concurrent mqtt code.
//! Since mosquitto libraries are low level and avalilable on almost all the platforms,
//! this crate is super portable
//!

#[macro_use]
extern crate log;
extern crate libc;
extern crate mosquitto_sys as bindings;
extern crate rand;
extern crate chan;
mod error;

use std::ptr;
use std::mem;
use std::ffi::{CString, CStr, NulError};
use std::collections::HashMap;
use std::time::Duration;
use rand::Rng;
use std::net::{SocketAddr, ToSocketAddrs};
use error::{Error, Result};
use chan::{Sender, Receiver};
#[macro_use]
extern crate lazy_static;
use std::sync::{Arc, Mutex};
use std::path::Path;


lazy_static! {
    static ref INSTANCES: Mutex<usize> = Mutex::new(0);
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

pub struct MqttClientOptions {
    keep_alive: Option<Duration>,
    clean_session: bool,
    client_id: Option<String>,
    retry_time: u32,
    ca_cert: Option<Path>,
    client_cert: Option<Path>,
    clinet_key: Option<Path>,
}

impl MqttClientOptions {
    pub fn new() -> Self {
        MqttClientOptions {
            keep_alive: Some(Duration::new(30, 0)),
            clean_session: true,
            client_id: None,
            retry_time: 60,
            ca_cert: None,
            client_cert: None,
            clinet_key: None,
        }
    }

    pub fn set_keep_alive(&mut self, secs: u16) -> &mut Self {
        self.keep_alive = Some(Duration::new(secs as u64, 0));
        self
    }


    pub fn set_retry_time(&mut self, secs: u32) -> &mut Self {
        self.retry_time = secs;
        self
    }

    pub fn set_client_id(&mut self, client_id: String) -> &mut Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn set_clean_session(&mut self, clean_session: bool) -> &mut Self {
        self.clean_session = clean_session;
        self
    }

    pub fn set_ca_crt(&mut self, path: Path) -> &mut Self {
        self.ca_cert = Some(path);
        self
    }

    pub fn set_client_crt(&mut self, path: Path) -> &mut Self {
        self.client_cert = Some(path);
        self
    }

    pub fn set_client_key(&mut self, path: Path) -> &mut Self {
        self.client_key = Some(path);
        self
    }

    pub fn generate_client_id(&mut self) -> &mut Self {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<u32>();
        self.client_id = Some(format!("mqttc_{}", id));
        self
    }

    pub fn connect<A: ToSocketAddrs>(mut self, addr: A) -> Result<MqttClient> {
        if self.client_id == None {
            self.generate_client_id();
        }

        let addr = try!(addr.to_socket_addrs()).next().expect("Socket address is broken");

        let icallbacks: HashMap<String, Box<FnMut(i32)>> = HashMap::new();
        let scallbacks: HashMap<String, Box<Fn(&str)>> = HashMap::new();

        let c_id = CString::new(self.client_id.clone().unwrap()).unwrap();
        let mosquitto_client = unsafe {
            bindings::mosquitto_new(c_id.as_ptr(), self.clean_session as u8, ptr::null_mut())
        };

        // set message retry time
        unsafe {
            bindings::mosquitto_message_retry_set(mosquitto_client, self.retry_time);
        }


        // set tls
        if self.ca_cert.is_some() && self.client_cert.is_some() && self.client_key.is_some() {
            let ca_cert = self.ca_cert.unwrap();
            let client_cert = self.client_cert.unwrap();
            let client_key = self.client_key.unwrap();

            if ca_cert.exists() == false {
                return Err(Error::InvalidCertPath("no ca cert found"));
            }else if client_cert.exists() == false {
                return Err(Error::InvalidCertPath("no client cert found"));
            }else if client_key.exists() == false {
                return Err(Error::InvalidCertPath("no client key found"));
            }

            c_ca_cert = CString::new(client_cert).unwrap();
            c_client_cert = CString::new(client_cert).unwrap();
            c_client_key = CString::new(client_key).unwrap();
                
                let tls_ret = unsafe {
                    bindings::mosquitto_tls_insecure_set(mosquitto_client, 1 as u8);
                    bindings::mosquitto_tls_set(mosquitto_client,
                                                          c_ca_cert.as_ptr(),
                                                          ptr::null_mut(),
                                                          c_client_cert.as_ptr(),
                                                          c_client_key.as_ptr(),
                                                          None);
                }

                if tls_ret != 0 {
                    cleanup();
                    return Err(Error::TlsError(tls_ret));
                }
        }

        let mut client = MqttClient {
            opts: self,
            host: addr,
            connect_synchronizer: chan::sync(0),
            icallbacks: icallbacks, // integer callbacks
            scallbacks: scallbacks, // string callbacks
            mosquitto: Arc::new(Mutex::new(mosquitto_client)),
        };
        client.onconnect_register();

        // initialize mosquitto lib once during the creation of 1st
        // client for tls to work
        if mosquitto_client != ptr::null_mut() {
            let mut instances = INSTANCES.lock().unwrap();
            *instances += 1;
            debug!("mosq client instance {:?} created", *instances);
            if *instances == 1 {
                unsafe {
                    debug!("@@@ Initializing mosquitto library @@@");
                    bindings::mosquitto_lib_init();
                }
            }
        } else {
            return Err(Error::InvalidMosqClient);
        }

        try!(client.connect());
        Ok(client)
    }
}

// #[derive(Default)]
// #[derive(Debug)]
pub struct MqttClient {
    opts: MqttClientOptions,
    host: SocketAddr,
    connect_synchronizer: (Sender<i32>, Receiver<i32>),
    icallbacks: HashMap<String, Box<FnMut(i32)>>,
    scallbacks: HashMap<String, Box<Fn(&str)>>,
    mosquitto: Arc<Mutex<*mut bindings::Struct_mosquitto>>,
}

impl MqttClient {
    fn connection_handshake(&self) -> Result<()> {
        let (_, ref receiver) = self.connect_synchronizer;
        let ret = receiver.recv();

        match ret {
            Some(value) => {
                if value == 0 {
                    debug!("handshake success!");
                    return Ok(());
                } else {
                    debug!("handshake error. error = {}", value);
                    Err(Error::ConnectionError(value))
                }
            }
            _ => Err(Error::ConnectionError(100)),
        }
    }


    pub fn reinitialise(&self, clean: bool) {

        let id = CString::new(self.opts.client_id.clone().unwrap()).unwrap();
        let mosquitto = *self.mosquitto.lock().unwrap();
        unsafe {
            bindings::mosquitto_reinitialise(mosquitto, id.as_ptr(), clean as u8, ptr::null_mut());
        }

    }



    /// Connects the client to broker. Connects to port 1883 by default (TODO)
    /// Speciy in `HOST:PORT` format if you want to connect to a different port.
    ///
    /// ```ignore
    /// match client.connect("localhost") {
    ///     Ok(_) => println!("Connection successful --> {:?}", client.id),
    ///     Err(n) => panic!("Connection error = {:?}", n),
    /// }
    /// ```
    ///
    pub fn connect(&mut self) -> Result<()> {
        let host = CString::new(self.host.ip().to_string()).unwrap();

        let n_ret;
        // Connect to broker
        // TODO: Take optional port number in the string and split it
        unsafe {
            let mosquitto = *self.mosquitto.lock().unwrap();
            n_ret = bindings::mosquitto_connect(mosquitto,
                                                host.as_ptr(),
                                                self.host.port() as i32,
                                                self.opts.keep_alive.unwrap().as_secs() as i32);

            if n_ret == 0 {
                // TODO: What happens to this thread if there is a problem if error is reported in callback (n_ret == 0 and error in callback (is this possible?))
                // Start a thread to process network traffic. All the callbacks are handled by this thread
                // Seems like this needs to be called per client. Or else callbacks are not working.
                bindings::mosquitto_loop_start(mosquitto);
            } else {
                return Err(Error::ConnectionError(n_ret));
            }
        }

        self.connection_handshake()
    }

    pub fn reconnect(&self) -> Result<()> {
        // Connect to broker
        let mosquitto = *self.mosquitto.lock().unwrap();
        unsafe {
            let n_ret = bindings::mosquitto_reconnect(mosquitto);
            if n_ret != 0 {
                return Err(Error::ConnectionError(n_ret));
            }
        };

        self.connection_handshake()
    }

    /// Registered callback is called when the broker sends a CONNACK message in response
    /// to a connection. Will be called even incase of failure. All your sub/pub stuff
    /// should ideally be done in this callback when connection is successful
    /// Callback argument specifies the connection state
    /// ```ignore
    /// let i = 100;
    ///
    /// client.onconnect_callback(move |a: i32| {
    ///         println!("i = {:?}", i);
    ///         println!("@@@ On connect callback {}@@@", a)
    ///     });
    /// ```
    fn onconnect_register(&self) {
        let mosquitto = *self.mosquitto.lock().unwrap();

        // setting client object as userdata. Setting 'callback' as userdata is buggy because by the
        // time the actual callback is invoked, other callbacks like 'on_subscribe' callback is overwriting
        // the userdata and wrong closure is getting invoked for on_connect callback
        let cb = self as *const _ as *mut libc::c_void;
        unsafe {
            // Set our closure as user data
            bindings::mosquitto_user_data_set(mosquitto, cb);
            // Register callback
            bindings::mosquitto_connect_callback_set(mosquitto, Some(onconnect_wrapper));
        }

        // Registered callback. user data is our closure
        unsafe extern "C" fn onconnect_wrapper(mqtt: *mut bindings::Struct_mosquitto,
                                               closure: *mut libc::c_void,
                                               val: libc::c_int) {
            let client: &mut MqttClient = mem::transmute(closure);
            let (ref sender, _) = client.connect_synchronizer;
            sender.send(val);
        }
    }


    /// Subscibe to a topic with a Qos
    ///
    /// ```ignore
    /// client.subscribe("hello/world", Qos::AtMostOnce);
    /// ```
    pub fn subscribe(&self, topic: &str, qos: Qos) -> Result<()> {
        let topic = CString::new(topic);

        let qos = match qos {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2,
        };

        let mosquitto = *self.mosquitto.lock().unwrap();
        let n_ret = unsafe {
            bindings::mosquitto_subscribe(mosquitto, ptr::null_mut(), topic.unwrap().as_ptr(), qos)
        };

        if n_ret == 0 {
            Ok(())
        } else {
            Err(Error::SubscribeError(n_ret))
        }
    }



    /// Publish a message with a Qos
    ///
    /// ```ignore
    /// let message = format!("{}...{:?} - Message {}", count, client.id, i);
    /// client.publish("hello/world", &message, Qos::AtLeastOnce);
    /// ```
    pub fn publish(&self,
                   mid: Option<&i32>,
                   topic: &str,
                   message: &Vec<u8>,
                   qos: Qos)
                   -> Result<()> {

        // CString::new(topic).unwrap().as_ptr() is wrong.
        // topic String gets destroyed and pointer is invalidated
        // Whem message is created, it will allocate to destroyed space of 'topic'
        // topic is now pointing to it and publish is happening on the same message String.
        //
        // Try let topic = CString::new(topic).unwrap().as_ptr(); instead of let topic = CString::new(topic)
        //

        let mosquitto = *self.mosquitto.lock().unwrap();
        let msg_len = message.len();

        let topic = CString::new(topic).unwrap();
        // let message = CString::new(message);

        let qos = match qos {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2,
        };

        let n_ret: i32;

        let c_mid = match mid {
            Some(m) => m as *const i32 as *mut i32,
            None => ptr::null_mut(),
        };

        unsafe {
            n_ret = bindings::mosquitto_publish(mosquitto,
                                                c_mid,
                                                topic.as_ptr(),
                                                msg_len as i32,
                                                message.as_ptr() as *mut libc::c_void,
                                                qos,
                                                0);
        }

        if n_ret == 0 {
            Ok(())
        } else {
            Err(Error::PublishError(n_ret))
        }
    }



    /// Registered callback is called when a message initiated with `publish` has been
    /// sent to the broker successfully.
    ///
    /// ```ignore
    /// client.onpublish_callback(move |mid| {
    ///         println!("@@@ Publish request received for message mid = {:?}", mid)
    ///     });
    /// ```
    pub fn onpublish_callback<F>(&mut self, callback: F)
        where F: FnMut(i32),
              F: 'static
    {
        let mosquitto = *self.mosquitto.lock().unwrap();
        self.icallbacks.insert("on_publish".to_string(), Box::new(callback));
        let cb = self as *const _ as *mut libc::c_void;

        unsafe {
            bindings::mosquitto_user_data_set(mosquitto, cb);
            bindings::mosquitto_publish_callback_set(mosquitto, Some(onpublish_wrapper));
        }

        unsafe extern "C" fn onpublish_wrapper(mqtt: *mut bindings::Struct_mosquitto,
                                               closure: *mut libc::c_void,
                                               mid: libc::c_int) {
            let client: &mut MqttClient = mem::transmute(closure);
            match client.icallbacks.get_mut("on_publish") {
                Some(cb) => cb(mid as i32),
                _ => panic!("No callback found"),
            }
        }

    }


    /// Registered callback will be called when a message is received from the broker
    ///
    /// ```ignore
    /// client.onmesssage_callback(move |s| {
    ///         println!("@@@ Message = {:?}, Count = {:?}", s, count);
    ///     });
    /// ```
    pub fn onmesssage_callback<F>(&mut self, callback: F)
        where F: Fn(&str),
              F: 'static
    {
        let mosquitto = *self.mosquitto.lock().unwrap();
        self.scallbacks.insert("on_message".to_string(), Box::new(callback));
        let cb = self as *const _ as *mut libc::c_void;
        unsafe {
            bindings::mosquitto_user_data_set(mosquitto, cb); /* Set our closure as user data */
            bindings::mosquitto_message_callback_set(mosquitto, Some(onmessage_wrapper));
        }


        unsafe extern "C" fn onmessage_wrapper(mqtt: *mut bindings::Struct_mosquitto, closure: *mut libc::c_void, mqtt_message: *const bindings::Struct_mosquitto_message)
        {

            let mqtt_message = (*mqtt_message).payload as *const libc::c_char;
            let mqtt_message = CStr::from_ptr(mqtt_message).to_bytes();
            let mqtt_message = std::str::from_utf8(mqtt_message).unwrap();

            let client: &mut MqttClient = mem::transmute(closure);
            match client.scallbacks.get("on_message") {
                Some(cb) => cb(mqtt_message),
                _ => panic!("No callback found"),
            }
        }
    }
}


impl Drop for MqttClient {
    fn drop(&mut self) {
        let mosquitto = *self.mosquitto.lock().unwrap();
        unsafe {
            bindings::mosquitto_disconnect(mosquitto);
            bindings::mosquitto_loop_stop(mosquitto, true as u8);
            bindings::mosquitto_destroy(mosquitto);
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
