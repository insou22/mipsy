use yew::prelude::*;

#[derive(PartialEq, Properties)]
struct ToggleSwitchProps {
    onclick: Callback<MouseEvent>,
    checked: bool,
}

#[function_component(ToggleSwitch)]
fn toggle_switch(
    ToggleSwitchProps {
        onclick,
        checked: _,
    }: &ToggleSwitchProps,
) -> Html {
    html! {
        <label for="default-toggle" class="relative inline-flex items-center mb-4 cursor-pointer">
            <input type="checkbox" value="" {onclick} id="default-toggle" class="sr-only peer" />
            <div class="w-11 h-6 bg-gray-200 rounded-full peer peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600">
            </div>
        </label>
    }
}
