use rspim_lib::*;
use std::io::{stdin, Read};
use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Zac K. <zac.kologlu@gmail.com>")]
struct Opts {
    #[clap(long, short, about("Verbose output"))]
    verbose: bool,
    #[clap(long("spim-compare"), about("Print exceptions line for diff to SPIM"))]
    spim_compare: bool,
    #[clap(long, about("Step-by-step execution"))]
    step: bool,
    file: Option<String>,
}

macro_rules! vprintln {
    ($opts:expr, $($arg:tt)*) => ({
        if ($opts).verbose {
            println!($($arg)*);
        }
    });
}



fn main() -> RSpimResult<()> {
    let opts: Opts = Opts::parse();

    if let None = opts.file {
        return Ok(());
    }

    let file_contents = std::fs::read_to_string(&opts.file.as_ref().unwrap()).expect("Could not read file {}");

    if opts.spim_compare {
        println!("Loaded: /home/zac/uni/teach/comp1521/20T2/work/spim-simulator/CPU/exceptions.s")
    }

    let iset       = rspim_lib::inst_set()?;
    let binary     = rspim_lib::compile(&iset, &file_contents)?;
    let decompiled = rspim_lib::decompile(&iset, &binary);

    vprintln!(opts, "Decompiled program:\n{}\n", decompiled);

    let mut runtime = rspim_lib::run(&binary);
    vprintln!(opts, "Running program:\n");
    loop {
        runtime.step()?;
        if opts.step {
            println!("State: {}", runtime.state());
            stdin().read(&mut [0]).unwrap();
        }
    }

    Ok(())
}
