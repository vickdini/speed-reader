use std::path::Path;
use lopdf::Document as LopdfDoc;
use super::epub::ExtractedSection;

pub fn extract_pdf_sections(path: &Path) -> Result<(String, Vec<ExtractedSection>), String> {
    let title = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled PDF")
        .to_string();

    // Primary extraction: lopdf gives exact 1-to-1 page extraction
    if let Ok(doc) = LopdfDoc::load(path) {
        let pages = doc.get_pages();
        if !pages.is_empty() {
            let mut sections = Vec::with_capacity(pages.len());
            let mut sorted_page_nums: Vec<u32> = pages.keys().copied().collect();
            sorted_page_nums.sort_unstable();

            for page_num in sorted_page_nums {
                let page_text = doc.extract_text(&[page_num]).unwrap_or_default();
                let cleaned = clean_pdf_text(&page_text);

                let meta = detect_pdf_page_meta(&cleaned);
                let title_label = if let Some(sub) = meta {
                    format!("Page {} — {}", page_num, sub)
                } else {
                    format!("Page {}", page_num)
                };

                sections.push(ExtractedSection {
                    title: title_label,
                    content: if cleaned.trim().is_empty() {
                        format!("[Page {} - Image / Blank Page]", page_num)
                    } else {
                        cleaned
                    },
                });
            }

            let has_content = sections.iter().any(|s| !s.content.starts_with("[Page "));
            if has_content {
                return Ok((title, sections));
            }
        }
    }

    // Fallback extraction: pdf_extract
    let raw = pdf_extract::extract_text(path)
        .map_err(|e| format!("Failed to extract PDF text: {}", e))?;

    let raw_pages: Vec<&str> = if raw.contains('\x0C') {
        raw.split('\x0C').collect()
    } else {
        raw.split("\n\n\n").collect()
    };

    let mut sections = Vec::new();
    let mut page_num = 1;
    for chunk in raw_pages {
        let cleaned = clean_pdf_text(chunk);
        if !cleaned.trim().is_empty() {
            sections.push(ExtractedSection {
                title: format!("Page {}", page_num),
                content: cleaned,
            });
            page_num += 1;
        }
    }

    if sections.is_empty() {
        return Err("PDF contains no extractable text (it might be a scanned image-only PDF).".to_string());
    }

    Ok((title, sections))
}

pub fn extract_pdf_text(path: &Path) -> Result<(String, String), String> {
    let (title, sections) = extract_pdf_sections(path)?;
    let full_text = sections.into_iter()
        .map(|s| s.content)
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok((title, full_text))
}

fn detect_pdf_page_meta(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return None;
    }

    // Check first 3 lines for chapter headings
    for &line in lines.iter().take(3) {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("chapter ")
            || lower.starts_with("prologue")
            || lower.starts_with("epilogue")
            || lower.starts_with("contents")
            || lower.starts_with("index")
            || lower.starts_with("acknowledgments")
        {
            let clean: String = line.chars().take(40).collect();
            return Some(clean);
        }
    }

    // Check running header line with page number (e.g. "One Company's Experience: Deere 215")
    if let Some(&first) = lines.first() {
        if first.len() < 55 && (first.chars().any(|c| c.is_numeric()) && first.chars().any(|c| c.is_alphabetic())) {
            let clean: String = first.chars().take(40).collect();
            return Some(clean);
        }
    }

    None
}

fn clean_pdf_text(input: &str) -> String {
    let preprocessed = input.replace('\x0C', "\n\n");
    crate::text_cleaner::clean_text_for_reading(&preprocessed)
}
