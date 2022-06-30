use crate::components::{color_picker::ColorPicker, dropdown::Dropdown, heading::Heading};
use crate::state::config::{MipsyWebConfig, PrimaryColor, SecondaryColor, TertiaryColor};
use bounce::use_atom;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::{prelude::*, Properties};

#[derive(Properties, Clone, PartialEq)]
pub struct ModalProps {
    pub should_display: UseStateHandle<bool>,
    pub analytics: UseStateHandle<bool>,
}

#[function_component(SettingsModal)]
pub fn render_modal(props: &ModalProps) -> Html {
    let config = use_atom::<MipsyWebConfig>();

    let classes = if *props.should_display {
        "modal overflow-auto bg-th-primary border-black border-2 absolute top-14 w-3/4 z-20 text-sm"
    } else {
        "modal hidden"
    };

    let is_opt_out: UseStateHandle<bool> =
        use_state_eq(|| match crate::get_localstorage("analytics-opt-out") {
            Some(val) => val == "true",
            None => false,
        });

    html! {
        <div class={classes} id="modal1" style="left: 13%;">
            <div class="modal-dialog">
                <div class="absolute modal-header top-0 right-0 h-16 w-16">
                    <div onclick={{
                            let display_modal = props.should_display.clone();
                            Callback::from(move |_| {
                            display_modal.set(!*display_modal);
                        })}}
                    class="
                        rounded text-center cursor-pointer text-6xl border-black 
                        border-2 hover:bg-red-700 border-none bg-transparent close-modal" 
                    aria-label="close">
                    {"x"}
                    </div>
                </div>
                <section class="modal-content p-2 flex items-center flex-col">

                    <h1 class="my-2 text-2xl">
                        <strong>{"Settings"}</strong>
                    </h1>
                    <br />

                    <div class="w-9/12">
                        <Heading
                            title="Editor Font Size"
                            subtitle="Adjust the font size of the editor"
                        />

                        <div class="w-3/12">
                            <Dropdown
                                onchange={
                                    let config = config.clone();
                                    Callback::from(move |e: Event| {
                                        let input: HtmlSelectElement = e.target_unchecked_into();
                                        let val = input.value();

                                        #[derive(Serialize, Deserialize)]
                                        struct Options {
                                            #[serde(rename = "fontSize")]
                                            font_size: u32,
                                        }
                                        let font_size = val.parse::<u32>().unwrap();
                                        config.set(MipsyWebConfig {
                                            font_size,
                                            ..(*config).clone()
                                        });
                                        crate::update_editor_options(
                                            JsValue::from_serde(&Options {
                                                font_size,
                                            }).unwrap()
                                        );
                                })}
                                label={"font size"}
                                hide_label={true}
                                // TODO - config selected, min max and font step
                                selected_value={(*config).font_size.to_string()}
                                options={
                                    (10..=70_i32)
                                        .step_by(2)
                                        .map(|x| x.to_string())
                                        .collect::<Vec<String>>()
                                }
                            />
                        </div>

                        <Heading
                            title="Tab Size"
                            subtitle="Adjust the tab size of the editor"
                        />

                        <div class="w-3/12">
                            <Dropdown
                                onchange={
                                    let config = config.clone();
                                    Callback::from(move |e: Event| {
                                    let input: HtmlSelectElement = e.target_unchecked_into();
                                    let val = input.value();
                                    let tab_size = val.parse::<u32>().unwrap();

                                    #[derive(Serialize, Deserialize)]
                                    struct Options {
                                        #[serde(rename = "tabSize")]
                                        tab_size: u32,
                                    }

                                    config.set(MipsyWebConfig {
                                        tab_size,
                                        ..(*config).clone()
                                    });
                                    crate::update_editor_model_options(
                                        JsValue::from_serde(&Options {
                                            tab_size,
                                        }).unwrap()
                                    );
                                })}
                                label={"tab size"}
                                hide_label={true}
                                selected_value={(*config).tab_size.to_string()}
                                // TODO - config selected, min max and font step
                                options={
                                    (2..=8_i32)
                                        .step_by(1)
                                        .map(|x| x.to_string())
                                        .collect::<Vec<String>>()
                                }
                            />
                        </div>

                        <Heading
                            title="Primary color"
                            subtitle="Adjust the primary color"
                        />

                        <div class="flex flex-row items-center">
                        <ColorPicker
                            oninput={
                                let config = config.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    let val = input.value();
                                    let color = val.parse::<String>().unwrap();
                                    config.set(MipsyWebConfig {
                                        primary_color: color.into(),
                                        ..(*config).clone()
                                    });
                                    crate::update_primary_color(&val);
                                })
                            }
                            color={(&*config.primary_color.0).to_string()}
                        />

                        <button
                            type="button"
                            class="text-black m-2 p-2 border bg-th-primary hover:bg-th-secondary border-black round"
                            onclick={
                                let config = config.clone();
                                Callback::from(move |_| {
                                    config.set(MipsyWebConfig {
                                        primary_color: PrimaryColor::default(),
                                        ..(*config).clone()
                                    });
                                    crate::update_primary_color(&*PrimaryColor::default().0);
                                })
                            }
                        >

                            {"reset"}
                        </button>
                        </div>

                        <Heading
                            title="Secondary color"
                            subtitle="Used for the background of the editor, register and IO areas"
                        />


                        <div class="flex flex-row items-center">
                        <ColorPicker
                            oninput={
                                let config = config.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    let val = input.value();
                                    let color = val.parse::<String>().unwrap();
                                    config.set(MipsyWebConfig {
                                        secondary_color: color.into(),
                                        ..(*config).clone()
                                    });
                                    crate::update_secondary_color(&val);
                                })
                            }
                            color={(&*config.secondary_color.0).to_string()}
                        />

                        <button
                            type="button"
                            class="text-black m-2 p-2 border bg-th-primary hover:bg-th-secondary border-black rounde"
                            onclick={
                                let config = config.clone();
                                Callback::from(move |_| {
                                    config.set(MipsyWebConfig {
                                        secondary_color: SecondaryColor::default(),
                                        ..(*config).clone()
                                    });
                                    crate::update_secondary_color(&*SecondaryColor::default().0);
                                })
                            }
                        >

                            {"reset"}
                        </button>
                        </div>

                        <Heading
                            title="Tertiary color"
                            subtitle="Used for unselected tabs"
                        />

                        <div class="flex flex-row items-center">
                        <ColorPicker
                            oninput={
                                let config = config.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    let val = input.value();
                                    let color = val.parse::<String>().unwrap();
                                    config.set(MipsyWebConfig {
                                        tertiary_color: color.into(),
                                        ..(*config).clone()
                                    });
                                    crate::update_tertiary_color(&val);
                                })
                            }
                            color={(&*config.tertiary_color.0).to_string()}
                        />

                        <button
                            type="button"
                            class="text-black m-2 p-2 border bg-th-primary hover:bg-th-secondary border-black rounded"
                            onclick={
                                let config = config.clone();
                                Callback::from(move |_| {
                                    config.set(MipsyWebConfig {
                                        tertiary_color: TertiaryColor::default(),
                                        ..(*config).clone()
                                    });
                                    crate::update_tertiary_color(&*TertiaryColor::default().0);
                                })
                            }
                        >

                            {"reset"}
                        </button>
                        </div>
                        <Heading
                            title="Analytics"
                            subtitle="Analytics is currently not implemented"
                        />

                        // disable analytics info until implemented
                        if false {
                            <AnalyticsInformation {is_opt_out} />
                        }
                    </div>
                </section>
                <footer class="modal-footer"></footer>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct AnalyticsInfoProps {
    pub is_opt_out: UseStateHandle<bool>,
}

#[function_component(AnalyticsInformation)]
pub fn analytics_info(AnalyticsInfoProps { is_opt_out }: &AnalyticsInfoProps) -> Html {
    html! {
        <>
        <p>
            {"You are currently opted "}
            <b>
            {
                if !**is_opt_out {
                    "IN to"
                } else {
                    "OUT of"
                }
            }
            </b>
            {" analytics."}
        </p>

        <button class="
            bg-th-primary border-2 border-black
            hover:bg-red-700 
            font-bold py-2 px-4 rounded
            m-2"
            onclick={{
                let opt_out = is_opt_out.clone();
                Callback::from(move |_| {
                    crate::set_localstorage("analytics-opt-out", if !*opt_out {"true"} else {"false"});
                    opt_out.set(!*opt_out);
                })}}
            >
            {
                if **is_opt_out {
                    "Opt in"
                } else {
                    "Opt out"
                }
            }
        </button>

        //TODO - put the below in an expand block
        <p>
            {"mipsy-web will in future use analytics to track the following information: "}
        </p>
        <table class="border-2 border-black border-collapse">
            <th class="border-2 border-black">
                {"Name"}
            </th>
            <th class="border-2 border-black">
                {"Description"}
            </th>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">
                    {"Session ID"}
                </td>
                <td class="p-1 border-2 border-black">
                    {"A uuid generated for each session"}
                </td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Seconds spent"}</td>
                <td class="p-1 border-2 border-black">{"The number of seconds spent on the app"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Load button count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the load button is clicked in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Save button count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the save button is clicked in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Run button count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the run button is clicked in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Kill button count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the kill button is clicked in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Reset button count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the reset button is clicked in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Code editor time"}</td>
                <td class="p-1 border-2 border-black">{"Seconds spent editing code in the monaco editor"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Step back count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the step back button is clicked in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Step forward count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the step forward button is clicked in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Download count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the download button is clicked in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Compiler error count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the compiler has an error in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Compiler error type"}</td>
                <td class="p-1 border-2 border-black">{"The type of error every time there is a compiler error"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Parser error count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the parser has an error in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Parser error type"}</td>
                <td class="p-1 border-2 border-black">{"The type of error every time there is a parser error"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Runtime error count"}</td>
                <td class="p-1 border-2 border-black">{"The number of times the runtime has an error in a session"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Runtime error type"}</td>
                <td class="p-1 border-2 border-black">{"The type of error every time there is a runtime error"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Time in decompiled tab"}</td>
                <td class="p-1 border-2 border-black">{"The number of seconds spent in the decompiled tab"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Time in data tab"}</td>
                <td class="p-1 border-2 border-black">{"The number of seconds spent in the data tab"}</td>
            </tr>
            <tr class="border-2 border-black">
                <td class="p-1 border-2 border-black">{"Time in code tab"}</td>
                <td class="p-1 border-2 border-black">{"The number of seconds spent in the code tab"}</td>
            </tr>
        </table>
        </>
    }
}
