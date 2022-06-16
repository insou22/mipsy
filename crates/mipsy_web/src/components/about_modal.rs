use git_version::git_version;
use yew::{prelude::*, Properties};

#[derive(Properties, Clone, PartialEq)]
pub struct ModalProps {
    pub should_display: UseStateHandle<bool>,
}

#[function_component(Modal)]
pub fn render_modal(props: &ModalProps) -> Html {
    let classes = if *props.should_display {
        "modal overflow-auto bg-th-primary border-black border-2 absolute top-28 w-3/4 z-20"
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
                            {"mipsy web is an online MIPS emulator and debugger that runs in the browser, built using the mipsy platform."}
                        </p>
                        <br />
                        <p>
                            {"mipsy web allows you load existing files, and to edit them in the microsoft Monaco editor (the same editor as vscode)"}
                        </p>
                        <p>
                            {"Once you have loaded a file, you can step through, run, step back, and view the decompiled, and data sections"}
                        </p>
                        <p>
                            {"You are able to view registers, IO, and any error output from the mipsy platform"}
                        </p>
                        <p>
                            {"Control-s will save and recompile your file, if you have edited it"}
                        </p>

                        <br />
                        <p class="mt-2">
                            {"mipsy web is beta software, and will eventually be a full replacement for QtSpim"}
                        </p>

                        <p>
                            {"mipsy web, alongside the mipsy platform, is fully open source "}
                            <a class="hover:underline text-blue-600 hover:text-blue-800 visited:text-purple-600 hover:underline" target="_blank" href="https://github.com/insou22/mipsy/">{"here"}</a>
                        </p>
                        <p class="mb-2">
                            {"Please leave any relevant feedback, issues or concerns on the "}
                            <a class="text-blue-600 hover:text-blue-800 visited:text-purple-600 hover:underline" href="https://github.com/insou22/mipsy/issues" target="_blank">
                                {"Github Issues page"}
                            </a>
                        </p>
                        <p class="mb-2">
                            {"You can also check the current and future items being worked on at "}
                            <a class="text-blue-600 hover:text-blue-800 visited:text-purple-600 hover:underline" href="https://github.com/insou22/mipsy/projects/2" target="_blank">
                                {"this project board"}
                            </a>

                        </p>

                        <h4 class="mt-2"> <strong> {"Unsupported Features"} </strong> </h4>
                        <p> {"The following features will not be supported in mipsy web"}</p>
                        <ul>
                            <li>{"FileRead, Write and Open Syscalls"}</li>
                        </ul>
                        <div class="mt-4 text-xs">
                            <p> {"Made with love by Shrey Somaiya for cs1521 at the School of Computer Science and Engineering, University of New South Wales, Sydney."} </p>
                            <p> {"with help from:"}</p>
                            <ul class="ml-4 list-disc">
                                <li>{"Zac Kologlu"}</li>
                                <li>{"Dylan Brotherson"}</li>
                                <li>{"Abiram Nadarajah"}</li>
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
