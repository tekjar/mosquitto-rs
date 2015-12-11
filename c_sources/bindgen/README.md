
UPDATE BINDGEN UTILITY
---

* git clone https://github.com/crabtw/rust-bindgen.git
* sudo apt-get install clang
* cd /usr/lib/llvm-x.x/lib ; sudo ln -s libclang.so.1 libclang.so
* cargo build


GENERATE BINDINGS
---

* ./bindgen ../mosquitto-1.4.5/lib/mosquitto.h -o ../src/lib.rs
