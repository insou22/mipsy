#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]
#![allow(unused_unsafe)]
use wasm_bindgen::prelude::*;
use yew::App;
pub mod app;
pub mod components;
pub mod pages;
pub mod worker;

use app::App as Application;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn split_setup();

    pub fn highlight();
}

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::{global, Reflect};

    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        App::<Application>::new().mount_to_body();
    } else {
        worker::Worker::register();
    }
    
    unsafe {
        split_setup();
    
    }
}
