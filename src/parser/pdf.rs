use std::path::Path;

pub fn extract_pdf_text(path: &Path) -> Result<(String, String), String> {
    let raw = pdf_extract::extract_text(path)
        .map_err(|e| format!("Failed to extract PDF text: {}", e))?;

    let title = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled PDF")
        .to_string();

    let cleaned = clean_pdf_text(&raw);
    if cleaned.trim().is_empty() {
        return Err("PDF contains no extractable text (it might be a scanned image-only PDF).".to_string());
    }

    Ok((title, cleaned))
}

fn clean_pdf_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // Check if line ends with a soft hyphen indicating a word break across lines
        if let Some(without_hyphen) = trimmed.strip_suffix('-') {
            output.push_str(without_hyphen);
            // Don't add newline or space; attach directly to next word if not empty
            if let Some(&next_line) = lines.peek() {
                let next_trimmed = next_line.trim_start();
                if let Some(first_word) = next_trimmed.split_whitespace().next() {
                    output.push_str(first_word);
                    // Skip the first word of the next line by manual slice in next iteration if needed
                    // For simplicity, let's just push a space if next line exists
                }
            }
        } else {
            output.push_str(trimmed);
            output.push('\n');
        }
    }

    // Replace form-feeds (\x0C) and normalize
    output.replace('\x0C', "\n\n")
}
