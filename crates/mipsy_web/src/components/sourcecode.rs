use log::info;
use mipsy_lib::MipsyError;
use yew::{function_component, html, Properties};
use yew::prelude::*;
use crate::pages::main::state::State;

#[derive(Properties, PartialEq)]
pub struct SourceCodeProps {
    pub file: Option<String>,
    pub state: UseStateHandle<State>,
}

#[function_component(SourceCode)]
pub fn render_source_code(props: &SourceCodeProps) -> Html {
    info!("calling render source table");
    let err_line_num = if let State::CompilerError(comp_err_state) = &*props.state {
        if let MipsyError::Compiler(err) = &comp_err_state.error {
            Some(err.line())
        }
         else {
            None
        }
    } else {
        None
    };

    html! {
        <div id="monaco_editor">
        </div>
    } 
}
