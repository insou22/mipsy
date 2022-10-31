use crate::components::data_segment::{FP_COLOR, SP_COLOR};
use crate::state::config::{MipsyWebConfig, RegisterBase};
use crate::state::state::{ErrorType, RegisterTab, State};
use crate::worker::{Worker, WorkerRequest};
use bounce::use_atom;
use derivative::Derivative;
use mipsy_lib::compile::breakpoints::WatchpointTarget;
use mipsy_lib::{Register, Safe};
use yew::{function_component, html, Properties, UseStateHandle, Callback};
use yew_agent::UseBridgeHandle;

#[derive(Properties, Derivative)]
#[derivative(PartialEq)]
pub struct RegisterProps {
    pub state: UseStateHandle<State>,
    pub tab: UseStateHandle<RegisterTab>,

    #[derivative(PartialEq = "ignore")]
    pub worker: UseBridgeHandle<Worker>,
}

#[function_component(Registers)]
pub fn render_running_registers(props: &RegisterProps) -> Html {
    let mips_state = match &*props.state {
        State::Compiled(state) => Some(state.mips_state.clone()),
        State::Error(ErrorType::RuntimeError(error)) => Some(error.mips_state.clone()),
        _ => None,
    };

    let config = use_atom::<MipsyWebConfig>();

    let show_uninitialised_registers = match &*props.tab {
        RegisterTab::AllRegisters => true,
        _ => false,
    };

    let registers = mips_state
        .clone()
        .map(|state| state.register_values.clone())
        .unwrap_or_else(|| vec![Safe::Uninitialised; 32]);

    let previous_registers = mips_state
        .map(|state| state.previous_registers.clone())
        .unwrap_or_else(|| vec![Safe::Uninitialised; 32]);

    html! {
        <table class="w-full border-collapse table-auto">
            <thead>
                <tr>
                    <th class="w-1/16">
                    {"Watch"}
                    </th>
                    <th class="w-1/4">
                    {"Register"}
                    </th>
                    <th class="w-3/4">
                    {"Value"}
                    </th>
                </tr>
            </thead>
            <tbody>
            {
                for registers.iter().enumerate().map(|(index, item)| {
                    if !show_uninitialised_registers &&
                        config.hide_uncommon_registers &&
                        (index == usize::from(Register::K0.to_number()) ||
                         index == usize::from(Register::K1.to_number()) ||
                         index == usize::from(Register::Gp.to_number())
                        )
                    {
                        html!{}
                    }
                    else if !show_uninitialised_registers && item == &Safe::Uninitialised {
                        html!{}
                    }
                    else {
                        let toggle_watchpoint = {
                            let worker = props.worker.clone();
                            Callback::from(move |_| {
                                worker.send(WorkerRequest::ToggleWatchpoint(index as u32))
                            })
                        };

                        let has_watchpoint = match &*props.state {
                            State::Compiled(curr) => {
                                let binary = curr.mips_state.binary.as_ref().unwrap();
                                binary.watchpoints.contains_key(
                                    &WatchpointTarget::Register(Register::from_u32(index as u32).unwrap())
                                )
                            }
                            State::Error(error_type) => {
                                if let ErrorType::RuntimeError(_error) = error_type {
                                    false
                                } else {
                                    unreachable!("Error in decompiled not possible if not compiled");
                                }
                            },
                            _ => unreachable!(),
                        };

                        html! {
                                <tr class={if registers[index] != previous_registers[index] {
                                        "bg-th-highlighting"
                                    } else {
                                        ""
                                    }
                                }>

                                    <td class="group w-10 text-center" >
                                        <button onclick={toggle_watchpoint}>
                                            if has_watchpoint {
                                                <StopIconFilled />
                                            } else {
                                                <StopIconOutline />
                                            }
                                        </button>
                                    </td>

                                    <td class="border-current border-b-2 pl-4 text-center"> {
                                            if index == Register::Sp.to_number() as usize {
                                                // make stack pointer green
                                                html! {
                                                    <span style={format!("color: {};", SP_COLOR)}>
                                                            {"$"}
                                                            {Register::from_u32(index as u32).unwrap().to_lower_str()}
                                                    </span>
                                                }
                                            }
                                            else if index == Register::Fp.to_number() as usize {
                                                html! {
                                                    <span style={format!("color: {};", FP_COLOR)}>
                                                            {"$"}
                                                            {Register::from_u32(index as u32).unwrap().to_lower_str()}
                                                    </span>
                                                }
                                            }
                                            else {
                                                html! {
                                                    <>
                                                        {"$"}
                                                        {Register::from_u32(index as u32).unwrap().to_lower_str()}
                                                    </>
                                                }
                                            }
                                        }
                                    </td>
                                    <td class="pl-4 border-b-2 border-current text-center">
                                        <pre>
                                            if let Safe::Valid(val) = item {
                                                {
                                                    match config.register_base {
                                                        RegisterBase::Hexadecimal => {
                                                            format!("0x{:08x}", val)
                                                        },
                                                        RegisterBase::Decimal => {
                                                            format!("{}", val)
                                                        },
                                                        RegisterBase::Binary => {
                                                            format!("0b{:b}", val)
                                                        },
                                                    }
                                                }
                                            } else {
                                                {"uninitialised"}
                                            }
                                        </pre>
                                    </td>
                            </tr>
                        }
                    }
                })
            }
            </tbody>
        </table>
    }
}

#[function_component(StopIconOutline)]
fn stop_icon_outline() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4"  fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
        </svg>
    }
}

#[function_component(StopIconFilled)]
fn stop_icon_filled() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z" clip-rule="evenodd" />
        </svg>
    }
}
