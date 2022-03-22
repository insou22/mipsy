use yew::prelude::*;
use yew::Properties;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub show_analytics_banner: UseStateHandle<bool>,
}

#[function_component(Banner)]
pub fn render_banner(props: &Props) -> Html {
    html! {

        <div class="absolute bottom-0 w-screen left-0 z-10 bg-blue-100/75 border-t border-b border-blue-500 px-4 py-6 flex items-center justify-center flex-row" role="alert">
            <div>
                {" This app collects anonymous analytics to improve your experience. See settings for information on what is collected and how it is used"}
                <button 
                    class="bg-transparent hover:bg-blue-500 text-blue-700 font-semibold hover:text-white mx-3 py-2 px-4 border border-blue-500 hover:border-transparent rounded" 
                    onclick={
                        let show_analytics_banner = props.show_analytics_banner.clone();
                        move |_| {
                            show_analytics_banner.set(false);
                            crate::set_localstorage("analytics_ack", "true");
                        }
                }>
                    {"Ok"}
                </button>
            </div>
        </div>
    }
}
