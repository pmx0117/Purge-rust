pub fn format_bytes(bytes: u64) -> String {
    let mut value = bytes as f64;
    let mut unit = "B";
    for next in ["KB", "MB", "GB", "TB"] {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = next;
    }
    if unit == "B" {
        format!("{bytes} B")
    } else if value >= 100.0 {
        format!("{value:.0} {unit}")
    } else {
        format!("{value:.1} {unit}")
    }
}

/// Windows reports binary storage units; retain Purge's one-decimal total style.
pub fn format_storage_bytes(bytes: u64) -> String {
    let mut value = bytes as f64;
    let mut unit = "B";
    for next in ["KB", "MB", "GB", "TB"] {
        if value < 1024. {
            break;
        }
        value /= 1024.;
        unit = next;
    }
    let number = format!("{value:.1}");
    format!("{} {unit}", number.strip_suffix(".0").unwrap_or(&number))
}
