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
}

impl MqttClientOptions {
    pub fn new() -> Self {
        MqttClientOptions {
            keep_alive: Some(Duration::new(30, 0)),
            clean_session: true,
            client_id: None,
        }
    }

    pub fn set_keep_alive(&mut self, secs: u16) -> &mut Self {
        self.keep_alive = Some(Duration::new(secs as u64, 0));
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

        let mut client = MqttClient {
            opts: self,
            host: addr,
            connect_channel: chan::sync(0),
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
    connect_channel: (Sender<i32>, Receiver<i32>),
    icallbacks: HashMap<String, Box<FnMut(i32)>>,
    scallbacks: HashMap<String, Box<Fn(&str)>>,
    mosquitto: Arc<Mutex<*mut bindings::Struct_mosquitto>>,
}

impl MqttClient {
    fn handshake(&self) -> Result<()> {
        let (_, ref receiver) = self.connect_channel;
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

        self.handshake()
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

        self.handshake()
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
            let (ref sender, _) = client.connect_channel;
            sender.send(val);
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
