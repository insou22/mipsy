use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;

pub(crate) fn load_command() -> Command {
    command(
        "load",
        vec!["l"],
        vec!["file"],
        vec![],
        "load a MIPS file to run",
        &format!(
            "Loads a MIPS file to run, overwriting whatever is currently loaded.\n\
             This command must be run prior to many others, such as `{}`, `{}`, `{}`, ...",
            "run".bold(),
            "step".bold(),
            "print".bold(),
        ),
        |state, _label, args| {
            let path = &args[0];

            let program = std::fs::read_to_string(path)
                .map_err(|err| CommandError::CannotReadFile { path: path.clone(), os_error: err.to_string() })?;
            
            let binary = mipsy_lib::compile(&state.iset, &program)
                .map_err(|err| CommandError::CannotCompile { path: path.clone(), program: program.clone(), mipsy_error: err })?;

            let runtime = mipsy_lib::run(&binary)
                .map_err(|err| CommandError::CannotCompile { path: path.clone(), program: program.clone(), mipsy_error: err })?;

            state.binary  = Some(binary);
            state.runtime = Some(runtime);
            state.exited  = false;
            prompt::success_nl("file loaded");

            Ok(())
        }
    )
}
