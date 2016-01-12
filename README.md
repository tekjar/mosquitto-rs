#mosquitto-rs ![travis](https://travis-ci.org/kteza1/mosquitto-rs.svg?branch=master)

Rust bindings and wrappers for mosquitto mqtt client library

###HOW TO BUILD
---

#####UBUNTU

* apt-get install libc-ares-dev libssl-dev libwrap0-dev uthash-dev uuid-dev
* cargo build

#####YOCTO

Get the sources and

* opkg install libcares-dev openssl-dev
* make && make install

####SETUP TLS CONNECTIONS

* Generate ca, server, client certificates using the guide [here](http://rockingdlabs.dunmire.org/exercises-experiments/ssl-client-certs-to-secure-mqtt)

* Use the below commands to verify your connection
```
sudo openssl s_client -connect localhost:8884 -CAfile ./ca.crt -cert client.crt -key client.key
```
```
mosquitto_sub -t "ather/log-ship" -v --cafile ca.crt --cert client.crt --key client.key -p 8884
```


###API USAGE EXAMPLES
---

####Create a client

```
let mut client = Client::new("test")
                    .keep_alive(5)
                    .clean_session(false)
                    .will("goodbye", "my last words");
```

####Connect to a broker

```
match client.connect("localhost"){
    Ok(_) => println!("Connection successful --> {:?}", client),
    Err(n) => panic!("Connection error = {:?}", n)
}
```

####Subscribe to a topic

```
/* Callback when broker says that it received subscribe request */

client.onsubscribe_callback(|mid|{
                                    println!("Subscribe request received for message {:?}", mid)
                                  });

client.subscribe("hello/world", Qos::AtMostOnce);
```

####Publish to a topic

```
client.onpublish_callback(|mid|{
                                    println!("Publish request received for message {:?}", mid)
                                });

client.publish("hello/world", "Hello World", Qos::AtMostOnce);
```


####On message received callback closures

```
let i = 100;

client.onconnect_callback(|a:i32|{
                                    println!("@@@ On connect callback {}@@@", a + i)
                                  });
```

```
let mut count = 0;

client.onmesssage_callback(|s|{
                                count += 1;
                                println!("Message = {:?}, Count = {:?}", s, count);
                            });
```

####EXTENDING THE MOSQUITTO BROKER FOR 100 THOUSAND CONNECTIONS
---

check [this](https://lists.launchpad.net/mosquitto-users/msg00163.html)
