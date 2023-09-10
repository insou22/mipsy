use crate::{
    state::state::{ErrorType::RuntimeError, State},
    worker::{Worker, WorkerRequest},
};
use derivative::Derivative;
use yew::{classes, function_component, html, Callback, Properties, UseStateHandle};
use yew_agent::UseBridgeHandle;

#[derive(Properties, Derivative)]
#[derivative(PartialEq)]
pub struct DecompiledProps {
    pub current_instr: Option<u32>,
    pub decompiled: String,
    pub state: UseStateHandle<State>,

    #[derivative(PartialEq = "ignore")]
    pub worker: UseBridgeHandle<Worker>,
}

#[function_component(DecompiledCode)]
pub fn render_decompiled(props: &DecompiledProps) -> Html {
    let runtime_instr = props.current_instr.unwrap_or(0);
    let decompiled = &props.decompiled;
    html! {
            <pre class="text-xs">
            <table id="decompiled_output">
            { html! {
                for decompiled.as_str().split("\n").into_iter().map(|item| {
                    if item == "" {
                        // this is &nbsp;
                        html! {
                            <tr>{"\u{00a0}"}</tr>
                        }
                    }
                    else {
                        // the actual hex address lives from 2-10, 01 are 0x
                        let source_instr = if item.starts_with("0x") {
                            Some(u32::from_str_radix(&item[2..10], 16).unwrap_or(0))
                        } else {
                            None
                        };

                        let should_highlight = if let Some(source_instr) = source_instr {
                            source_instr == runtime_instr
                        } else {
                            false
                        };


                        let current_is_breakpoint = match &*props.state {

                            State::NoFile => unreachable!("cannot have decompiled if no file"),
                            State::Error(error_type) => {
                                if let RuntimeError(_error) = error_type {
                                    false
                                } else {
                                    unreachable!("Error in decompiled not possible if not compiled");
                                }
                            },
                            State::Compiled(curr) => {
                                let binary = curr.mips_state.binary.as_ref().expect("binary must exist");
                                let addr = if source_instr.is_none() {
                                    binary.get_label(&item.trim().replace(":", "")).ok().expect("label must exist")
                                } else {
                                    source_instr.expect("none case handled above")
                                };
                                binary.breakpoints.contains_key(&addr)
                            }

                        };

                        let toggle_breakpoint = {
                            let state = props.state.clone();
                            let item = String::from(item);
                            let source_instr = source_instr.clone();
                            let worker = props.worker.clone();
                            Callback::from(move |_| {

                                match &*state {

                                    State::NoFile => unreachable!(),
                                    State::Error(error_type) =>  {
                                        if let RuntimeError(error) = error_type {
                                            let binary = error.mips_state.binary.as_ref().expect("binary must exist");
                                            let addr = if source_instr.is_none() {
                                                binary.get_label(&item.trim().replace(":", "")).ok().expect("label must exist")
                                            } else {
                                                source_instr.expect("none case handled above")
                                            };
                                            worker.send(WorkerRequest::ToggleBreakpoint(addr));
                                        } else {
                                            unreachable!("Error in decompiled not possible if not compiled");
                                        }


                                    }

                                    State::Compiled(curr) => {
                                        let binary = curr.mips_state.binary.as_ref().expect("binary must exist");
                                        let addr = if source_instr.is_none() {
                                            binary.get_label(&item.trim().replace(":", "")).ok().expect("label must exist")
                                        } else {
                                            source_instr.expect("none case handled above")
                                        };
                                        worker.send(WorkerRequest::ToggleBreakpoint(addr));
                                    },
                                }
                            })
                        };
                        html! {
                            <tr
                              class={
                                classes!("", if should_highlight {
                                  "bg-th-highlighting"
                                } else {
                                  ""
                                })
                              }>
                                <td class="group w-10 text-center" >
                                    <button onclick={toggle_breakpoint} class={classes!("text-center", "text-xs", if !current_is_breakpoint {"group-hover:visible invisible"} else {""})}>
                                        if current_is_breakpoint {
                                            <StopIconFilled />
                                        } else {
                                            <StopIconOutline />
                                        }
                                    </button>
                                </td>
                                <td>
                                    {item}
                                </td>
                            </tr>
                        }
                    }
                })
            }}
            </table>
            </pre>
    }
}

#[function_component(StopIconOutline)]
pub(crate) fn stop_icon_outline() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4"  fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
        </svg>
    }
}

#[function_component(StopIconFilled)]
pub(crate) fn stop_icon_filled() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z" clip-rule="evenodd" />
        </svg>
    }
}
