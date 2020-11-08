use std::fmt::Display;
use colored::*;

pub(crate) fn unknown_command<D: Display>(command: D) {
    error(format!("{} `{}`", "unknown command", command));
    tip_nl(format!("{}{}{}", "use `", "help".bold(), "` for a list of commands"));
}

pub(crate) fn banner<D: Display>(text: D) {
    print!("{}{}", text, ": ".bold());
}

pub(crate) fn banner_nl<D: Display>(text: D) {
    banner(text);
    println!();
}

pub(crate) fn ebanner<D: Display>(text: D) {
    eprint!("{}{}", text, ": ".bold());
}

pub(crate) fn tip<D: Display>(text: D) {
    banner("tip".yellow().bold());
    println!("{}", text);
}

pub(crate) fn tip_nl<D: Display>(text: D) {
    tip(text);
    println!();
}

pub(crate) fn success<D: Display>(text: D) {
    banner("success".green().bold());
    println!("{}", text);
}

pub(crate) fn success_nl<D: Display>(text: D) {
    success(text);
    println!();
}

pub(crate) fn error_nonl<D: Display>(text: D) {
    ebanner("error".red());
    eprint!("{}", text);
}

pub(crate) fn error<D: Display>(text: D) {
    ebanner("error".red());
    eprintln!("{}", text);
}

pub(crate) fn error_nl<D: Display>(text: D) {
    error(text);
    println!();
}

pub(crate) fn syscall<D: Display>(code: i32, text: D) {
    print!("{}{}{}{}", "\n[SYSCALL ".yellow().bold(), code.to_string().bold(), "] ".yellow().bold(), text);
}

pub(crate) fn syscall_nl<D: Display>(code: i32, text: D) {
    syscall(code, text);
    println!("\n");
}
