use crate::pages::main::state::State;
use mipsy_lib::{Register, Safe};
use yew::{function_component, html, Properties, UseStateHandle};

#[derive(Properties, PartialEq)]
pub struct RegisterProps {
    pub state: UseStateHandle<State>,
}

#[function_component(Registers)]
pub fn render_running_registers(props: &RegisterProps) -> Html {
    let mut registers = vec![Safe::Uninitialised; 32];
    let mut previous_registers = vec![Safe::Uninitialised; 32];
    if let State::Compiled(state) = &*props.state {
        registers = state.mips_state.register_values.clone();
        previous_registers = state.mips_state.previous_registers.clone();
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
                    match item {
                        Safe::Valid(val) => {
                            html! {
                                <tr class={if registers[index] != previous_registers[index] {
                                        "bg-th-highlighting"
                                    } else {
                                        ""
                                    }
                                }>
                                    <td class="border-gray-500 border-b-2 pl-4">
                                        {"$"}
                                        {Register::from_u32(index as u32).unwrap().to_lower_str()}
                                    </td>
                                    <td class="pl-4 border-b-2 border-gray-500">
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
