use rspim_lib::*;
use error::RSpimResult;
use std::io::{stdin, Read};
use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Zac K. <zac.kologlu@gmail.com>")]
struct Opts {
    #[clap(long, about("Step-by-step compilation and runtime"))]
    steps: bool,
    #[clap(long("spim-compare"), about("Print exceptions line for diff to SPIM"))]
    spim_compare: bool,
    file: Option<String>,
}

impl Opts {
    fn print_step_info(&self, print: &str) {
        if self.steps {
            println!("{}", print);
        }
    }

    fn step(&self, print: &str) {
        if self.steps {
            println!("{}", print);

            stdin().read(&mut [0]).unwrap();
        }
    }
}



fn main() -> RSpimResult<()> {
    let opts: Opts = Opts::parse();

    if let None = opts.file {
        return Ok(());
    }

    let file_contents = std::fs::read_to_string(&opts.file.as_ref().unwrap()).expect("Could not read file {}");

    let yaml = yaml::parse::get_instructions();
    opts.step(&format!("Parsed mips.yaml: \n\n{:#x?}\n\n", yaml));

    let iset = inst::instruction::InstSet::new(&yaml)?;
    opts.step(&format!("Loaded instruction set: \n\n{:#x?}\n\n", iset));

    let tokens = compile::lexer::tokenise(&file_contents)?;
    opts.step(&format!("Lexed {} into tokens: \n\n{:x?}\n\n", &opts.file.as_ref().unwrap(), tokens));

    let program = compile::compiler::generate(tokens, &iset)?;
    opts.step(&format!("Successfully generated program: \n\n{:#010x?}\n\n", program));

    let decompiled = decompile::decompile(&program, &iset);
    opts.step(&format!("Successfully compiled program: \n\n{}\n\n", decompiled));

    opts.print_step_info("Labels: ");
    for (label, addr) in &program.labels {
        opts.print_step_info(&format!("    {:9} => 0x{:08x}", label, addr));
    }
    opts.step(&format!("\n"));

    let mut runtime = runtime::Runtime::new(&program);
    opts.step(&format!("Loaded runtime: {:}", runtime.state()));

    if opts.spim_compare {
        println!("Loaded: /home/zac/uni/teach/comp1521/20T2/work/spim-simulator/CPU/exceptions.s");
    }

    loop {
        match runtime.step() {
            Ok(_) => {},
            Err(error::RSpimError::Runtime(error::runtime_error::RuntimeError::UninitializedRegister(31))) => { break; }
            Err(e) => {
                println!("Error: {:x?}", e);
                let timeline_len = runtime.timeline_len();
                println!("Timeline length: {}", timeline_len);

                for i in (1..=5).rev() {
                    if (timeline_len as isize - i) < 0 {
                        continue;
                    }

                    println!("{}", runtime.nth_state(timeline_len - i as usize).unwrap());
                }

                break;
            }
        }
        // println!("Current state: {:}", runtime.state());
        // pause();
    }

    Ok(())
}
