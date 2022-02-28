use log::info;
use yew::prelude::*;
use yew::{function_component, html, Properties};

#[derive(Properties, Debug)]
pub struct SourceCodeProps {
    pub file: Option<String>,
    pub save_keydown: Callback<KeyboardEvent>,
}

impl PartialEq for SourceCodeProps {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file
    }
}

#[function_component(SourceCode)]
pub fn render_source_code(props: &SourceCodeProps) -> Html {
    info!("calling render source table");
    html! {
        <div onkeydown={props.save_keydown.clone()} id="monaco_editor">
        </div>
    }
}
