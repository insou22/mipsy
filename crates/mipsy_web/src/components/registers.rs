use crate::state::state::{ErrorType, RegisterTab, State};
use mipsy_lib::{Register, Safe};
use yew::{function_component, html, Properties, UseStateHandle};
#[derive(Properties, PartialEq)]
pub struct RegisterProps {
    pub state: UseStateHandle<State>,
    pub tab: UseStateHandle<RegisterTab>,
}

#[function_component(Registers)]
pub fn render_running_registers(props: &RegisterProps) -> Html {
    let mips_state = match &*props.state {
        State::Compiled(state) => Some(state.mips_state.clone()),
        State::Error(ErrorType::RuntimeError(error)) => Some(error.mips_state.clone()),
        _ => None,
    };

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

                    if show_uninitialised_registers || item != &Safe::Uninitialised {
                        html! {
                                <tr class={if registers[index] != previous_registers[index] {
                                        "bg-th-highlighting"
                                    } else {
                                        ""
                                    }
                                }>
                                    <td class="border-gray-500 border-b-2 pl-4 text-center"> {
                                            if index == 29 {
                                                // make stack pointer green
                                                html! {
                                                    <span style="color: green;">
                                                            {"$"}
                                                            {Register::from_u32(index as u32).unwrap().to_lower_str()}
                                                    </span>
                                                }
                                            }
                                            else if index == 30 {
                                                // make frame pointer red
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
                                            if let Safe::Valid(val) = item {
                                                {format!("0x{:08x}", val)}
                                            } else {
                                                {"uninitialised"}
                                            }
                                        </pre>
                                    </td>
                            </tr>
                        }
                    } else {
                        html! {}
                    }
                })
            }
            </tbody>
        </table>
    }
}
