#![allow(unused_unsafe)]
use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
pub mod components;
pub mod pages;
pub mod utils;
pub mod worker;
pub mod state;

#[wasm_bindgen]
extern "C" {
    pub fn split_setup();

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

    pub fn get_localstorage(key: &str) -> Option<String>;
    pub fn set_localstorage(key: &str, value: &str);
}
