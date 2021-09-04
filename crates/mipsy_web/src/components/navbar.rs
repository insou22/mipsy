use yew::prelude::*;

pub struct NavBar {
    link: ComponentLink<Self>,
    pub props: Props,
}

#[derive(Properties, Clone)]
pub struct Props {
    #[prop_or_default]
    pub load_onchange: Callback<ChangeData>,
    pub run_onclick: Callback<MouseEvent>,
}

impl Component for NavBar {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        NavBar { link, props }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        // TODO - use hashmap of icon->svg and iter to render
        html! {
        <nav class="flex bg-red-100 items-center justify-between flex-wrap bg-teal-500 p-6">
          <div class="flex items-center flex-shrink-0 text-black mr-6">
            <span class="font-semibold text-xl tracking-tight">{"Mipsy"}</span>
          </div>
          <div class="w-full block flex-grow flex items-center w-auto">
            <div class="flex-grow flex flex-row">
              <label for="load_file" class="mr-2 text-sm flex place-items-center flex-row inline-block cursor-pointer px-3 py-3 leading-none border rounded text-black border-black hover:border-transparent hover:text-teal-500 hover:bg-white">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M2 6a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1H8a3 3 0 00-3 3v1.5a1.5 1.5 0 01-3 0V6z" clip-rule="evenodd" />
                  <path d="M6 12a2 2 0 012-2h8a2 2 0 012 2v2a2 2 0 01-2 2H2h2a2 2 0 002-2v-2z" />
                </svg>
                {"Load"}
              </label>
              <input id="load_file" onchange=&self.props.load_onchange type="file" accept=".s" class="hidden" />
              <button onclick=&self.props.run_onclick class="mr-2 flex place-items-center flex-row inline-block cursor-pointer text-sm px-2 py-2 border rounded text-black border-black hover:border-transparent hover:text-teal-500 hover:bg-white">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd" />
                </svg>
                {"Run"}
              </button>
              <button class="flex flex-row mr-2 place-items-center inline-block cursor-pointer text-sm px-2 py-2 border rounded text-black border-black hover:border-transparent hover:text-teal-500 hover:bg-white">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                </svg>
                {"Kill"}
              </button>
              <button class="flex flex-row mr-2 place-items-center inline-block cursor-pointer text-sm px-2 py-2 border rounded text-black border-black hover:border-transparent hover:text-teal-500 hover:bg-white">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                </svg>
                {"Step Next"}
              </button>
              <button class="flex mr-2 flex-row place-items-center inline-block cursor-pointer text-sm px-2 py-2 border rounded text-black border-black hover:border-transparent hover:text-teal-500 hover:bg-white">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                {"Step Back"}
              </button>
            </div>
            <div class="text-sm">
              <a href="#responsive-header" class="block mt-4 inline-block mt-0 text-teal-200 hover:text-white mr-4"> {"MIPS documentation"}</a>

            </div>
          </div>
        </nav>
              }
    }
}
