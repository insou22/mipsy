use yew::prelude::*;
use yew::{Children, Properties};

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

pub struct PageBackground {
}

impl Component for PageBackground {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        PageBackground { }
    }


    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="min-h-screen py-2 bg-th-primary">
                { for ctx.props().children.iter() }
            </div>
        }
    }
}
