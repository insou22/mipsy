use colored::*;
use std::fmt::Display;

pub fn unknown_command<D: Display>(command: D) {
    error(format!("{} `{}`", "unknown command", command));
    tip_nl(format!(
        "{}{}{}",
        "use `",
        "help".bold(),
        "` for a list of commands"
    ));
}

pub fn banner<D: Display>(text: D) {
    print!("{}{}", text, ": ".bold());
}

pub fn banner_nl<D: Display>(text: D) {
    banner(text);
    println!();
}

pub fn ebanner<D: Display>(text: D) {
    eprint!("{}{}", text, ": ".bold());
}

pub fn tip<D: Display>(text: D) {
    banner("tip".yellow().bold());
    println!("{}", text);
}

pub fn tip_nl<D: Display>(text: D) {
    tip(text);
    println!();
}

pub fn success<D: Display>(text: D) {
    banner("success".green().bold());
    println!("{}", text);
}

pub fn success_nl<D: Display>(text: D) {
    success(text);
    println!();
}

pub fn error_nonl<D: Display>(text: D) {
    ebanner("error".bright_red().bold());
    eprint!("{}", text);
}

pub fn error<D: Display>(text: D) {
    ebanner("error".bright_red().bold());
    eprintln!("{}", text);
}

pub fn error_nl<D: Display>(text: D) {
    error(text);
    println!();
}

pub fn warning<D: Display>(text: D) {
    ebanner("warning".yellow().bold());
    eprintln!("{}", text);
}

pub fn warning_nl<D: Display>(text: D) {
    warning(text);
    println!();
}

pub fn syscall<D: Display>(code: i32, text: D) {
    print!(
        "{}{}{}{}",
        "\n[SYSCALL ".yellow().bold(),
        code.to_string().bold(),
        "] ".yellow().bold(),
        text
    );
}

pub fn syscall_nl<D: Display>(code: i32, text: D) {
    syscall(code, text);
    println!("\n");
}
