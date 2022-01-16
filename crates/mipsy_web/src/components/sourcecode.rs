use log::info;
use yew::{function_component, html, Properties};

#[derive(Properties, PartialEq)]
pub struct SourceCodeProps {
    pub file: Option<String>,
}

#[function_component(SourceCode)]
pub fn render_source_code(props: &SourceCodeProps) -> Html {
    info!("calling render source table");
    let file_len = props
        .file
        .as_ref()
        .unwrap_or(&"".to_string())
        .len()
        .to_string()
        .len();
    info!("{}", file_len);

    html! {
        // if we ever want to do specific things on specific lines...
        {
            for props.file.as_ref().unwrap_or(&"".to_string()).as_str().split("\n").into_iter().enumerate().map(|(index, item)| {
                if item == "" {
                    // this is &nbsp;
                    html! {
                        <tr>
                            <pre>
                                <code class="language-mips" style="padding: 0 !important;">
                                {format!("{:indent$} ",index, indent=file_len)}
                                {"\u{00a0}"}
                                </code>
                            </pre>
                        </tr>
                    }
                }
                else {
                    html! {
                        <tr>
                            <pre>
                                <code class="language-mips" style="padding: 0 !important;">
                                    {format!("{:indent$} ",index, indent=file_len)}
                                    {item}
                                </code>
                            </pre>
                        </tr>
                    }
                }
            }
            )
        }
    }
}
