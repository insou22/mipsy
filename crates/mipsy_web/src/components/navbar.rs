use yew::prelude::*;

pub struct NavBar {
    link: ComponentLink<Self>,
    pub props: Props,
}


#[derive(Properties, Clone)]
pub struct Props {    
    #[prop_or_default]    
    pub onchange: Callback<ChangeData>,
}

impl Component for NavBar {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        NavBar {
            link,
            props
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
    
    fn view(&self) -> Html {
        
        html! {
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
                        <label for="load_file" class="inline-block cursor-pointer text-sm px-5 py-2 leading-none border rounded text-black border-black hover:border-transparent hover:text-teal-500 hover:bg-white mt-4 lg:mt-0">
                            {"Load"}
                        </label>
                        <input id="load_file" onchange=&self.props.onchange type="file" accept=".s" class="hidden" />
                    </div>
                </div>
            </nav>
        }  
    }
}
