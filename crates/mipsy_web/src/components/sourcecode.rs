use log::info;
use mipsy_lib::MipsyError;
use yew::{function_component, html, Properties};
use yew::prelude::*;
use crate::{utils::generate_highlighted_line, pages::main::state::State};

#[derive(Properties, PartialEq)]
pub struct SourceCodeProps {
    pub file: Option<String>,
    pub state: UseStateHandle<State>,
}

#[function_component(SourceCode)]
pub fn render_source_code(props: &SourceCodeProps) -> Html {
    info!("calling render source table");
    let file_len = props
        .file
        .as_ref()
        .unwrap_or(&"".to_string())
        .len()
        .to_string()
        .len();
    
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

    let highlighted_compiler_err = if let State::CompilerError(comp_err_state) = &*props.state {
        if let MipsyError::Compiler(err) = &comp_err_state.error {

            let default = "".to_string();
            let file = props.file.as_ref().unwrap_or(&default);
            
            generate_highlighted_line(file.into(), err)

        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };


    html! {
        // if we ever want to do specific things on specific lines...
        {
            for props.file.as_ref().unwrap_or(&"".to_string()).as_str().split("\n").into_iter().enumerate().map(|(index, item)| {
                html! {
                    <>

                        if Some((index + 1) as u32) == err_line_num {
                            <tr>
                                <pre>
                                    <code class="language-mips" style="padding: 0 !important;">
                                        <p>{highlighted_compiler_err.clone()}</p>
                                    </code>
                                </pre>
                            </tr>
                        } else {
                            if item == "" {
                                // this is &nbsp;
                                <tr>
                                    <pre>
                                        <code class="language-mips" style="padding: 0 !important;">
                                            {format!("{:indent$} ", index + 1, indent=file_len)}
                                            {"\u{00a0}"}
                                        </code>
                                    </pre>
                                </tr>
                            } else {
                                <tr>
                                    <pre>
                                        <code class="language-mips" style="padding: 0 !important;">
                                            {format!("{:indent$} ", index + 1, indent=file_len)}
                                            {item}
                                        </code>
                                    </pre>
                                </tr>
                            }
                        }
                    </>
                } 


            })
        }
    }
}
