use log::info;
use yew::{function_component, html, Properties};
use crate::pages::main::state::RunningState;

#[derive(Properties, PartialEq)]
pub struct DecompiledProps {
    pub state: RunningState,
}

#[function_component(DecompiledCode)]
pub fn render_decompiled(props: &DecompiledProps) -> Html {
    let runtime_instr = props.state.mips_state.current_instr.unwrap_or(0);
    let decompiled = &props.state.decompiled;
    html! {
        for decompiled.as_str().split("\n").into_iter().map(|item| {
            if item == "" {
                // this is &nbsp;
                html! {
                    <tr>{"\u{00a0}"}</tr>
                }
            }
            else {
                let should_highlight = if item.starts_with("0x") {
                    // the actual hex address lives from 2-10, 01 are 0x
                    let source_instr = u32::from_str_radix(&item[2..10], 16).unwrap_or(0);
                    source_instr == runtime_instr
                } else {
                    false
                };

                html! {
                    <tr
                      class={
                        if should_highlight {
                          "bg-th-highlighting"
                        } else {
                          ""
                        }
                      }>
                        {item}
                    </tr>
                }
            }
        })
    }
}
