#![recursion_limit = "1024"]
#![allow(clippy::clippy::large_enum_variant)]

pub mod app;
pub mod components;
pub mod pages;

use app::App;

pub fn main() {
    yew::start_app::<App>();
}
