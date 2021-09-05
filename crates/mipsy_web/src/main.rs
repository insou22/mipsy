#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]

use yew::{ App };
pub mod app;
pub mod components;
pub mod pages;

use app::App as Application;


pub fn main() {

    yew::initialize();
    
    let document = yew::utils::document();
    let element = document.query_selector("#mount_application").unwrap().unwrap();
    let app: yew::App<Application> = App::new();
    app.mount(element);


    yew::run_loop();
}
