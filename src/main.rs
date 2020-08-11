mod lexer;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <file>", &args[0]);
        return;
    }

    let file_name = &args[1];
    println!("Tokenising file: {}", file_name);

    let file_contents = std::fs::read_to_string(file_name).expect("Could not read file {}");
    let tokens = lexer::tokenise(&file_contents);

    println!("Tokens: {:?}", tokens);
}
