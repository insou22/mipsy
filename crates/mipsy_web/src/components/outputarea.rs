use crate::pages::main::app::ReadSyscalls;
use yew::prelude::*;
use yew::Properties;

#[derive(Properties, Clone, PartialEq)]
pub struct OutputProps {
    pub show_io: UseStateHandle<bool>,
    pub mipsy_output_tab_title: String,
    pub input_ref: NodeRef,
    pub on_input_keydown: Callback<KeyboardEvent>,
    pub running_output: Html,
    pub input_needed: Option<ReadSyscalls>,
}

#[function_component(OutputArea)]
pub fn render_output_area(props: &OutputProps) -> Html {
    let syscall_input_needed = match props.input_needed {
        Some(_) => true,
        None => false,
    };

    let input_maxlength = match &props.input_needed {
        Some(item) => match item {
            ReadSyscalls::ReadChar => "1".to_string(),
            // should we have a limit for size?
            ReadSyscalls::ReadInt => "".to_string(),
            ReadSyscalls::ReadDouble => "".to_string(),
            ReadSyscalls::ReadFloat => "".to_string(),
            ReadSyscalls::ReadString => "".to_string(),
        },
        None => "".to_string(),
    };

    let (io_tab_classes, mipsy_output_tab_classes) = {
        let default_tab_classes =
            "w-1/2 float-left border-t-2 border-r-2 border-current cursor-pointer px-1 py-2";
        let left_tab_classes = format!("{} border-l-2", default_tab_classes);
        let selected_classes = "bg-th-primary";
        let unselected_classes = "bg-th-tabunselected hover:bg-th-secondary";

        (
            format!(
                "{} {}",
                left_tab_classes,
                if *props.show_io {
                    selected_classes
                } else {
                    unselected_classes
                }
            ),
            format!(
                "{} {}",
                default_tab_classes,
                if *props.show_io {
                    unselected_classes
                } else {
                    selected_classes
                }
            ),
        )
    };

    let input_classes = if !syscall_input_needed {
        if *props.show_io {
            "block w-full cursor-not-allowed"
        } else {
            "hidden"
        }
    } else {
        "block w-full bg-th-highlighting"
    };

    let switch_to_io = {
        let show_io = props.show_io.clone();
        Callback::from(move |_| {
            show_io.set(true);
        })
    };

    let switch_to_errors = {
        let show_io = props.show_io.clone();
        Callback::from(move |_| {
            show_io.set(false);
        })
    };

    html! {
        <div id="output" class="min-w-full">
            <div style="height: 10%;" class="flex overflow-hidden border-1 border-current">
                <button class={io_tab_classes} onclick={switch_to_io.clone()}>{"I/O"}</button>
                <button
                    class={mipsy_output_tab_classes}
                    onclick={switch_to_errors.clone()}
                >
                    {props.mipsy_output_tab_title.clone()}
                </button>
            </div>
            <div
                style={if *props.show_io {"height: 80%;"} else {"height: 90%;"}}
                class="py-2 w-full flex overflow-y-auto flex-wrap-reverse bg-th-secondary px-2 border-2 border-current"
            >
                <div class="w-full overflow-y-auto">
                <h1>
                    <strong>
                        {if *props.show_io {"I/O"} else {"Mipsy Output"}}
                    </strong>
                </h1>
                <pre style="width:100%;" class="text-sm whitespace-pre-wrap">
                    {props.running_output.clone()}
                </pre>
                </div>
            </div>
            <div style="height: 10%;" class={if *props.show_io {"border-l-2 border-r-2 border-b-2 border-current"} else {"hidden"}}>
                <input
                    ref={props.input_ref.clone()}
                    id="user_input"
                    type="text"
                    maxlength={input_maxlength.clone()}
                    disabled={!syscall_input_needed}
                    onkeydown={props.on_input_keydown.clone()}
                    style="padding-left: 3px; height: 100%;"
                    class={input_classes} placeholder="> ..."/>
            </div>
        </div>
    }
}
