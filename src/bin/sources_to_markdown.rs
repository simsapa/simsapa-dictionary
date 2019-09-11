extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate kankyo;

use std::env;
use std::path::PathBuf;

use simsapa_dictionary::models::DictWord;

fn main() {
    //std::env::set_var("RUST_LOG", "simsapa_dictionary=info");

    match kankyo::init() {
        Ok(_) => {}
        Err(e) => info!("Couldn't find a .env file: {:?}", e),
    }

    env_logger::init();
    info!("🚀 Launched");

    let p = match env::var("SUTTACENTRAL_ROOT") {
        Ok(x) => x,
        Err(_) => panic!("Missing env var: SUTTACENTRAL_ROOT"),
    };
    let suttacentral_root = PathBuf::from(p);
    if ! suttacentral_root.exists() {
        panic!("Folder doesn't exist: {:?}", &suttacentral_root);
    }

    println!("yay!");
}
