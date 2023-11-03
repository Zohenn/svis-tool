use anyhow::{anyhow, Error, Result};
use std::env;

use output::terminal::print_file_info;

use crate::core::analyze_path;

mod core;
mod output;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let mut files_checked = 0u32;
            let mut files_with_errors: Vec<(String, Error)> = vec![];

            analyze_path(&args[1], |file, result| {
                files_checked += 1;
                match result {
                    Ok(info) => print_file_info(&info),
                    Err(err) => files_with_errors.push((file.to_owned(), err)),
                }
            })?;

            for (file, err) in files_with_errors {
                println!(
                    "Error when parsing file {file}, make sure the sourcemap is correct: {err}"
                );
            }

            println!("Number of files checked: {}", files_checked);

            Ok(())
        }
        _ => Err(anyhow!(
            "Expected exactly 1 argument, {} were passed",
            args.len() - 1
        )),
    }
}
