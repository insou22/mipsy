#![allow(unused_unsafe)]
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
pub mod components;
pub mod pages;
pub mod state;
pub mod utils;
pub mod worker;

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

    pub fn update_editor_options(options: JsValue);
    pub fn update_editor_model_options(options: JsValue);

    pub fn set_cursor_position(line: u32, column: u32);
    pub fn get_cursor_position() -> JsValue;

    pub fn update_primary_color(color: &str);
    pub fn update_secondary_color(color: &str);
    pub fn update_tertiary_color(color: &str);
    pub fn update_highlight_color(color: &str);
    pub fn update_font_color(color: &str);
}
