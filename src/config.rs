use std::env;
use rustc_serialize::Decodable;
use toml::{Decoder, Value, Parser};

use file_loader::load_from_file;

#[derive(Debug, RustcDecodable)]
pub struct Config {
    pub n: u32,
    pub max_buffers: usize,
    pub uniforms: Uniforms,
    pub debug: DebugConfig,
}
#[derive(Debug, RustcDecodable)]
pub struct Uniforms {
    pub decay: f32,
    pub thickness: f32,
    pub thinning: f32,
    pub base_hue: f32,
    pub colorize: bool,
}
#[derive(Debug, RustcDecodable)]
pub struct DebugConfig {
    pub print_drop: bool,
}

pub fn load_config() -> Config {
    let config_filename = env::args().nth(1)
        .unwrap_or_else(|| panic!("put in a config file"));
    let config_file = load_from_file(&config_filename);
    let mut parser = Parser::new(&config_file);
    let config = match parser.parse() {
        Some(config) => config,
        None => {
            for err in &parser.errors {
                println!("{}", err);
            }
            panic!("invalid config file");
        }
    };
    Config::decode(&mut Decoder::new(Value::Table(config)))
        .unwrap_or_else(|e| panic!("invalid config file: {}", e))
}
