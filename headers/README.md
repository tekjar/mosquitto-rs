#### GENERATING BINDINGS

* Install mosquitto library

```
brew install mosquitto
```

* Export below library path to make bindgen work

```
export DYLD_LIBRARY_PATH=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/:$DYLD_LIBRARY_PATH
```

* Generate bindings

```
./bindgen -lmosquitto mosquitto.h -o ../src/bindings/mod.rs
```

* export below variable for rust to be able to find libmosquitto.so (Mac OSX)

```
export LIBRARY_PATH=/usr/local/lib
```

* test

```
cargo test -- --nocapture
``` 
