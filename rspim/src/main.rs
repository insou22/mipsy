use rspim_lib::*;
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

    let iset       = rspim_lib::inst_set()?;
    let binary     = rspim_lib::compile(&iset, &file_contents)?;
    let decompiled = rspim_lib::decompile(&iset, &binary);

    println!("Decompiled program:\n{}\n\n", decompiled);

    let mut runtime = rspim_lib::run(&binary);
    println!("Running program:\n");
    loop {
        runtime.step()?;
    }

    Ok(())
}
