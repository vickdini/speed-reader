pub mod epub;
pub mod pdf;
pub mod txt;

use std::path::Path;

pub fn load_document(path: &Path) -> Result<(String, String), String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    let ext = path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "pdf" => pdf::extract_pdf_text(path),
        "epub" => epub::extract_epub_text(path),
        "txt" | "text" | "md" | "markdown" | "rst" | "log" | "" => txt::extract_txt_text(path),
        other => {
            // Attempt text loader as fallback
            txt::extract_txt_text(path)
                .map_err(|_| format!("Unsupported file extension '.{}'", other))
        }
    }
}
