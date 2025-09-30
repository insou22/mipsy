use super::*;

#[allow(unreachable_code)]
pub(crate) fn command() -> Command {
    Command::new()
        .with_name("exit")
        .with_name("ex")
        .with_name("quit")
        .with_name("q")
        .with_desc("exit mipsy")
        .with_exec(|_, _, _, label, _| {
            if label == "__help__" {
                Ok("Immediately exits mipsy".into())
            } else {
                std::process::exit(0)
            }
        })
}
