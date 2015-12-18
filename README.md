#mosquitto-rs

Rust bindings and wrappers for mosquitto mqtt client library

###HOW TO BUILD
---

#####UBUNTU
* apt-get install libc-ares-dev libssl-dev libwrap0-dev uthash-dev uuid-dev
* cargo build
* 

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

