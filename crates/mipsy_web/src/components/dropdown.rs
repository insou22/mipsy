use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct DropDownProps {
    pub onchange: Callback<Event>,
    pub options: Vec<String>,
    pub label: String,
    pub hide_label: bool,
    pub selected_value: Option<String>,
}

#[function_component(Dropdown)]
pub fn dropdown(
    DropDownProps {
        onchange,
        options,
        label,
        hide_label,
        selected_value,
    }: &DropDownProps,
) -> Html {
    html! {
        <>
            <label for="options"
                class={
                    classes!(
                        "block", "mb-2", "text-sm", "font-medium",
                        "text-gray-900", "dark:text-gray-400",
                        if *hide_label { "hidden" } else { "" }
                    )
                }
            >
                {label}
            </label>
            <select {onchange} id="options" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                {
                    for options.iter().map(|opt| {
                        let is_selected = selected_value.as_deref() == Some(opt);
                        html! {
                            <option
                                selected={is_selected}
                                value={opt.clone()}
                            >
                                {opt}
                            </option>
                        }
                    })
                }
            </select>
        </>
    }
}
