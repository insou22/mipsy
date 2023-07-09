use crate::components::data_segment::{FP_COLOR, SP_COLOR};
use crate::components::decompiled::{StopIconFilled, StopIconOutline};
use crate::state::config::{MipsyWebConfig, RegisterBase};
use crate::state::state::{ErrorType, RegisterTab, State};
use crate::worker::{WorkerRequest, MipsyWebWorker};
use bounce::use_atom;
use derivative::Derivative;
use gloo_worker::WorkerBridge;
use mipsy_lib::compile::breakpoints::{TargetAction, WatchpointTarget};
use mipsy_lib::{Register, Safe, KDATA_BOT, TEXT_BOT};
use yew::{function_component, html, Html, Callback, Properties, UseStateHandle};

const MIXED_BASE_MIN_ADDRESS: u32 = TEXT_BOT - 1024;
const MIXED_BASE_MAX_ADDRESS: u32 = KDATA_BOT + 1024;

#[derive(Properties, Derivative)]
#[derivative(PartialEq)]
pub struct RegisterProps {
    pub tab: UseStateHandle<RegisterTab>,

    #[derivative(PartialEq = "ignore")]
    pub worker: UseStateHandle<WorkerBridge<MipsyWebWorker>>,
}

pub fn register_is_uncommon(index: usize) -> bool {
    index == usize::from(Register::K0.to_number()) ||
        index == usize::from(Register::K1.to_number()) ||
        index == usize::from(Register::Gp.to_number()) 
}

#[function_component(Registers)]
pub fn render_running_registers(props: &RegisterProps) -> Html {
    let state = use_atom::<State>();
    
    let mips_state = match &*state {
        State::Compiled(state) => Some(state.mips_state.clone()),
        State::Error(ErrorType::RuntimeError(error)) => Some(error.mips_state.clone()),
        _ => None,
    };

    let config = use_atom::<MipsyWebConfig>();

    let show_uninitialised_registers = matches!(&*props.tab, RegisterTab::AllRegisters);

    let registers = mips_state
        .clone()
        .map(|state| state.register_values)
        .unwrap_or_else(|| vec![Safe::Uninitialised; 32]);

    let previous_registers = mips_state
        .map(|state| state.previous_registers)
        .unwrap_or_else(|| vec![Safe::Uninitialised; 32]);

    html! {
        <table class="w-full border-collapse table-auto">
            <thead>
                <tr>
                    <th class="w-1/16">
                    {"Read"}
                    </th>
                    <th class="w-1/16">
                    {"Write"}
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
                    // dont' show uninit registers, or uncommon K0, K1, GP 
                    if !show_uninitialised_registers && item.as_option().is_none() || register_is_uncommon(index)  {
                        html!{}
                    }
                    else {
                        let toggle_read = {
                            let worker = props.worker.clone();
                            Callback::from(move |_| {
                                worker.send(WorkerRequest::ToggleWatchpoint(index as u32, TargetAction::ReadOnly))
                            })
                        };

                        let toggle_write = {
                            let worker = props.worker.clone();
                            Callback::from(move |_| {
                                worker.send(WorkerRequest::ToggleWatchpoint(index as u32, TargetAction::WriteOnly))
                            })
                        };

                        let watchpoint = match &*state {
                            State::Compiled(curr) => {
                                let binary = curr.mips_state.binary.as_ref().unwrap();
                                binary.watchpoints
                                    .get(&WatchpointTarget::Register(Register::from_u32(index as u32).unwrap()))
                            }
                            State::Error(error_type) => {
                                if let ErrorType::RuntimeError(_error) = error_type {
                                    None
                                } else {
                                    unreachable!("Error in decompiled not possible if not compiled");
                                }
                            },
                            State::NoFile => {
                                None
                            }
                        };

                        html! {
                                <tr class={if registers[index] != previous_registers[index] {
                                        "bg-th-highlighting"
                                    } else {
                                        ""
                                    }
                                }>

                                    <td class="text-center" >
                                        <button onclick={toggle_read}>
                                            if watchpoint.map_or(false, |wp| wp.action.fits(&TargetAction::ReadOnly)) {
                                                <StopIconFilled />
                                            } else {
                                                <StopIconOutline />
                                            }
                                        </button>
                                    </td>

                                    <td class="text-center" >
                                        <button onclick={toggle_write}>
                                            if watchpoint.map_or(false, |wp| wp.action.fits(&TargetAction::WriteOnly)) {
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
                                                        RegisterBase::Mixed => {
                                                            let as_u32 = *val as u32;

                                                            if (MIXED_BASE_MIN_ADDRESS..MIXED_BASE_MAX_ADDRESS).contains(&as_u32) {
                                                                format!("0x{:08x}", as_u32)
                                                            } else {
                                                                format!("{}", val)
                                                            }
                                                        }
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
