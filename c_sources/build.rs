extern crate pkg_config;
use std::{env, fs};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {

    // let target = PathBuf::from(&env::var("TARGET").unwrap());
    let current_dir = PathBuf::from(&env::current_dir().unwrap());
    let out_dir = PathBuf::from(&env::var_os("OUT_DIR").unwrap());

    let src_dir = current_dir.join("mosquitto-1.4.5");

    // match pkg_config::find_library("mosquitto") {
    //     Ok(_) => return,
    //     Err(e) => {
    //         panic!("Couldn't find mosquitto {:?}), install mosquitto first...",
    //                e)
    //     }
    // }


    // run(Command::new("make")
    //             .current_dir(&src_dir)
    //             .env("DESTDIR", &out_dir));

    // fs::copy(&src_dir.join("lib/libmosquitto.so.1"), &out_dir.join("libmosquitto.so")).unwrap();

    // //panic!("{:?}, {:?}", out_dir.display(), current_dir.display());

    println!("cargo:rustc-flags=-L {:?} -l mosquitto", out_dir.display());
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    // //println!("cargo:rustc-link-lib=static=mosquitto");
    // println!("cargo:root={:?}", out_dir.display());
}

fn run(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    assert!(cmd.stdout(Stdio::inherit())
               .stderr(Stdio::inherit())
               .status()
               .unwrap()
               .success());

}
