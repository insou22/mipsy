use yew::prelude::*;
use yew::Properties;
use crate::pages::main::state::RunningState;

#[derive(Properties, Clone, PartialEq)]
pub struct DataSegmentProps{
    pub state: RunningState
}



#[function_component(DataSegment)]
pub fn data_segment(props: &DataSegmentProps) -> Html {
    let pages = props.state.Runtime.state.pages;
    html! {
        <div id="output" class="min-w-full">
            {"Data fn"}
        </div>
    }
}
