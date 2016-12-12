extern crate toml;
extern crate rustc_serialize;
#[macro_use]
extern crate glium;
extern crate portaudio;
extern crate num;
extern crate rustfft;

mod file_loader;
mod config;
mod audio;
mod display;

use config::load_config;
use audio::init_audio;
use display::display;

fn main() {
    let config = load_config();
    let (mut stream, buffer) = init_audio(&config).unwrap();
    stream.start().unwrap();
    display(&config, buffer);
    stream.stop().unwrap();
}
