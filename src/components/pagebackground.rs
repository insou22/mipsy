use yew::prelude::*;
use yew::{Children, Properties};

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
}

#[function_component(PageBackground)]
pub fn render_page_background(props: &Props) -> Html {
    html! {
        <div class="min-h-screen py-2 bg-th-primary">
            { for props.children.iter() }
        </div>
    }
}
