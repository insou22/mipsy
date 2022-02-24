use crate::pages::main::state::RunningState;
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
        <div id="output" class="min-w-full">
            <table style="width: 100%">
                {
                    for pages.into_iter().map(|(page_addr, page_contents)| {
                        let segment = get_segment(page_addr);

                        if !should_display_segment(segment) {
                            return html! {};
                        }

                        if curr_segment != segment {
                            curr_segment = segment;

                            html! {
                                <>
                                    { render_segment_header(segment) }
                                    { render_page(page_addr, page_contents) }
                                </>
                            }
                        } else {
                            render_page(page_addr, page_contents)
                        }
                    })
                }
            </table>
        </div>
    }
}

fn render_segment_header(segment: Segment) -> Html {
    html! {
        <tr style="border: 1px solid;">
            {
                match segment {
                    Segment::None  => html! {},
                    Segment::Text  => html! { <b>{ "Text segment" }</b> },
                    Segment::Data  => html! { <b>{ "Data segment" }</b> },
                    Segment::Stack => html! { <b>{ "Stack segment" }</b> },
                    Segment::KText => html! { <b>{ "Kernel text segment" }</b> },
                    Segment::KData => html! { <b>{ "Kernel data segment" }</b> },
                }
            }
        </tr>
    }
}

fn render_page(page_addr: u32, page_contents: Vec<Safe<u8>>) -> Html {
    const ROWS: usize = 4;
    const ROW_SIZE: usize = PAGE_SIZE / ROWS;

    html! {
        {
            for (0..ROWS).map(|nth| {
                html! {
                    <tr style="border: 1px solid;">
                        <td>{ format!("0x{:08x}", page_addr as usize + nth * ROW_SIZE) }</td>
                        {
                            for (0..ROW_SIZE).map(|offset| {
                                html! {
                                    <td>
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
                                    </td>
                                }
                            })
                        }

                        // I'm sure there is a nicer way of doing this...
                        <td><pre>{ "      " }</pre></td>

                        {
                            for (0..ROW_SIZE).map(|offset| {
                                let value = page_contents[nth * ROW_SIZE + offset].into_option();

                                html! {
                                    <td>
                                        {
                                            value
                                                .map(|value| value as u32)
                                                .and_then(char::from_u32)
                                                .filter(|&char| char.is_ascii_graphic() || char == ' ')
                                                .map(|value| html! { value })
                                                .unwrap_or(html! { "_" })
                                        }
                                    </td>
                                }
                            })
                        }
                    </tr>
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
