use colored::Colorize;
use mipsy_lib::inst::{InstSignature, PseudoSignature};
use std::rc::Rc;

use mipsy_lib::{compile, MpProgram};
use mipsy_parser::{parser::MpAttributedItem, MpItem};

use crate::interactive::{commands::watchpoint::args_text, error::CommandError};

use super::*;

fn instruction_names(h: &MyHelper) -> Vec<String> {
    h.state
        .iset
        .native_set()
        .iter()
        .map(InstSignature::name)
        .chain(h.state.iset.pseudo_set().iter().map(PseudoSignature::name))
        .map(str::to_owned)
        .collect()
}

pub(crate) fn command() -> Command {
    Command::new()
        .with_name(".")
        .with_desc("execute a MIPS instruction")
        .with_var_args()
        .with_required_arg(Argument::new(
            "instruction",
            |_, a, h| match instruction_names(h).contains(&a.to_owned()) {
                true => Ok(ArgumentKind::String(a.to_owned())),
                false => Err(CommandError::BadArgument {
                    arg: "instruction".to_owned(),
                    instead: a.to_owned(),
                }),
            },
            |_, _, h| instruction_names(h),
        ))
        .with_help("Executes a MIPS instruction immediately".to_owned())
        .with_varargs_format("{args}".magenta().to_string())
        .with_exec(|_, helper, args| {
            let line = args_text(args).join(" ");

            let inst = mipsy_parser::parse_instruction(&line, helper.state.config.tab_size)
                .map_err(|error| CommandError::CannotParseLine {
                    line: line.to_string(),
                    error,
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

            let binary = helper
                .state
                .binary
                .as_ref()
                .ok_or(CommandError::MustLoadFile)?;

            compile::check_post_data_label(&program, binary).map_err(|error| {
                CommandError::CannotCompileLine {
                    line: line.to_string(),
                    error,
                }
            })?;

            let opcodes = mipsy_lib::compile1(binary, &helper.state.iset, &inst)
                .map_err(|error| {
                    error.into_compiler_mipsy_error(Rc::from(""), 1, inst.col(), inst.col_end())
                })
                .map_err(|error| CommandError::CannotCompileLine {
                    line: line.to_string(),
                    error,
                })?;

            for opcode in opcodes {
                State::exec_inst(helper, opcode, true).map_err(|err| {
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
