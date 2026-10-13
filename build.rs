use std::env;
// use dotenvy::dotenv;
// use std::{fs::File, io::Write};

fn main() {
    println!("cargo:rerun-if-changed=.env");

    if let Ok(iter) = dotenvy::from_filename_iter(".env") {
        for (key, value) in env::vars() {
            println!("cargo::rustc-env={}={}", key, value);
        }
    }

}