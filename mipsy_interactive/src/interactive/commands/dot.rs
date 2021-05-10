use std::rc::Rc;

use mipsy_lib::{MPProgram, compile};
use mipsy_parser::MPItem;

use crate::interactive::error::CommandError;

use super::*;

pub(crate) fn dot_command() -> Command {
    command_varargs(
        ".",
        vec![],
        vec!["instruction"],
        "execute a MIPS instruction",
        "Executes a MIPS instruction immediately",
        |state, _label, args| {
            let line = args.join(" ");

            let inst = mipsy_parser::parse_instruction(&line)
                    .map_err(|error| CommandError::CannotParseLine { line: line.to_string(), error })?;
            
            let program = MPProgram::new(
                vec![(MPItem::Instruction(inst.clone()), None, 1)]
            );

            compile::check_pre(&program)
                    .map_err(|error| CommandError::CannotCompileLine { line: line.to_string(), error })?;

            let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

            compile::check_post_data_label(&program, binary)
                    .map_err(|error| CommandError::CannotCompileLine { line: line.to_string(), error })?;

            let opcodes = mipsy_lib::compile1(binary, &state.iset, &inst)
                    .map_err(|error| error.to_compiler_mipsy_error(Rc::from(""), 1, inst.col(), inst.col_end()))
                    .map_err(|error| CommandError::CannotCompileLine { line: line.to_string(), error })?;

            for opcode in opcodes {
                state.exec_inst(opcode, true)
                    .map_err(|err| {
                        let mipsy_error = match err {
                            CommandError::RuntimeError { mipsy_error } => mipsy_error,
                            _ => unreachable!(),
                        };

                        CommandError::REPLRuntimeError { mipsy_error, line: line.to_string() }
                    })?;
            }

            Ok(())
        }
    )
}
