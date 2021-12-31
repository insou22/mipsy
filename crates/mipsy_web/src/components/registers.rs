use crate::pages::main::{app::State, state::RunningState};
use std::{cell::RefCell, rc::Rc};
use yew::{function_component, html, Properties};
use log::info;
use mipsy_lib::{Safe, Register};

#[derive(Properties, PartialEq)]
pub struct RegisterProps {
    pub state: State,
}

/*
impl PartialEq for RegisterProps {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

*/

#[function_component(Registers)]
pub fn render_running_registers(props: &RegisterProps) -> Html {
    let mut registers = vec![Safe::Uninitialised; 32];
    if let State::Running(state) = &props.state {
        registers = state.mips_state.register_values.clone();
    };

    html! {
        <table class="w-full border-collapse table-auto">
            <thead>
                <tr>
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
                    html! {
                        <tr>
                        {
                            match item {
                                Safe::Valid(val) => {
                                    html! {
                                        <>
                                            <td class="border-gray-500 border-b-2 pl-4">
                                                {"$"}
                                                {Register::from_u32(index as u32).unwrap().to_lower_str()}
                                            </td>
                                            <td class="pl-4 border-b-2 border-gray-500">
                                                <pre>
                                                    {format!("0x{:08x}", val)}
                                                </pre>
                                            </td>
                                        </>
                                    }
                                }

                                Safe::Uninitialised => {html!{}}
                            }
                        }
                        </tr>
                    }
                })
            }
            </tbody>
        </table>
    }
}
