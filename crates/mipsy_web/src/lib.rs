#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]

use wasm_bindgen::prelude::*;
use yew::App;
pub mod app;
pub mod components;
pub mod pages;
pub mod worker;

use app::App as Application;
use yew::prelude::*;

#[wasm_bindgen(module = "/split.js")]
extern "C" {
    fn split_setup();
}

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::{global, Reflect};

    yew::initialize();

    let document = yew::utils::document();

    let element = document
        .query_selector("#mount_application")
        .unwrap()
        .unwrap();

    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        App::<Application>::new().mount(element);
    } else {
        worker::Worker::register();
    }

    split_setup();

    yew::run_loop();
}
