use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;
use mipsy_lib::compile::CompilerOptions;
use mipsy_parser::TaggedFile;
use mipsy_utils::expand_tilde;

pub(crate) fn load_command() -> Command {
    command_varargs(
        "load",
        vec!["l"],
        vec!["files"],
        "-- {args}".magenta().to_string(),
        "load a MIPS file to run",
        &format!(
            "Loads a MIPS file to run, overwriting whatever is currently loaded.\n\
             This command must be run prior to many others, such as `{}`, `{}`, `{}`, ...",
            "run".bold(),
            "step".bold(),
            "print".bold(),
        ),
        |state, _label, args| {

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

            let program: Vec<_> = files.iter()
                    .map(|name| {
                        let mut path = name;

                        #[cfg(unix)]
                        if path == "-" {
                            path = &stdin;
                        }

                        match std::fs::read_to_string(expand_tilde(path)) {
                            Ok(content) => Ok((path.to_string(), content)),
                            Err(err)    => Err(CommandError::CannotReadFile {
                                path: path.clone(),
                                os_error: err.to_string()
                            })
                        }
                    })
                    .collect::<Result<_, _>>()?;

            state.program = Some(program);
            let program = state.program.as_ref().unwrap();

            let binary_files = program.iter()
                    .map(|(path, file)| TaggedFile::new(Some(path), file))
                    .collect::<Vec<_>>();

            let binary = mipsy_lib::compile(&state.iset, binary_files, &CompilerOptions::default(), &state.config)
                .map_err(|err| CommandError::CannotCompile { mipsy_error: err })?;

            let runtime = mipsy_lib::runtime(&binary, &arguments.iter().map(|x| &**x).collect::<Vec<_>>());

            state.binary  = Some(binary);
            state.runtime = Some(runtime);
            state.exited  = false;

            let loaded = if program.len() == 1 {
                "file loaded"
            } else {
                "files loaded"
            };

            prompt::success_nl(loaded);

            Ok(())
        }
    )
}
