pub fn format_percentage(numerator: u64, denominator: u64) -> String {
    format!("{:.2}%", numerator as f64 / denominator as f64 * 100f64)
}

pub fn format_bytes(bytes: u64) -> String {
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
