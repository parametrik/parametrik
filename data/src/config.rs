use std::fs::File;
use std::io::prelude::*;

fn get_env(key: &str) -> String {
    std::env::var(key).expect(&format!("{} must be set", key.to_string()))
}

lazy_static! {
    pub static ref DATABASE_URL: String = get_env("DATABASE_URL");
    pub static ref DOMAIN: String = get_env("DOMAIN");
}

pub fn get_secret_key() -> Vec<u8> {
    let mut file = File::open("../private_rsa.der").expect("Missing private key file");
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer).expect("Could not read file");
    buffer
}

pub fn get_public_key() -> Vec<u8> {
    let mut file = File::open("../public_rsa.der").expect("Missing public key file");
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer).expect("Could not read file");
    buffer
}
