use console::Style;

use super::{analyzer::SourceMappingFileInfo, parser::SourceMapping};

pub fn print_file_info(mapping: &SourceMapping, info: &[SourceMappingFileInfo]) {
    let file_style = Style::new().underlined();
    let highlight_style = Style::new().cyan();
    let highlight_style2 = Style::new().green();

    let source_file_meta = std::fs::metadata(mapping.file()).unwrap();
    let source_file_len = source_file_meta.len() - mapping.source_map_len();

    println!(
        "File {}, total size {}:",
        file_style.apply_to(mapping.file()),
        highlight_style.apply_to(format_bytes(source_file_len))
    );

    let mut info = info.iter().collect::<Vec<&SourceMappingFileInfo>>();
    info.sort_by_key(|i| i.bytes);

    for info in info.iter().rev() {
        println!(
            "- {}, size {} ({})",
            file_style.apply_to(info.file_name),
            highlight_style.apply_to(format_bytes(info.bytes as u64)),
            highlight_style2.apply_to(format!(
                "{:.2}%",
                info.bytes as f64 / source_file_len as f64 * 100f64
            )),
        );
    }
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
