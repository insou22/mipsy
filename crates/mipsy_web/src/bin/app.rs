use bounce::BounceRoot;
use mipsy_web::pages;
use yew::prelude::*;

#[function_component(AppWrapper)]
pub fn app_wrapper() -> Html {
    html! {
        <BounceRoot>
            <pages::main::app::App />
        </BounceRoot>
    }
}

fn main() {
    let document = web_sys::window().unwrap().document().unwrap();
    let entry_point = document.get_element_by_id("yew_app").unwrap();

    let result = yew::Renderer::<AppWrapper>::with_root(entry_point).render();

    wasm_logger::init(wasm_logger::Config::default());

}
