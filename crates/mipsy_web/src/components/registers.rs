use crate::state::state::{State, ErrorType};
use mipsy_lib::{Register, Safe};
use yew::{function_component, html, Properties, UseStateHandle};
#[derive(Properties, PartialEq)]
pub struct RegisterProps {
    pub state: UseStateHandle<State>,
}

#[function_component(Registers)]
pub fn render_running_registers(props: &RegisterProps) -> Html {
    let mips_state = match &*props.state {
        State::Compiled(state) => Some(state.mips_state.clone()),
        State::Error(ErrorType::RuntimeError(error)) => Some(error.mips_state.clone()),
        _ => None,
    };

    let registers = mips_state.clone()
        .map(|state| state.register_values.clone())
        .unwrap_or_else(|| vec![Safe::Uninitialised; 32]);
    
    let previous_registers = mips_state
        .map(|state| state.previous_registers.clone())
        .unwrap_or_else(|| vec![Safe::Uninitialised; 32]);

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
                    match item {
                        Safe::Valid(val) => {
                            html! {
                                <tr class={if registers[index] != previous_registers[index] {
                                        "bg-th-highlighting"
                                    } else {
                                        ""
                                    }
                                }>
                                    <td class="border-gray-500 border-b-2 pl-4 text-center"> {
                                            if index == 29 {
                                                html! {
                                                    <span style="color: green;">
                                                            {"$"}
                                                            {Register::from_u32(index as u32).unwrap().to_lower_str()}
                                                    </span>
                                                }
                                            }
                                            else if index == 30 {
                                                html! {
                                                    <span style="color: red;">
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
                                    <td class="pl-4 border-b-2 border-gray-500 text-center">
                                        <pre>
                                            {format!("0x{:08x}", val)}
                                        </pre>
                                    </td>
                            </tr>
                            }
                        }

                        Safe::Uninitialised => {html!{}}
                    }
                })
            }
            </tbody>
        </table>
    }
}
