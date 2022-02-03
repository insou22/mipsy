#![allow(unused_unsafe)]
use yew_agent::Threaded;
use wasm_bindgen::prelude::*;

pub mod components;
pub mod pages;
pub mod worker;
pub mod utils;
use pages::main::app::App as Application;

#[wasm_bindgen]
extern "C" {
    fn split_setup();

    pub fn highlight();
}

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::{global, Reflect};
    wasm_logger::init(wasm_logger::Config::default());
    unsafe {
        if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
            yew::start_app::<Application>();
        } else {
            worker::Worker::register();
        }
    
        split_setup();
    
    }
}
