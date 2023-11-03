use console::Style;

use crate::{
    core::{
        analyzer::{SourceMappingFileInfo, SourceMappingInfo},
        parser::SourceMapping,
    },
    output::utils::{format_bytes, format_percentage},
};

pub fn print_file_info(info: &SourceMappingInfo) {
    let file_style = Style::new().bold();
    let highlight_style = Style::new().cyan();
    let highlight_style2 = Style::new().green();

    let mapping = &info.source_mapping;

    if mapping.is_empty() {
        println!(
            "File {} contains empty sourcemap (both \"sources\" and \"mappings\" arrays are empty)",
            file_style.apply_to(mapping.file())
        );
        return;
    }

    let sources_root = get_sources_root(mapping);

    let source_file_len = mapping.source_file_len() - mapping.source_map_len();

    println!(
        "File {}, total size {}.",
        file_style.apply_to(mapping.file()),
        highlight_style.apply_to(format_bytes(source_file_len))
    );
    println!(
        "Size contribution per file (all paths are relative to {}):",
        file_style.apply_to(sources_root)
    );

    let mut info_by_file = info
        .info_by_file
        .iter()
        .collect::<Vec<&SourceMappingFileInfo>>();
    info_by_file.sort_by_key(|i| i.bytes);

    for file_info in info_by_file.iter().rev() {
        println!(
            "- {}, size {} ({})",
            file_style.apply_to(without_relative_part(info.get_file_name(file_info.file))),
            highlight_style.apply_to(format_bytes(file_info.bytes as u64)),
            highlight_style2.apply_to(format_percentage(file_info.bytes as u64, source_file_len)),
        );
    }

    let sum_bytes = info.sum_bytes as u64;

    println!(
        "Total: {} ({})",
        highlight_style.apply_to(format_bytes(sum_bytes)),
        highlight_style2.apply_to(format_percentage(sum_bytes, source_file_len)),
    );

    let rest = source_file_len - sum_bytes;
    println!(
        "Remaining size taken by preamble, imports, whitespace, comments, etc.: {} ({})",
        highlight_style.apply_to(format_bytes(rest)),
        highlight_style2.apply_to(format_percentage(rest, source_file_len))
    );
}

fn get_sources_root(mapping: &SourceMapping) -> String {
    match mapping.source_root() {
        Some(path) if !path.is_empty() => return path.to_owned(),
        _ => {}
    }

    // This looks like crap
    let relative_jumps = mapping
        .sources()
        .first()
        .unwrap()
        .split('/')
        .take_while(|part| part == &"..")
        .count();

    // TODO: This looks like crap even more
    mapping
        .file()
        .split('/')
        .rev()
        .skip((relative_jumps + 1) as usize)
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .collect::<Vec<&str>>()
        .join("/")
}

fn without_relative_part(file: &str) -> &str {
    file.trim_start_matches("../")
}
