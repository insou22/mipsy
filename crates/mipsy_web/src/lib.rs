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

    pub fn init_editor();

    pub fn set_editor_value(value: &str);

    pub fn get_editor_value() -> String;

    pub fn trigger_download_file(filename: &str, content: &str);

    pub fn highlight_section(startLineNumber: u32, startColumn: u32,  endColumn: u32); 
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
