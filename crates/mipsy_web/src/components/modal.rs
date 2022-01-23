use git_version::git_version;
use yew::prelude::*;
use yew::Properties;

#[derive(Properties, Clone, PartialEq)]
pub struct ModalProps {
    pub should_display: UseStateHandle<bool>,
}

#[function_component(Modal)]
pub fn render_modal(props: &ModalProps) -> Html {
    let classes = if *props.should_display {
        "modal overflow-auto bg-th-primary border-black border-2 absolute top-28 w-3/4"
    } else {
        "modal hidden"
    };

    html! {
        <div class={classes} id="modal1" style="left: 13%;">
            <div class="modal-dialog">
                <div class="absolute modal-header top-0 right-0 h-16 w-16">
                    <div onclick={{
                            let display_modal = props.should_display.clone();
                            Callback::from(move |_| {
                            display_modal.set(!*display_modal);
                        })}} 
                    class="text-center cursor-pointer text-6xl border-black border-2 hover:bg-red-700 border-none bg-transparent close-modal" aria-label="close">
                    {"x"}
                    </div>
                </div>
                <section class="modal-content p-2 flex items-center flex-col">
                    <div>
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
                        <h2 class="mt-2"> <strong> {"Unimplemented Features"} </strong> </h2>
                        <p > {"Many features have yet to be implemented, including (but not limited to)"}</p>
                        <ul class="ml-4 list-disc">
                            <li>{"Runtime Errors"}</li>
                            <li>{"Separate text and data segments"}</li>
                            <li>{"Custom Settings + Theming"}</li>
                            <li>{"Highlighted Register Changes between steps"}</li>
                        </ul>

                        <h2 class="mt-4"> <strong> {"Unsupported Features"} </strong> </h2>
                        <p> {"The following features will not be supported in mipsy_web"}</p>
                        <ul>
                            <li>{"FileRead, Write and Open Syscalls"}</li>
                        </ul>
                        <div class="mt-4 text-xs">
                            <p> {"Made with love by Shrey Somaiya for cs1521 at UNSW CSE"} </p>
                            <p> {"with help from:"}</p>
                            <ul class="ml-4 list-disc">
                                <li>{"Zac Kologlu - partnering on development and major implementation decisions."}</li>
                                <li>{"Dylan Brotherson"}</li>
                                <li>{"Andrew Taylor"}</li>
                                <li>{"Jashank Jeremy"}</li>
                                <li>{"You, for testing this out!"}</li>
                            </ul>
                            <p class="mt-3 ml-4">
                                {"Hash: "}
                                {git_version!()}
                            </p>
                        </div>
                    </div>
                </section>
                <footer class="modal-footer"></footer>
            </div>
        </div>
    }
}
