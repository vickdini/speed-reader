pub mod epub;
pub mod pdf;
pub mod txt;

use std::collections::HashMap;
use std::path::Path;
pub use crate::reader::{ExtractedSection, ParsedDocument, ReadingImage, ReadingTable, SectionBlock};

pub fn load_document_sections_with_mode(
    path: &Path,
    epub_by_page: bool,
) -> Result<ParsedDocument, String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "pdf" => {
            let (title, sections) = pdf::extract_pdf_sections(path)?;
            Ok(ParsedDocument {
                title,
                sections,
                images: HashMap::new(),
            })
        }
        "epub" => epub::extract_epub_sections_with_mode(path, epub_by_page),
        "txt" | "text" | "md" | "markdown" | "rst" | "log" | "" => {
            let (title, sections) = txt::extract_txt_sections(path)?;
            Ok(ParsedDocument {
                title,
                sections,
                images: HashMap::new(),
            })
        }
        _ => {
            let (title, sections) = txt::extract_txt_sections(path)?;
            Ok(ParsedDocument {
                title,
                sections,
                images: HashMap::new(),
            })
        }
    }
}

pub fn load_document_sections(path: &Path) -> Result<(String, Vec<ExtractedSection>), String> {
    let parsed = load_document_sections_with_mode(path, true)?;
    Ok((parsed.title, parsed.sections))
}

pub fn load_document(path: &Path) -> Result<(String, String), String> {
    let (title, sections) = load_document_sections(path)?;
    let full_text = sections
        .into_iter()
        .map(|s| s.content)
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok((title, full_text))
}
