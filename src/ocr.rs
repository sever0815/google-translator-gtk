use std::path::Path;
use std::process::Command;

/// Run tesseract on an image file and return the recognized text.
/// `lang_hint` is a tesseract language code (e.g. "rus"); when None — or when
/// no traineddata is installed for it — all installed languages are used.
pub fn ocr_image(path: &Path, lang_hint: Option<&str>) -> Result<String, String> {
    if Command::new("tesseract").arg("--version").output().is_err() {
        return Err(
            "tesseract not found — install it with language data \
             (Gentoo: emerge app-text/tesseract app-text/tessdata_fast; \
             Arch: pacman -S tesseract tesseract-data-eng tesseract-data-rus)"
                .into(),
        );
    }
    let installed = installed_langs();
    if installed.is_empty() {
        return Err(
            "no tesseract language data installed \
             (Gentoo: emerge app-text/tessdata_fast)"
                .into(),
        );
    }
    let out = Command::new("tesseract")
        .arg(path)
        .arg("-")
        .arg("-l")
        .arg(pick_langs(lang_hint, &installed))
        .output()
        .map_err(|e| format!("tesseract failed to start: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "tesseract error: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        return Err("No text recognized in image".into());
    }
    Ok(text)
}

/// Use the hinted language only if its traineddata is actually present;
/// tessdata_fast ships whatever LINGUAS selected, which is usually a subset.
fn pick_langs(hint: Option<&str>, installed: &[String]) -> String {
    match hint {
        Some(l) if installed.iter().any(|i| i == l) => l.to_string(),
        _ => installed.join("+"),
    }
}

fn installed_langs() -> Vec<String> {
    let Ok(out) = Command::new("tesseract").arg("--list-langs").output() else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .skip(1) // "List of available languages ..." header
        .map(str::trim)
        .filter(|l| !l.is_empty() && *l != "osd")
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::pick_langs;

    #[test]
    fn hint_used_only_when_installed() {
        let have: Vec<String> = ["eng", "rus"].iter().map(|s| s.to_string()).collect();
        assert_eq!(pick_langs(Some("rus"), &have), "rus");
        assert_eq!(pick_langs(Some("fra"), &have), "eng+rus"); // not in tessdata_fast
        assert_eq!(pick_langs(None, &have), "eng+rus");
    }
}
