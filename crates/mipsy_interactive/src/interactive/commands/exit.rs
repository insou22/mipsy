use super::*;

#[allow(unreachable_code)]
pub(crate) fn command() -> Command {
    Command::new()
        .with_name("exit")
        .with_name("ex")
        .with_name("quit")
        .with_name("q")
        .with_desc("exit mipsy")
        .with_help("Immediately exits mipsy".to_owned())
        .with_exec(|_, _, _, _| std::process::exit(0))
}
