use rspim_lib::*;
use clap::Clap;

mod interactive;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Zac K. <zac.kologlu@gmail.com>")]
struct Opts {
    #[clap(long, short, about("Verbose output"))]
    verbose: bool,
    #[clap(long, about("Step-by-step execution"))]
    step: bool,
    #[clap(long, about("Just compile program instead of executing"))]
    compile: bool,
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
        interactive::launch();
    }

    let file_contents = std::fs::read_to_string(&opts.file.as_ref().unwrap()).expect("Could not read file {}");

    let iset       = rspim_lib::inst_set()?;
    let binary     = rspim_lib::compile(&iset, &file_contents)?;

    if opts.compile {
        let decompiled = rspim_lib::decompile(&iset, &binary);
        println!("Compiled program:\n{}\n", decompiled);
        return Ok(())
    }

    let mut runtime = rspim_lib::run(&binary)?;
    loop {
        runtime.step()?;
    }
}
