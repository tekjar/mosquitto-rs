# mosquitto-rs

Rust bindings for mosquitto mqtt client library

####Create a client

```
let mut client = Client::new("test")
                    .keep_alive(30)
                    .clean_session(true);
```

####Connect to a broker
```
match client.connect("192.168.0.134"){
    Ok(_) => println!("Connection successful --> {:?}", client),
    Err(n) => panic!("Connection error = {:?}", n)
}
```

####Subscribe to a topic

```
client.subscribe("hello/world", Qos::AtMostOnce);

/* Callback when broker says that it received subscribe request */

client.onsubscribe_callback(|mid|println!("Subscribe request received for message {:?}", mid));
```

####Publish to a topic

```
client.publish("hello/world", "Hello World", Qos::AtMostOnce);
client.onpublish_callback(|mid|println!("Publish request received for message {:?}", mid));
```


####On message received callback closures

```
let i = 100;

client.onconnect_callback(|a:i32|println!("@@@ On connect callback {}@@@", a + i));
```

```
let mut count = 0;
client.onmesssage_callback(|s|{
                                count += 1;
                                println!("Message = {:?}, Count = {:?}", s, count);
                            });
```

