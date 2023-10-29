use super::parser::{SourceMapping, EMPTY_MAPPING};

#[derive(Debug)]
pub struct SourceMappingFileInfo<'file_name> {
    pub bytes: u32,
    pub file_name: &'file_name str,
}

pub fn calculate_size_by_file<'source_mapping>(
    file_contents: &'source_mapping str,
    source_mapping: &'source_mapping SourceMapping,
) -> Vec<SourceMappingFileInfo<'source_mapping>> {
    let file_lines = file_contents.lines().collect::<Vec<&str>>();

    let mut info_by_file = source_mapping
        .sources()
        .iter()
        .map(|file_name| SourceMappingFileInfo {
            bytes: 0,
            file_name,
        })
        .collect::<Vec<_>>();

    let mut prev_mapping = &EMPTY_MAPPING;
    let mappings = source_mapping.mappings();
    for (index, mapping) in mappings.iter().enumerate() {
        let info = info_by_file.get_mut(mapping.src_file() as usize).unwrap();

        if index == 0
            || mapping.src_file() != prev_mapping.src_file()
            || mapping.gen_line() != prev_mapping.gen_line()
        {
            // Source maps usually skip keywords and other non-identifier tokens,
            // so if either file or line has changed compared to the previous mapping
            // we probably should add the amount of characters from the start of line.
            // E.g. function example() {} --> mapping might point to "example",
            // but "function" was skipped.
            info.bytes += mapping.gen_column();
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

        info.bytes += mapping_end_column - mapping.gen_column();

        prev_mapping = mapping;
    }

    info_by_file
}
