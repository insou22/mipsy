use yew::prelude::*;
use yew::{Children, Properties};

#[derive(Properties, Clone)]
pub struct Props {
    #[prop_or_default]
    pub toggle_modal_onclick: Callback<MouseEvent>,
    pub should_display: bool,

}

pub struct Modal{
    pub props: Props,
}

impl Component for Modal {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Modal { props }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        let classes = if self.props.should_display {
            "modal bg-th-primary border-black border-2 absolute top-1/4 h-1/3 w-3/4"
        } else {
            "modal hidden"
        };

        html! {
            <div class={classes} id="modal1" style="left: 13%;">
                <div class="modal-dialog">
                    <div class="absolute modal-header top-0 right-0 h-16 w-16">
                        <div onclick={self.props.toggle_modal_onclick.clone()} class="cursor-pointer text-6xl border-black border-2 hover:bg-red-700 border-none bg-transparent close-modal" aria-label="close">
                        {"x"}
                        </div>
                    </div>
                    <section class="modal-content p-2 flex items-center flex-col">
                        <h1 class="my-2">
                        <strong>{"Welcome to Mipsy Web"}</strong>
                        </h1>
                        <br />
                        <p>
                            {"mipsy_web is a MIPS emulator built using the mipsy platform."}
                        </p>
                        <p>
                            {"mipsy_web, alongside the mipsy platform, is fully open source "}
                            <a class="hover:underline text-blue-600 hover:text-blue-800 visited:text-purple-600 hover:underline" target="_blank" href="https://github.com/insou22/mipsy/">{"here"}</a>
                        </p>
                        <br />
                        <p class="mt-2">
                            {"mipsy_web is pre-alpha software, and will eventually be a full replacement for QtSpim"}
                        </p>
                        <br />
                        <p class="mb-2">
                            {"Please leave any relevant feedback, issues or concerns on the "}
                            <a class="text-blue-600 hover:text-blue-800 visited:text-purple-600 hover:underline" href="https://github.com/insou22/mipsy/issues" target="_blank">
                                {"Github Issues page"}
                            </a>
                        </p>
                    </section>
                    <footer class="modal-footer"></footer>
                </div>
            </div>
        }
    }
}
