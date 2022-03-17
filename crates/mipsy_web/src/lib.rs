#![allow(unused_unsafe)]
use wasm_bindgen::prelude::*;
use yew_agent::Threaded;
use wasm_bindgen::closure::Closure;
pub mod components;
pub mod pages;
pub mod utils;
pub mod worker;
pub mod state;
use pages::main::app::App as Application;

#[wasm_bindgen]
extern "C" {
    fn split_setup();

    pub fn init_editor();

    pub fn set_editor_value(value: &str);

    pub fn get_editor_value() -> String;

    pub fn get_localstorage_file_contents() -> String;
    pub fn set_localstorage_file_contents(value: &str);

    pub fn get_localstorage_filename() -> String;
    pub fn set_localstorage_filename(value: &str);

    pub fn trigger_download_file(filename: &str, content: &str);

    pub fn highlight_section(startLineNumber: u32, startColumn: u32, endColumn: u32);

    pub fn remove_highlight();

    pub fn set_model_change_listener(callback: &Closure<dyn Fn()>);
}

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::{global, Reflect};
    wasm_logger::init(wasm_logger::Config::default());

    unsafe {
        if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
            let document = web_sys::window().unwrap().document().unwrap();
            let entry_point = document.get_element_by_id("yew_app").unwrap();
            yew::start_app_in_element::<Application>(entry_point);
        } else {
            worker::Worker::register();
        }

        split_setup();
    }
}
