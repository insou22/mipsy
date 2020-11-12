use crate::interactive::error::CommandError;

use super::*;

pub(crate) fn dot_command() -> Command {
    command_varargs(
        ".",
        vec![],
        vec!["instruction"],
        "execute a MIPS instruction",
        &format!(
            "Executes a MIPS instruction immediately"
        ),
        |state, _label, args| {
            let line = args.join(" ");

            let inst = mipsy_parser::parse_instruction(line.as_bytes())
                    .map(|(_, inst)| inst)
                    .map_err(|_err| CommandError::CannotParseLine { line: line.to_string() })?;

            let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

            let opcodes = mipsy_lib::compile1(binary, &state.iset, &inst)
                    .map_err(|mipsy_error| CommandError::RuntimeError { mipsy_error })?;

            for opcode in opcodes {
                state.exec_inst(opcode, true)?;
            }

            Ok(())
        }
    )
}
