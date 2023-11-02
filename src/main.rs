use anyhow::{anyhow, Error, Result};
use core::output::print_file_info;
use std::env;

use crate::core::{analyzer::calculate_size_by_file, parser::parse_file_by_path};

mod core;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let path_meta = std::fs::metadata(&args[1])?;

            let mut files_to_check: Vec<String> = vec![];

            if path_meta.is_dir() {
                for entry in std::fs::read_dir(&args[1])? {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            match path.extension().unwrap().to_str().unwrap() {
                                "js" => files_to_check.push(path.to_str().unwrap().to_owned()),
                                _ => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
            } else {
                files_to_check.push(args[1].clone());
            }

            files_to_check.sort();

            let mut files_with_errors: Vec<(&str, Error)> = vec![];

            for file in files_to_check.iter() {
                match handle_file(file) {
                    Ok(_) => {}
                    Err(err) => files_with_errors.push((file, err)),
                }
            }

            for (file, err) in files_with_errors {
                println!(
                    "Error when parsing file {file}, make sure the sourcemap is correct: {err}"
                );
            }

            println!("Number of files checked: {}", files_to_check.len());

            Ok(())
        }
        _ => Err(anyhow!(
            "Expected exactly 1 argument, {} were passed",
            args.len() - 1
        )),
    }
}

fn handle_file(file: &str) -> Result<()> {
    let (file_contents, mapping) = parse_file_by_path(file)?;

    let info = calculate_size_by_file(&file_contents, &mapping)?;

    print_file_info(&mapping, &info);

    Ok(())
}
