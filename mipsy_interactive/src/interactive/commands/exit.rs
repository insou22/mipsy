use super::*;

#[allow(unreachable_code)]
pub(crate) fn exit_command() -> Command {
    command(
        "exit",
        vec!["ex", "quit", "q"],
        vec![],
        vec![],
        "exit mipsy",
        "Immediately exits mipsy",
        |_state, _label, _args| {
            std::process::exit(0)
        }
    )
}
