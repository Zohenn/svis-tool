use console::Style;

use super::{
    analyzer::{SourceMappingFileInfo, SourceMappingInfo},
    parser::SourceMapping,
};

pub fn print_file_info(mapping: &SourceMapping, info: &SourceMappingInfo) {
    let file_style = Style::new().bold();
    let highlight_style = Style::new().cyan();
    let highlight_style2 = Style::new().green();

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

    for info in info_by_file.iter().rev() {
        println!(
            "- {}, size {} ({})",
            file_style.apply_to(without_relative_part(info.file_name)),
            highlight_style.apply_to(format_bytes(info.bytes as u64)),
            highlight_style2.apply_to(format_percentage(info.bytes as u64, source_file_len)),
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

fn format_percentage(numerator: u64, denominator: u64) -> String {
    format!("{:.2}%", numerator as f64 / denominator as f64 * 100f64)
}

fn format_bytes(bytes: u64) -> String {
    let kilos = bytes as f64 / 1024f64;
    let megs = kilos as f64 / 1024f64;

    if megs > 1f64 {
        format!("{megs:.2} MiB")
    } else if kilos > 1f64 {
        format!("{kilos:.2} KiB")
    } else {
        format!("{bytes} B")
    }
}
