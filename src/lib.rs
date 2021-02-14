#![feature(map_first_last)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate walkdir;

#[macro_use]
extern crate diesel;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate calamine;

extern crate html2md;
extern crate zip;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate kankyo;
extern crate chrono;

extern crate comrak;
extern crate handlebars;
extern crate deunicode;

extern crate pali_dict_core;

pub mod app;
pub mod dictionary;
pub mod error;
pub mod helpers;
pub mod sc_data;
pub mod db_models;
pub mod db_schema;

