use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ColorpickerProps {
    pub oninput: Callback<InputEvent>,
    pub color: String
}

#[function_component(ColorPicker)]
pub fn colorpicker(
    ColorpickerProps {
        oninput,
        color,
    }: &ColorpickerProps,
) -> Html {
    html! {
        <label for="default-toggle" class="">
            <input type="color" value={color.to_owned()} {oninput} id="default-toggle" class="peer" />
        </label>
    }
}
