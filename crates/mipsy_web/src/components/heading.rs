use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HeadingProps {
    pub title: String,
    pub subtitle: Option<String>,
}

#[function_component(Heading)]
pub fn heading(HeadingProps { title, subtitle }: &HeadingProps) -> Html {
    html! {
        <div class="my-2">
            <h3 class="text-xl">
                <strong>
                    {title}
                </strong>
            </h3>
            <p> {subtitle.as_ref().unwrap_or(&"".to_string())} </p>
        </div>
    }
}
