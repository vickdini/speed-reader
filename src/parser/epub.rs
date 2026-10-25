use std::path::Path;
use epub::doc::EpubDoc;

pub fn extract_epub_text(path: &Path) -> Result<(String, String), String> {
    let mut doc = EpubDoc::new(path).map_err(|e| format!("Failed to open EPUB: {}", e))?;
    
    let title = doc.mdata("title")
        .map(|t| t.value.clone())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled EPUB")
                .to_string()
        });

    let mut full_text = String::new();

    // Iterate through all spine items/resources
    let len = doc.get_num_chapters();
    for _ in 0..len {
        if let Some((content, _mime)) = doc.get_current_str() {
            let clean = strip_html_tags(&content);
            if !clean.trim().is_empty() {
                full_text.push_str(&clean);
                full_text.push_str("\n\n");
            }
        }
        if !doc.go_next() {
            break;
        }
    }

    if full_text.trim().is_empty() {
        return Err("EPUB file contains no readable text content".to_string());
    }

    Ok((title, full_text))
}

pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut in_script_or_style = false;

    let mut chars = html.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            in_tag = true;
            tag_name.clear();
            // Read tag name
            let mut is_closing = false;
            if let Some(&next_c) = chars.peek() {
                if next_c == '/' {
                    is_closing = true;
                    chars.next();
                }
            }
            while let Some(&tc) = chars.peek() {
                if tc.is_whitespace() || tc == '>' || tc == '/' {
                    break;
                }
                tag_name.push(tc.to_ascii_lowercase());
                chars.next();
            }

            if tag_name == "script" || tag_name == "style" {
                in_script_or_style = !is_closing;
            }

            if !in_script_or_style && !is_closing {
                if tag_name == "p" || tag_name == "div" || tag_name == "h1" || tag_name == "h2" 
                   || tag_name == "h3" || tag_name == "h4" || tag_name == "h5" || tag_name == "h6" 
                   || tag_name == "li" || tag_name == "tr" {
                    result.push('\n');
                } else if tag_name == "br" {
                    result.push('\n');
                }
            }
            continue;
        }

        if c == '>' {
            in_tag = false;
            continue;
        }

        if in_tag || in_script_or_style {
            continue;
        }

        if c == '&' {
            let mut entity = String::new();
            while let Some(&ec) = chars.peek() {
                if ec == ';' || ec.is_whitespace() || entity.len() > 10 {
                    if ec == ';' {
                        chars.next();
                    }
                    break;
                }
                entity.push(ec);
                chars.next();
            }

            match entity.as_str() {
                "nbsp" => result.push(' '),
                "amp" => result.push('&'),
                "lt" => result.push('<'),
                "gt" => result.push('>'),
                "quot" => result.push('"'),
                "apos" => result.push('\''),
                "mdash" => result.push('—'),
                "ndash" => result.push('–'),
                "hellip" => result.push_str("..."),
                "rsquo" | "lsquo" => result.push('\''),
                "rdquo" | "ldquo" => result.push('"'),
                s if s.starts_with('#') => {
                    if let Ok(code) = s[1..].parse::<u32>() {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    } else if s.starts_with("#x") || s.starts_with("#X") {
                        if let Ok(code) = u32::from_str_radix(&s[2..], 16) {
                            if let Some(ch) = char::from_u32(code) {
                                result.push(ch);
                            }
                        }
                    }
                }
                _ => result.push(' '),
            }
            continue;
        }

        result.push(c);
    }

    // Collapse excessive empty lines and spaces
    let mut cleaned = String::with_capacity(result.len());
    let mut consecutive_newlines = 0;
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                cleaned.push('\n');
            }
        } else {
            consecutive_newlines = 0;
            cleaned.push_str(trimmed);
            cleaned.push('\n');
        }
    }

    cleaned
}
