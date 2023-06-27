use super::*;

#[allow(unreachable_code)]
pub(crate) fn exit_command() -> Command {
    command(
        "exit",
        vec!["ex", "quit", "q"],
        vec![],
        vec![],
        vec![],
        "exit mipsy",
        |_, _state, label, _args| {
            if label == "__help__" {
                return Ok("Immediately exits mipsy".into());
            }

            std::process::exit(0)
        },
    )
}
