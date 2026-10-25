use std::fs;
use std::path::Path;
use super::epub::ExtractedSection;

pub fn extract_txt_sections(path: &Path) -> Result<(String, Vec<ExtractedSection>), String> {
    let bytes = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    
    let content = match String::from_utf8(bytes.clone()) {
        Ok(s) => s,
        Err(_) => String::from_utf8_lossy(&bytes).to_string(),
    };

    let title = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled Document")
        .to_string();

    if content.trim().is_empty() {
        return Err("File is empty".to_string());
    }

    let mut sections = Vec::new();
    let mut current_section_title = "Introduction".to_string();
    let mut current_section_content = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        // Check if markdown header
        if trimmed.starts_with("# ") || trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            if !current_section_content.trim().is_empty() {
                sections.push(ExtractedSection {
                    title: current_section_title,
                    content: current_section_content.clone(),
                });
                current_section_content.clear();
            }
            let heading_text = trimmed.trim_start_matches('#').trim().to_string();
            current_section_title = heading_text;
        } else {
            current_section_content.push_str(line);
            current_section_content.push('\n');
        }
    }

    if !current_section_content.trim().is_empty() {
        sections.push(ExtractedSection {
            title: current_section_title,
            content: current_section_content,
        });
    }

    // If no markdown headings were found and the text is large, chunk by ~1200 words
    if sections.len() <= 1 {
        let full_text = if sections.is_empty() { content } else { sections.remove(0).content };
        let words: Vec<&str> = full_text.split_whitespace().collect();
        if words.len() > 1500 {
            let chunk_size = 1200;
            let mut chunk_idx = 1;
            for chunk in words.chunks(chunk_size) {
                sections.push(ExtractedSection {
                    title: format!("Section {}", chunk_idx),
                    content: chunk.join(" "),
                });
                chunk_idx += 1;
            }
        } else {
            sections.push(ExtractedSection {
                title: "Full Text".to_string(),
                content: full_text,
            });
        }
    }

    Ok((title, sections))
}

pub fn extract_txt_text(path: &Path) -> Result<(String, String), String> {
    let (title, sections) = extract_txt_sections(path)?;
    let full_text = sections.into_iter()
        .map(|s| s.content)
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok((title, full_text))
}
