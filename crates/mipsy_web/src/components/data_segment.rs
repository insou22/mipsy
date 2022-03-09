use crate::state::state::RunningState;
use mipsy_lib::compile::TEXT_TOP;
use mipsy_lib::runtime::PAGE_SIZE;
use mipsy_lib::Safe;
use mipsy_lib::GLOBAL_BOT;
use mipsy_lib::KDATA_BOT;
use mipsy_lib::KTEXT_BOT;
use mipsy_lib::STACK_BOT;
use mipsy_lib::STACK_TOP;
use mipsy_lib::TEXT_BOT;
use yew::prelude::*;
use yew::Properties;

#[derive(Properties, Clone, PartialEq)]
pub struct DataSegmentProps {
    pub state: RunningState,
}

// TODO(shreys): Make this user-configurable
fn should_display_segment(segment: Segment) -> bool {
    match segment {
        Segment::None => false,
        Segment::Text => false,
        Segment::Data => true,
        Segment::Stack => true,
        Segment::KText => false,
        Segment::KData => false,
    }
}

// NOTE to any future adventurers wanting to change the layout of this component:
// This layout currently makes use of CSS grid crazy semantics
// It is modelled as a grid with 40 columns
// The first div spans the first column is the address at which we are inspecting
// The second div spans columns 3-22 (ie, 19 columns long) are the hex values of the memory
//    columns 7, 12, 17 are gaps between 4 byte chunks
// The third div spans columns 24-40 (ie, 16 columns long) are the ascii values of the memory

#[function_component(DataSegment)]
pub fn data_segment(props: &DataSegmentProps) -> Html {
    let mut pages = props
        .state
        .mips_state
        .memory
        .iter()
        .map(|(key, val)| (key.clone(), val.clone()))
        .collect::<Vec<_>>();

    pages.sort_by_key(|(key, _)| *key);

    let mut curr_segment = Segment::None;

    html! {
        <div id="output" style="min-width: 650px;margin-top: 10px;">
            <div style="width: 100%;">
                {
                    for pages.into_iter().map(|(page_addr, page_contents)| {
                        let segment = get_segment(page_addr);

                        if !should_display_segment(segment) {
                            return html! {};
                        }
                        html! {
                            <>
                                // if we need a header, render it
                                {
                                    if curr_segment != segment {
                                        curr_segment = segment;
                                        html! {
                                            <>
                                                { render_segment_header(segment) }
                                            </>
                                        }
                                    } else {
                                        html!{}
                                    }
                                }

                                // render the data
                                <div style="display: grid; width: 100%; grid-template-columns: repeat(40, [col-start] 1fr); font-size: 11.5px; font-family: monospace;">
                                    { render_page(page_addr, page_contents) }
                                </div>
                            </>
                        }
                        
                    })
                }
            </div>
        </div>
    }
}

fn render_segment_header(segment: Segment) -> Html {
    html! {
        <h4>
            {
                match segment {
                    Segment::None  => {""} 
                    Segment::Text  => {"Text segment"},
                    Segment::Data  => {"Data segment"},
                    Segment::Stack => {"Stack segment"},
                    Segment::KText => {"Kernel text segment"},
                    Segment::KData => {"Kernel data segment"},
                }
            }
        </h4>
    }
}

fn render_page(page_addr: u32, page_contents: Vec<Safe<u8>>) -> Html {
    const ROWS: usize = 4;
    const ROW_SIZE: usize = PAGE_SIZE / ROWS;

    html! {
        {
            for (0..ROWS).map(|nth| {
                html! {
                    <>
                        
                        <div style="grid-column: col-start / span 1">
                            { format!("0x{:08x}", page_addr as usize + nth * ROW_SIZE) }
                        </div>
                        <div style="grid-column: col-start 3 / span 19; display: grid;  grid-template-columns: repeat(19, 1fr)">
                        {
                            for (0..ROW_SIZE).enumerate().map(|(i, offset)| {
                                html! {
                                    <>
                                        // add an extra column to gap between 4 bytes
                                        if i > 0 && i % 4 == 0 {
                                            <pre>{"  "}</pre>
                                        }
                                        <div style="text-align: center;">
                                            {
                                                match page_contents[nth * ROW_SIZE + offset] {
                                                    Safe::Valid(byte) => {
                                                        html! { format!("{:02x}", byte) }
                                                    }
                                                    Safe::Uninitialised => {
                                                        html! { "__" }
                                                    }
                                                }
                                            }
                                        </div>
                                    </>
                                }
                            })
                        }
                        </div>

                        <div style="grid-column: col-start 25 / span 16; display: grid; grid-template-columns: repeat(16, 1fr);">
                        {
                            for (0..ROW_SIZE).map(|offset| {
                                let value = page_contents[nth * ROW_SIZE + offset].into_option();

                                    
                                html! {
                                    <div style="text-align: center;">
                                        {
                                            value
                                                .map(|value| value as u32)
                                                .and_then(char::from_u32)
                                                .filter(|&char| char.is_ascii_graphic() || char == ' ')
                                                .map(|value| html! { value })
                                                .unwrap_or(html! { "_" })
                                        }
                                    </div>
                                }
                            })
                        }
                        </div>
                    </>
                }
            })
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Segment {
    None,
    Text,
    Data,
    Stack,
    KText,
    KData,
}

fn get_segment(address: u32) -> Segment {
    match address {
        // TODO(zkol): Update this when exclusive range matching is stabilised
        _ if address < TEXT_BOT => Segment::None,
        _ if address >= TEXT_BOT && address <= TEXT_TOP => Segment::Text,
        _ if address >= GLOBAL_BOT && address < STACK_BOT => Segment::Data,
        _ if address >= STACK_BOT && address <= STACK_TOP => Segment::Stack,
        _ if address >= KTEXT_BOT && address < KDATA_BOT => Segment::KText,
        _ if address >= KDATA_BOT => Segment::KData,
        _ => unreachable!(),
    }
}
