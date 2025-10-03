use colored::Colorize;
use mipsy_lib::Binary;
use std::rc::Rc;

use mipsy_lib::{compile, MpProgram};
use mipsy_parser::{parser::MpAttributedItem, MpItem};

use crate::interactive::{commands::watchpoint::args_text, error::CommandError};

use super::*;

pub(crate) fn command() -> Command {
    Command::new()
        .with_name(".")
        .with_desc("execute a MIPS instruction")
        .with_var_args()
        // TODO: context for hints and sanitisation
        .with_required_arg(Argument::new(
            "instruction",
            |_, a, _| Ok(ArgumentKind::String(a.to_owned())),
            |_, _, _| vec![],
        ))
        .with_help("Executes a MIPS instruction immediately".to_owned())
        .with_varargs_format("{args}".magenta().to_string())
        .with_exec(|_, state, helper, args| {
            let line = args_text(args).join(" ");

            let inst =
                mipsy_parser::parse_instruction(&line, state.config.tab_size).map_err(|error| {
                    CommandError::CannotParseLine {
                        line: line.to_string(),
                        error,
                    }
                })?;

            let program = MpProgram::new(
                vec![MpAttributedItem::new(
                    MpItem::Instruction(inst.clone()),
                    vec![],
                    None,
                    1,
                )],
                vec![],
            );

            compile::check_pre(&program).map_err(|error| CommandError::CannotCompileLine {
                line: line.to_string(),
                error,
            })?;

            let empty_binary = Binary::default();
            let binary = state.binary.as_ref().unwrap_or(&empty_binary);

            compile::check_post_data_label(&program, binary).map_err(|error| {
                CommandError::CannotCompileLine {
                    line: line.to_string(),
                    error,
                }
            })?;

            let opcodes = mipsy_lib::compile1(binary, &state.iset, &inst)
                .map_err(|error| {
                    error.into_compiler_mipsy_error(Rc::from(""), 1, inst.col(), inst.col_end())
                })
                .map_err(|error| CommandError::CannotCompileLine {
                    line: line.to_string(),
                    error,
                })?;

            for opcode in opcodes {
                state.exec_inst(opcode, true, helper).map_err(|err| {
                    let mipsy_error = match err {
                        CommandError::RuntimeError { mipsy_error } => mipsy_error,
                        _ => unreachable!(),
                    };

                    CommandError::ReplRuntimeError {
                        mipsy_error,
                        line: line.to_string(),
                    }
                })?;
            }

            Ok("".into())
        })
}
