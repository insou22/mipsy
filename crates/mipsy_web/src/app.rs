use yew::prelude::*;

pub enum Msg {}

pub struct App {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
                <div class="min-h-screen bg-gray-300">
                    <nav class="flex bg-red-100 items-center justify-between flex-wrap bg-teal-500 p-6">
                        <div class="flex items-center flex-shrink-0 text-black mr-6">
                            <span class="font-semibold text-xl tracking-tight">{"Mipsy"}</span>
                        </div>
                        <div class="w-full block flex-grow lg:flex lg:items-center lg:w-auto">
                            <div class="text-sm lg:flex-grow">
                                <a href="#responsive-header" class="block mt-4 lg:inline-block lg:mt-0 text-teal-200 hover:text-white mr-4"> {"Docs"} </a>
                                <a href="#responsive-header" class="block mt-4 lg:inline-block lg:mt-0 text-teal-200 hover:text-white mr-4"> {"Examples"}</a>
                                <a href="#responsive-header" class="block mt-4 lg:inline-block lg:mt-0 text-teal-200 hover:text-white"> {"Blog"} </a>
                            </div>
                           <div>
                                <button href="#" class="inline-block text-sm px-4 py-2 leading-none border rounded text-black border-black hover:border-transparent hover:text-teal-500 hover:bg-white mt-4 lg:mt-0">{"Load"}</button>
                            </div>
                        </div>
                    </nav>
                </div>
            }
    }
}
