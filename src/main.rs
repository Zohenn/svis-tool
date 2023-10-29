use anyhow::{anyhow, Result};
use core::output::print_file_info;
use std::env;

use crate::core::{analyzer::calculate_size_by_file, parser::parse_file_by_path};

mod core;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let (file_contents, mapping) = parse_file_by_path(&args[1])?;

            let info = calculate_size_by_file(&file_contents, &mapping);

            print_file_info(&mapping, &info);

            Ok(())
        }
        _ => Err(anyhow!(
            "Expected exactly 1 argument, {} were passed",
            args.len() - 1
        )),
    }
}
