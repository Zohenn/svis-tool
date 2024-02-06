use console::Style;

use core::analyzer::{SourceMappingFileInfo, SourceMappingInfo};

use crate::utils::{format_bytes, format_percentage, without_relative_part};

pub struct Styles {
    pub file: Style,
    pub highlight: Style,
    pub highlight2: Style,
    pub error: Style,
}

pub fn get_default_styles() -> Styles {
    Styles {
        file: Style::new().bold(),
        highlight: Style::new().cyan(),
        highlight2: Style::new().green(),
        error: Style::new().red(),
    }
}

pub fn print_file_info(info: &SourceMappingInfo) {
    let styles = get_default_styles();

    let mapping = &info.source_mapping;

    if mapping.is_empty() {
        println!(
            "File {} contains empty sourcemap (both \"sources\" and \"mappings\" arrays are empty)",
            styles.file.apply_to(mapping.file())
        );
        return;
    }

    let sources_root = mapping.sources_root();

    let source_file_len = mapping.source_file_without_source_map_len();

    println!(
        "File {}, total size {}.",
        styles.file.apply_to(mapping.file()),
        styles.highlight.apply_to(format_bytes(source_file_len))
    );
    println!(
        "Size contribution per file (all paths are relative to {}):",
        styles.file.apply_to(sources_root)
    );

    let mut info_by_file = info.info_by_file.iter().collect::<Vec<&SourceMappingFileInfo>>();
    info_by_file.sort_by_key(|i| i.bytes);

    for file_info in info_by_file.iter().rev() {
        println!(
            "- {}, size {} ({})",
            styles
                .file
                .apply_to(without_relative_part(info.get_file_name(file_info.file))),
            styles.highlight.apply_to(format_bytes(file_info.bytes as u64)),
            styles
                .highlight2
                .apply_to(format_percentage(file_info.bytes as u64, source_file_len)),
        );
    }

    let sum_bytes = info.sum_bytes as u64;

    println!(
        "Total: {} ({})",
        styles.highlight.apply_to(format_bytes(sum_bytes)),
        styles
            .highlight2
            .apply_to(format_percentage(sum_bytes, source_file_len)),
    );

    let rest = source_file_len - sum_bytes;
    println!(
        "Remaining size taken by preamble, imports, whitespace, comments, etc.: {} ({})",
        styles.highlight.apply_to(format_bytes(rest)),
        styles.highlight2.apply_to(format_percentage(rest, source_file_len))
    );
}
