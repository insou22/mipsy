pub mod error;
pub mod inst;
pub mod yaml;
pub mod util;
pub mod compile;
pub mod decompile;
pub mod runtime;

use error::RSpimResult;

fn main() -> RSpimResult<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <file>", &args[0]);
        return Ok(());
    }

    let file_name = &args[1];
    let file_contents = std::fs::read_to_string(file_name).expect("Could not read file {}");


    let yaml = yaml::parse::get_instructions();
    println!("Parsed mips.yaml: \n\n{:#x?}\n\n", yaml);

    let iset = inst::instruction::InstSet::new(&yaml)?;
    println!("Loaded instruction set: \n\n{:#x?}\n\n", iset);

    let tokens = compile::lexer::tokenise(&file_contents)?;
    println!("Lexed {} into tokens: \n\n{:x?}\n\n", file_name, tokens);

    let program = compile::compiler::generate(tokens, &iset)?;
    println!("Successfully generated program: \n\n{:#010x?}\n\n", program);

    let decompiled = decompile::decompile(&program, &iset);
    println!("Successfully decompiled program: \n\n{}\n\n", decompiled);

    Ok(())
}

