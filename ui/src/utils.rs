pub fn without_relative_part(file: &str) -> &str {
    file.trim_start_matches("../")
}

pub fn format_percentage(numerator: u64, denominator: u64) -> String {
    format!("{:.2}%", numerator as f64 / denominator as f64 * 100f64)
}

pub fn format_bytes(bytes: u64) -> String {
    let kilos = bytes as f64 / 1024f64;
    let megs = kilos / 1024f64;

    if megs > 1f64 {
        format!("{megs:.2} M")
    } else if kilos > 1f64 {
        format!("{kilos:.2} K")
    } else {
        format!("{bytes} B")
    }
}
