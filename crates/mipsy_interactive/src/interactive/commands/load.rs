use crate::interactive::{commands::watchpoint::args_text, error::CommandError, prompt};

use super::*;
use colored::*;
use mipsy_lib::compile::CompilerOptions;
use mipsy_parser::TaggedFile;
use mipsy_utils::expand_tilde;

pub(crate) fn command() -> Command {
    Command::new()
        .with_name("load")
        .with_name("l")
        .with_var_args(Argument::new(
            "<files>".magenta().to_string(),
            |_, a, _| Ok(ArgumentKind::String(a.to_owned())),
            |_, a, h| h.file_hints(a),
        ) )
        .with_required_arg(Argument::new(
            "files",
            |_, a, _| Ok(ArgumentKind::String(a.to_owned())),
            |_, a, h| h.file_hints(a),
        ))
        .with_desc("load a MIPS file to run")
        .with_help(format!(
            "Loads a MIPS file to run, overwriting whatever is currently loaded.\n\
                This command must be run prior to many others, such as `{}`, `{}`, `{}`, ...",
            "run".bold(),
            "step".bold(),
            "print".bold(),
        ))
        .with_exec(|_, helper, args| {
            let args = &args_text(args)[..];

            let (files, arguments) = {
                if let Some(index) = args.iter().position(|arg| arg == "--") {
                    let (files, arguments) = args.split_at(index);

                    (files, &arguments[1..])
                } else {
                    (args, &[][..])
                }
            };

            #[cfg(unix)]
            let stdin = String::from("/dev/stdin");

            let program = {
                let mut program = Vec::with_capacity(files.len());
                for file in files.iter().map(|mut name| {
                    #[cfg(unix)]
                    if name == "-" {
                        name = &stdin;
                    }

                    match std::fs::read_to_string(expand_tilde(name)) {
                        Ok(content) => Ok((name.to_string(), content)),
                        Err(err) => Err(CommandError::CannotReadFile {
                            path: name.to_string(),
                            os_error: err.to_string(),
                        }),
                    }
                }) {
                    program.push(file?)
                }
                program
            };

            helper.state.program = Some(program);
            let program = helper.state.program.as_ref().unwrap();

            let binary_files = program
                .iter()
                .map(|(path, file)| TaggedFile::new(Some(path), file))
                .collect();

            let binary = mipsy_lib::compile(
                &helper.state.iset,
                binary_files,
                &CompilerOptions::default(),
                &helper.state.config,
            )
            .map_err(|err| CommandError::CannotCompile { mipsy_error: err })?;

            let runtime = mipsy_lib::runtime(
                &binary,
                arguments
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .as_slice(),
            );

            helper.state.binary = Some(binary);
            helper.state.runtime = runtime;
            helper.state.exited = false;

            let loaded = if program.len() == 1 {
                "file loaded"
            } else {
                "files loaded"
            };

            prompt::success_nl(loaded);

            Ok("".into())
        })
}
