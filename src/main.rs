use anyhow::{anyhow, Result};
use std::env;

use crate::core::parser::parse_file_by_path;

mod core;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            println!("{}", args[1]);
            let mapping = parse_file_by_path(&args[1])?;

            println!("{:?}", mapping.mappings().last());

            Ok(())
        }
        _ => Err(anyhow!(
            "Expected exactly 1 argument, {} were passed",
            args.len() - 1
        )),
    }
}
