use std::collections::HashMap;

use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;

pub(crate) fn load_command() -> Command {
    command_varargs(
        "load",
        vec!["l"],
        vec!["file"],
        "load a MIPS file to run",
        &format!(
            "Loads a MIPS file to run, overwriting whatever is currently loaded.\n\
             This command must be run prior to many others, such as `{}`, `{}`, `{}`, ...",
            "run".bold(),
            "step".bold(),
            "print".bold(),
        ),
        |state, _label, args| {

            let program: HashMap<_, _> = args.iter()
                    .map(|path| {
                        match std::fs::read_to_string(path) {
                            Ok(content) => Ok((path.to_string(), content)),
                            Err(err)     => Err(CommandError::CannotReadFile { path: path.clone(), os_error: err.to_string() })
                        }
                    })
                    .collect::<Result<_, _>>()?;

            state.program = Some(program);
            let program = state.program.as_ref().unwrap();

            let binary_files = program.iter()
                    .map(|(path, file)| (Some(&**path), &**file))
                    .collect::<Vec<_>>();

            let binary = mipsy_lib::compile(&state.iset, binary_files)
                .map_err(|err| CommandError::CannotCompile { mipsy_error: err })?;

            let runtime = mipsy_lib::runtime(&binary);

            state.binary  = Some(binary);
            state.runtime = Some(runtime);
            state.exited  = false;
            prompt::success_nl("file loaded");

            Ok(())
        }
    )
}
