use crate::interactive::error::CommandError;
use mipsy_lib::DATA_BOT;

use super::*;
use colored::*;

pub(crate) fn command() -> Command {
    Command::new()
        .with_name("labels")
        .with_name("ls")
        .with_name("las")
        .with_name("lbls")
        .with_desc("prints the addresses of all labels")
        .with_help("Prints the addresses of all labels in the currently loaded program.".to_owned())
        .with_exec(|_, helper, _| {
            let binary = helper
                .state
                .binary
                .as_ref()
                .ok_or(CommandError::MustLoadFile)?;

            let max_len = binary
                .labels
                .keys()
                .filter(|&label| {
                    !(label.starts_with("kernel__") || label == &String::from("_start"))
                })
                .map(|label| label.len())
                .max()
                .unwrap_or(0);

            let mut entries: Vec<(String, u32)> = binary
                .labels
                .iter()
                .map(|(key, &val)| (key.to_string(), val))
                .filter(|(key, _)| !(key.starts_with("kernel__") || key == &String::from("_start")))
                .collect();

            entries.sort_by_key(|(_, val)| *val);

            println!("\n{}", "[text]".green().bold());

            let mut printed_data_header = false;
            for (label, addr) in entries {
                if addr >= DATA_BOT && !printed_data_header {
                    println!("\n{}", "[data]".green().bold());
                    printed_data_header = true;
                }

                println!(
                    "{:max_len$} => 0x{:08x}",
                    label.yellow().bold(),
                    addr,
                    max_len = max_len
                );
            }
            println!();

            Ok("".into())
        })
}
