use self::{
    analyzer::{calculate_size_by_file, SourceMappingInfo},
    parser::parse_file_by_path,
};
use anyhow::{Error, Result};

pub mod analyzer;
pub mod parser;
mod vlq;

pub fn analyze_path(path: &str, mut on_file_result: impl FnMut(&str, Result<SourceMappingInfo, Error>)) -> Result<()> {
    let files_to_check = discover_files(path)?;

    for file in files_to_check.iter() {
        on_file_result(file, handle_file(file));
    }

    Ok(())
}

pub fn discover_files(path: &str) -> Result<Vec<String>> {
    let path_meta = std::fs::metadata(path)?;

    let mut files_to_check: Vec<String> = vec![];

    if path_meta.is_dir() {
        for entry in (std::fs::read_dir(path)?).flatten() {
            let path = entry.path();
            if let "js" = path.extension().unwrap().to_str().unwrap() {
                files_to_check.push(path.to_str().unwrap().to_owned())
            }
        }
    } else {
        files_to_check.push(path.to_owned());
    }

    files_to_check.sort();

    Ok(files_to_check)
}

pub fn handle_file(file: &str) -> Result<SourceMappingInfo> {
    let (file_contents, mapping) = parse_file_by_path(file)?;

    let info = calculate_size_by_file(&file_contents, mapping)?;

    Ok(info)
}
