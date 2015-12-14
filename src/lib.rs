extern crate libc;
extern crate c_sources as bindings;

use std::ptr;
use std::mem;
use std::ffi::{CString, CStr};
use std::collections::HashMap;

// #[derive(Default)]
// #[derive(Debug)]
pub struct Client<'b, 'c, 'd> {
    pub id: String,
    pub user_name: Option<&'b str>,
    pub password: Option<&'c str>,
    pub host: Option<&'d str>,
    pub keep_alive: i32,
    pub clean_session: bool,
    pub callbacks: HashMap<String, Box<Fn(i32)>>,
    pub mosquitto: *mut bindings::Struct_mosquitto,
}

pub enum Qos {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl<'b, 'c, 'd> Client<'b, 'c, 'd>{

    pub fn new(id: &str) -> Client {

        let mosquitto: *mut bindings::Struct_mosquitto;
        let callbacks: HashMap<String, Box<Fn(i32)>> = HashMap::new();

        let mut client = Client {
            id: id.to_string(),
            user_name: None,
            password: None,
            host: None,
            keep_alive: 10,
            clean_session: true,
            callbacks: callbacks,
            mosquitto: ptr::null_mut(),
        };

        let id = CString::new(id);
        unsafe {
            //Creating new client and setting reference to Client callbacks as userdata.
            //let c = &client as *const _ as *mut libc::c_void;
            let i = 10;
            let c = &i as *const _ as *mut libc::c_void;
            println!("###### {:?}", c);
            client.mosquitto = bindings::mosquitto_new(id.unwrap().as_ptr(), true as u8, c);
            bindings::mosquitto_user_data_set(client.mosquitto, c);
        }
        client
    }

    pub fn auth(mut self, user_name: &'b str, password: &'c str) -> Self {
        self.user_name = Some(user_name);
        self.password = Some(password);
        self
    }

    pub fn keep_alive(mut self, keepalive: i32) -> Self {
        self.keep_alive = keepalive;
        self
    }

    pub fn clean_session(mut self, clean: bool) -> Self {
        self.clean_session = clean;


        // Reinitialise the client if clean_session is changed to false
        if clean == false {

            unsafe {
                let id = self.id.clone();
                let id = CString::new(id);

                bindings::mosquitto_reinitialise(self.mosquitto,
                                                 id.unwrap().as_ptr(),
                                                 clean as u8,
                                                 ptr::null_mut());
            }
        }

        self
    }

    pub fn will(self, topic: &str, message: &str) -> Self {

        let msg_len = message.len();
        let topic = CString::new(topic);
        let message = CString::new(message);

        unsafe {
            // Publish will with Qos 2
            bindings::mosquitto_will_set(self.mosquitto,
                                         topic.unwrap().as_ptr(),
                                         msg_len as i32,
                                         message.unwrap().as_ptr() as *mut libc::c_void,
                                         2,
                                         0);
        }

        self

    }

    pub fn connect(&mut self, host: &'d str) -> Result<&Self, i32> {

        self.host = Some(host);

        let host = CString::new(host);

        let n_ret;
        let u_name;
        let pwd;

        // Set username and password before connecting
        match self.user_name {
            Some(user_name) => {
                u_name = CString::new(user_name);
                match self.password {
                    Some(password) => {
                        println!("user_name = {:?}, password = {:?}", user_name, password);
                        pwd = CString::new(password);
                        unsafe {
                            bindings::mosquitto_username_pw_set(self.mosquitto,
                                                                u_name.unwrap().as_ptr(),
                                                                pwd.unwrap().as_ptr());
                        }
                    }
                    None => (),
                }

            }
            None => (),
        }

        // Connect to broker
        unsafe {
            n_ret = bindings::mosquitto_connect(self.mosquitto,
                                                host.unwrap().as_ptr(),
                                                1883,
                                                self.keep_alive);
            if n_ret == 0 {
                Ok(self)
            } else {
                Err(n_ret)
            }
        }
    }

    // Registered callback is called when the broker sends a CONNACK message in response
    // to a connection. Will be called even incase of failure. All your sub/pub stuff
    // should ideally be done in this callback when connection is successful
    pub fn onconnect_callback<F>(&mut self, callback:  F)
        where F: Fn(i32), F: 'static
    {
        self.callbacks.insert("on_connect".to_string(), Box::new(callback));
        let cb = &self.callbacks as *const _ as *mut libc::c_void;
        println!("###### {:?}", cb);

        unsafe {
            // Set our closure as user data
            bindings::mosquitto_user_data_set(self.mosquitto, cb);
            // Register callback
            bindings::mosquitto_connect_callback_set(self.mosquitto, Some(onconnect_wrapper));
        }

        // Registered callback. user data is our closure
        unsafe extern "C" fn onconnect_wrapper(mqtt: *mut bindings::Struct_mosquitto,
                                                  closure: *mut libc::c_void,
                                                  val: libc::c_int)
        {
            println!("###### {:?}", closure);

            let callbacks: &mut HashMap<String, Box<Fn(i32)>> = mem::transmute(closure);
            match (*callbacks).get("on_connect") {
                Some(cb) => cb(val as i32),
                _ => panic!("No callback found"),
            }
        }
        //mem::forget(cb);
    }

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
                                          topic.unwrap().as_ptr(),
                                          qos);
        }
    }

    // Call back that will be called when broker responds to a subscription
    pub fn onsubscribe_callback<F>(&mut self, callback: F)
        where F: Fn(i32), F:'static
    {
        self.callbacks.insert("on_subscribe".to_string(), Box::new(callback));
        let cb = &self.callbacks as *const _ as *mut libc::c_void;

        unsafe {
            bindings::mosquitto_user_data_set(self.mosquitto, cb);
            bindings::mosquitto_subscribe_callback_set(self.mosquitto,
                                                       Some(onsubscribe_wrapper));
        }

        unsafe extern "C" fn onsubscribe_wrapper(mqtt: *mut bindings::Struct_mosquitto,
                                                    closure: *mut libc::c_void,
                                                    mid: libc::c_int,
                                                    qos_count: libc::c_int,
                                                    qos_list: *const ::libc::c_int)
        {
                        println!("###### {:?}", closure);

            let callbacks: &mut HashMap<String, Box<Fn(i32)>> = mem::transmute(closure);
            match callbacks.get("on_subscribe") {
                Some(cb) => cb(mid as i32),
                _ => panic!("No callback found"),
            }
        }
    }


    pub fn publish(&self, topic: &str, message: &str, qos: Qos) {

        let msg_len = message.len();
        //
        // CString::new(topic).unwrap().as_ptr() is wrong.
        // topic String gets destroyed and pointer is invalidated
        // Whem message is created, it will allocate to destroyed space of 'topic'
        // topic is now pointing to it and publish is happening on the same message String.
        //
        // Try let topic = CString::new(topic).unwrap().as_ptr(); instead of let topic = CString::new(topic)
        //

        let topic = CString::new(topic);
        let message = CString::new(message);

        let qos = match qos {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2,
        };

        unsafe {
            bindings::mosquitto_publish(self.mosquitto,
                                        ptr::null_mut(),
                                        topic.unwrap().as_ptr(),
                                        msg_len as i32,
                                        message.unwrap().as_ptr() as *mut libc::c_void,
                                        qos,
                                        0);
        }
    }


    pub fn onpublish_callback<F>(&mut self, callback: F)
        where F: Fn(i32), F: 'static
    {
        self.callbacks.insert("on_publish".to_string(), Box::new(callback));
        let cb = &self.callbacks as *const _ as *mut libc::c_void;

        unsafe {
            bindings::mosquitto_user_data_set(self.mosquitto, cb);
            bindings::mosquitto_publish_callback_set(self.mosquitto, Some(onpublish_wrapper));
        }

        unsafe extern "C" fn onpublish_wrapper(mqtt: *mut bindings::Struct_mosquitto,
                                                  closure: *mut libc::c_void,
                                                  mid: libc::c_int)
        {
            let callbacks: &mut HashMap<String, Box<Fn(i32)>> = mem::transmute(closure);
            match callbacks.get("on_publish") {
                Some(cb) => cb(mid as i32),
                _ => panic!("No callback found"),
            }
        }
    }


    // pub fn onmesssage_callback<F>(&self, callback: F)
    //     where F: Fn(&str), F:'static
    // {
    //     self.callbacks.insert("on_message".to_string(), Box::new(callback));
    //     let cb = &self.callbacks as *const _ as *mut libc::c_void;
    //     unsafe {
    //         bindings::mosquitto_user_data_set(self.mosquitto, cb); /* Set our closure as user data */
    //         bindings::mosquitto_message_callback_set(self.mosquitto, Some(onmessage_wrapper));
    //     }


    //     unsafe extern "C" fn onmessage_wrapper(mqtt: *mut bindings::Struct_mosquitto, closure: *mut libc::c_void, mqtt_message: *const bindings::Struct_mosquitto_message)
    //     {

    //         let mqtt_message = (*mqtt_message).payload as *const i8;
    //         let mqtt_message = CStr::from_ptr(mqtt_message).to_bytes();
    //         let mqtt_message = std::str::from_utf8(mqtt_message).unwrap();

    //         let callbacks: &mut HashMap<String, Box<Fn(&str)>> = mem::transmute(closure);
    //         match callbacks.get("on_publish") {
    //             Some(cb) => cb(mqtt_message),
    //             _ => panic!("No callback found"),
    //         }
    //     }
    // }

    pub fn loop_forever(&self) {
        unsafe {
            bindings::mosquitto_loop_forever(self.mosquitto, 1000, 1000);
        }
    }
}


impl<'b, 'c, 'd> Drop for Client<'b, 'c, 'd>{
    fn drop(&mut self) {
        unsafe {
            bindings::mosquitto_destroy(self.mosquitto);
            bindings::mosquitto_lib_cleanup();
        }
    }
}
