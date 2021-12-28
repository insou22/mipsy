use yew::prelude::*;
use yew::Properties;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub is_disabled: bool,
    pub show_io_tab: Callback<MouseEvent>,
    pub show_mipsy_tab: Callback<MouseEvent>,
    pub show_io: bool,
    pub mipsy_output_tab_title: String,
    pub input_ref: NodeRef,
    pub on_input_keydown: Callback<KeyboardEvent>,
    pub running_output: Html,
    pub input_maxlength: String,
    pub input_needed: bool,
    //TODO - figure out best abstrtaction for pub render_running_output: 

}


pub struct OutputArea {
}

impl Component for OutputArea {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        OutputArea { }
    }
    // this component has no messaging
    fn update(&mut self, _ctx: &Context<Self>, _: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>,) -> Html {

        let (mipsy_tab_button_classes, io_tab_classes) = {
				    let mut default = (
						    String::from("w-1/2 hover:bg-white float-left border-t-2 border-r-2 border-black cursor-pointer px-1 py-2"),
						    String::from("w-1/2 hover:bg-white float-left border-t-2 border-r-2 border-l-2 border-black cursor-pointer px-1 py-2")
					  );
					
					  if ctx.props().show_io {
						    default.1	= format!("{} {}", &default.1, String::from("bg-th-tabclicked"));
 					
					  } else {
						    default.0	= format!("{} {}", &default.0, String::from("bg-th-tabclicked"));
					  };
	
					  default	
				};

        //FIXME - double chekc this behaves as expected
        let input_classes = if !ctx.props().input_needed {
            if ctx.props().show_io {
                    "block w-full cursor-not-allowed"
                } else {
                    "hidden"
                }
            } else {
                "block w-full bg-th-highlighting"
        };


        html! {
            <div id="output" class="min-w-full">                    
                <div style="height: 10%;" class="flex overflow-hidden border-1 border-black">
                    <button class={io_tab_classes} onclick={ctx.props().show_io_tab.clone()}>{"I/O"}</button>
                    <button
                        class={mipsy_tab_button_classes} 
                        onclick={ctx.props().show_mipsy_tab.clone()}
                    >
                        {ctx.props().mipsy_output_tab_title.clone()}
                    </button>
                </div>
                <div 
                    style={if ctx.props().show_io {"height: 80%;"} else {"height: 90%;"}} 
                    class="py-2 w-full flex overflow-y-auto flex-wrap-reverse bg-th-secondary px-2 border-2 border-gray-600"
                >
                    <div class="w-full overflow-y-auto">
                    <h1> 
                        <strong> 
                            {if ctx.props().show_io {"Output"} else {"Mipsy Output"}}
                        </strong>
                    </h1>
                    <pre style="width:100%;" class="text-sm whitespace-pre-wrap">
                        {ctx.props().running_output.clone()}
                    </pre>
                    </div>
                </div>
                <div style="height: 10%;" class={if ctx.props().show_io {"border-l-2 border-r-2 border-b-2 border-black"} else {"hidden"}}>
                    <input
                        ref={ctx.props().input_ref.clone()}
                        id="user_input"
                        type="text"
                        maxlength={ctx.props().input_maxlength.clone()}
                        disabled={ctx.props().is_disabled}
                        onkeydown={ctx.props().on_input_keydown.clone()}
                        style="padding-left: 3px; height: 100%;"
                        class={input_classes} placeholder="> ..."/>
                </div>
            </div>
        }
    }
}
