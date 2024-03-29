use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ToggleSwitchProps {
    pub onclick: Callback<MouseEvent>,
    pub checked: bool,
}

#[function_component(ToggleSwitch)]
pub fn toggle_switch(ToggleSwitchProps { onclick, checked }: &ToggleSwitchProps) -> Html {
    html! {
        <label for="checkbox-toggle" class="relative inline-flex items-center mb-4 cursor-pointer">
            <input type="checkbox" checked={*checked} {onclick} id="checkbox-toggle" class="sr-only peer" />
            // Lord bless tailwindcss
            <div class="w-11 h-6 bg-gray-200 rounded-full peer peer-focus:ring-4
                        peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 
                        dark:bg-gray-700 peer-checked:after:translate-x-full 
                        peer-checked:after:border-white after:content-[''] 
                        after:absolute after:top-0.5 after:left-[2px] 
                        after:bg-white after:border-gray-300 
                        after:border after:rounded-full after:h-5 
                        after:w-5 after:transition-all 
                        dark:border-gray-600 
                        peer-checked:bg-blue-600">
            </div>
        </label>
    }
}
