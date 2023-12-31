use anyhow::{anyhow, Context, Result};

use super::parser::{SourceMapping, EMPTY_MAPPING};

#[derive(Debug)]
pub struct SourceMappingFileInfo {
    pub bytes: u32,
    pub file: u32,
}

#[derive(Debug)]
pub struct SourceMappingInfo {
    pub source_mapping: SourceMapping,
    pub sum_bytes: u32,
    pub info_by_file: Vec<SourceMappingFileInfo>,
}

impl SourceMappingInfo {
    pub fn get_file_name(&self, file: u32) -> &str {
        &self.source_mapping.sources()[file as usize]
    }
}

pub fn calculate_size_by_file(file_contents: &str, source_mapping: SourceMapping) -> Result<SourceMappingInfo> {
    let file_lines = file_contents.lines().collect::<Vec<&str>>();

    let mut sum_bytes = 0u32;
    let mut info_by_file = source_mapping
        .sources()
        .into_iter()
        .enumerate()
        .map(|(file, _)| SourceMappingFileInfo {
            bytes: 0,
            file: file as u32,
        })
        .collect::<Vec<_>>();

    let mut prev_mapping = &EMPTY_MAPPING;
    let mappings = source_mapping.mappings();
    for (index, mapping) in mappings.iter().enumerate() {
        let info = info_by_file.get_mut(mapping.src_file() as usize).unwrap();
        let mut bytes = 0u32;

        if index == 0
            || //mapping.src_file() != prev_mapping.src_file() ||
            mapping.gen_line()
                != prev_mapping.gen_line()
        {
            // Source maps usually skip keywords and other non-identifier tokens,
            // so if either file or line has changed compared to the previous mapping
            // we probably should add the amount of characters from the start of line.
            // E.g. function example() {} --> mapping might point to "example",
            // but "function" was skipped.
            bytes += mapping.gen_column();
        }

        let line = file_lines[mapping.gen_line() as usize];

        let mapping_end_column = {
            let next_mapping = mappings.get(index + 1);

            match next_mapping {
                Some(next_mapping) => {
                    if next_mapping.gen_line() != mapping.gen_line() {
                        line.len() as u32
                    } else {
                        next_mapping.gen_column()
                    }
                }
                None => line.len() as u32,
            }
        };

        bytes += mapping_end_column.checked_sub(mapping.gen_column()).with_context(|| {
            // This only happens in my test project where sourcemap is invalid, e.g. it maps
            // inexistent columns in generated file to inexistent columns in source file.
            anyhow!(
                "Subtraction with overflow: calculating bytes for path {}, operation: {} - {}",
                source_mapping.file(),
                mapping_end_column,
                mapping.gen_column(),
            )
        })?;

        info.bytes += bytes;
        sum_bytes += bytes;

        prev_mapping = mapping;
    }

    Ok(SourceMappingInfo {
        source_mapping,
        sum_bytes,
        info_by_file,
    })
}
