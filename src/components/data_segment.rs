use crate::state::state::RunningState;
use mipsy_lib::runtime::PAGE_SIZE;
use mipsy_lib::util::{get_segment, Segment};
use mipsy_lib::Register;
use mipsy_lib::Safe;
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

pub const SP_COLOR: &str = "green";
pub const FP_COLOR: &str = "red";
pub const SP_FP_COLOR: &str = "blue";
pub const GP_COLOR: &str = "transparent";

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

    let registers = Some(props.state.mips_state.clone())
        .clone()
        .map(|state| state.register_values.clone())
        .unwrap_or_else(|| vec![Safe::Uninitialised; 32]);

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
                                    { render_page(&props, page_addr, page_contents, &registers) }
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

trait Escape {
    fn escape(&self) -> String;
}

impl Escape for char {
    fn escape(self: &char) -> String {
        match self {
            '\0' => r"\0".to_string(),           // null
            '\t' => r"\t".to_string(),           // tab
            '\r' => r"\r".to_string(),           // carriage return
            '\n' => r"\n".to_string(),           // newline
            '\x07' => r"\a".to_string(),         // bell
            '\x08' => r"\b".to_string(),         // backspace
            '\x0B' => r"\v".to_string(),         // vertical tab
            '\x0C' => r"\f".to_string(),         // form feed
            '\x1B' => r"\e".to_string(),         // escape
            '\x20'..='\x7E' => self.to_string(), // printable ASCII
            _ => ".".to_string(),                // everything else
        }
    }
}

fn render_page(
    props: &DataSegmentProps,
    page_addr: u32,
    page_contents: Vec<Safe<u8>>,
    registers: &[Safe<i32>],
) -> Html {
    const ROWS: usize = 4;
    const ROW_SIZE: usize = PAGE_SIZE / ROWS;
    const SP: usize = Register::Sp.to_number() as usize;
    const FP: usize = Register::Fp.to_number() as usize;
    const GP: usize = Register::Gp.to_number() as usize;

    html! {
        for (0..ROWS).map(|nth| {
            html! {
                <>
                <div style="grid-column: col-start / span 1">
                    { format!("0x{:08x}", page_addr as usize + nth * ROW_SIZE) }
                </div>
                <div style="grid-column: col-start 3 / span 19; display: grid;  grid-template-columns: repeat(19, 1fr)">
                {
                    for (0..ROW_SIZE).enumerate().map(|(i, offset)| {
                        let full_offset = nth * ROW_SIZE + offset;
                        let full_page_addr = page_addr as usize + full_offset;
                        let class = "cursor-help";
                        let mut style = String::new();
                        let mut title = String::new();

                        if let Some((label, _)) = props.state.mips_state.binary.clone().as_ref().unwrap()
                            .labels
                            .iter()
                            .find(|(_, &addr)| addr == full_page_addr as u32)
                        {
                            title.push_str(&format!("{}:\n", label));
                        }

                        title.push_str(&format!("0x{:08X}", full_page_addr));

                        if full_page_addr == registers[SP].into_option().unwrap_or(0) as usize &&
                            full_page_addr == registers[FP].into_option().unwrap_or(0) as usize {
                                style.push_str(&format!("border: 2px solid {};", SP_FP_COLOR));
                                title.push_str("\n$sp");
                                title.push_str("\n$fp");
                        }
                        else if full_page_addr == registers[SP].into_option().unwrap_or(0) as usize {
                            style.push_str(&format!("border: 2px solid {};", SP_COLOR));
                            title.push_str("\n$sp");
                        }
                        else if full_page_addr == registers[FP].into_option().unwrap_or(0) as usize {
                            style.push_str(&format!("border: 2px solid {};", FP_COLOR));
                            title.push_str("\n$fp");
                        }
                        else if full_page_addr == registers[GP].into_option().unwrap_or(0) as usize {
                            style.push_str(&format!("border: 2px solid {};", GP_COLOR));
                            title.push_str("\n$gp");
                        }
                        else {
                            style.push_str("border: 2px solid transparent;");
                        }

                        html! {
                            <>
                                // add an extra column to gap between 4 bytes
                                if i > 0 && i % 4 == 0 {
                                    <div>{""}</div>
                                }
                                <div style="text-align: center;">
                                    {
                                        html! {
                                            <span class={class} style={style} title={title}>
                                            {
                                                    render_data(page_contents[full_offset])
                                            }
                                            </span>
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
                                        .map(|c| c.escape())
                                        .filter(|char| char.len() == 2 || (char.len() == 1 && char.as_bytes()[0].is_ascii_graphic()) || char == " ")
                                        .map(|value| html! { value })
                                        .unwrap_or_else(|| html! { "_" })
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

fn render_data(data_val: Safe<u8>) -> Html {
    match data_val {
        Safe::Valid(byte) => {
            html! { format!("{:02x}", byte) }
        }
        Safe::Uninitialised => {
            html! { "__" }
        }
    }
}
