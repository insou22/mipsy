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

            let inst = mipsy_parser::parse_instruction(&line)
                    .map_err(|err| CommandError::CannotParseLine { line: line.to_string(), col: err.col })?;

            let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

            let opcodes = mipsy_lib::compile1(binary, &state.iset, &inst)
                    .map_err(|mipsy_error| CommandError::CannotCompileLine { line, mipsy_error })?;

            for opcode in opcodes {
                state.exec_inst(opcode, true)?;
            }

            Ok(())
        }
    )
}
