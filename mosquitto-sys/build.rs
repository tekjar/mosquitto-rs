use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let current_dir = PathBuf::from(&env::current_dir().unwrap());
    let out_dir = PathBuf::from(&env::var_os("OUT_DIR").unwrap());

    //println!("cargo:rustc-flags=-L {:?} -l mosquitto", out_dir.display());
    //println!("cargo:rustc-link-search=native={}", out_dir.display());
}
