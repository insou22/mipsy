use yew::prelude::*;
use yew::{Properties, Children};


#[derive(Properties, Clone)]
pub struct Props {    
    #[prop_or_default]    
    pub children: Children,
}

pub struct PageBackground {
    pub props: Props,
}

impl Component for PageBackground {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        PageBackground {
            props,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props; 
        true
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="min-h-screen bg-gray-300">
                { for self.props.children.iter() }
            </div>
        }
    }
}
