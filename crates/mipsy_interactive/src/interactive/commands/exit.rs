use super::*;

#[allow(unreachable_code)]
pub(crate) fn exit_command() -> Command {
    command(
        "exit",
        vec!["ex", "quit", "q"],
        vec![],
        vec![],
        "exit mipsy",
        |_state, label, _args| {
            if label == "_help" {
                return Ok(
                        "Immediately exits mipsy".into()
                )
            }

            std::process::exit(0)
        }
    )
}
