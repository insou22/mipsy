use mipsy_web::pages;

fn main() {
    let document = web_sys::window().unwrap().document().unwrap();
    let entry_point = document.get_element_by_id("yew_app").unwrap();
    yew::start_app_in_element::<pages::main::app::App>(entry_point);

    wasm_logger::init(wasm_logger::Config::default());

    mipsy_web::split_setup();
}
