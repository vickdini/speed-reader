pub mod epub;
pub mod pdf;
pub mod txt;

pub use epub::ExtractedSection;
use std::path::Path;

pub fn load_document_sections(path: &Path) -> Result<(String, Vec<ExtractedSection>), String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    let ext = path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "pdf" => pdf::extract_pdf_sections(path),
        "epub" => epub::extract_epub_sections(path),
        "txt" | "text" | "md" | "markdown" | "rst" | "log" | "" => txt::extract_txt_sections(path),
        _ => txt::extract_txt_sections(path),
    }
}

pub fn load_document(path: &Path) -> Result<(String, String), String> {
    let (title, sections) = load_document_sections(path)?;
    let full_text = sections.into_iter()
        .map(|s| s.content)
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok((title, full_text))
}
