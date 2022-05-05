use log::info;
use yew::prelude::*;
use yew::{function_component, html, Properties};

#[derive(Properties, Debug, PartialEq)]
pub struct SourceCodeProps {
    pub file: Option<String>,
    pub save_keydown: Callback<KeyboardEvent>,
}

#[function_component(SourceCode)]
pub fn render_source_code(props: &SourceCodeProps) -> Html {
    html! {
        <div onkeydown={props.save_keydown.clone()} id="monaco_editor">
        </div>
    }
}
