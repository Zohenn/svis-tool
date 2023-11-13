use anyhow::{anyhow, Error, Result};

use sourcemap_vis::ui::terminal::{get_default_styles, print_file_info};

use sourcemap_vis::core::analyze_path;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        2 => {
            let styles = get_default_styles();
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
                    "{} Error when parsing file {}, make sure the sourcemap is correct:\n- {}",
                    styles.error.apply_to("!"),
                    styles.file.apply_to(file),
                    err,
                );
            }

            println!(
                "Files checked: {}",
                styles.highlight.apply_to(files_checked)
            );

            Ok(())
        }
        _ => Err(anyhow!(
            "Expected exactly 1 argument, {} were passed",
            args.len() - 1
        )),
    }
}
