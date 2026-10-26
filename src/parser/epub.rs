use std::collections::HashMap;
use std::path::Path;
use epub::doc::EpubDoc;

pub use crate::reader::{
    ExtractedSection, ParsedDocument, ReadingImage, ReadingTable, SectionBlock,
};

pub fn extract_epub_sections_with_mode(
    path: &Path,
    by_page: bool,
) -> Result<ParsedDocument, String> {
    let mut doc = EpubDoc::new(path).map_err(|e| format!("Failed to open EPUB: {}", e))?;

    let title = doc
        .mdata("title")
        .map(|t| t.value.clone())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled EPUB")
                .to_string()
        });

    // Extract all image resources into an images map for fast lookup
    let mut images: HashMap<String, Vec<u8>> = HashMap::new();

    if let Some((cover_bytes, _mime)) = doc.get_cover() {
        images.insert("cover".to_string(), cover_bytes.clone());
    }

    let resource_info: Vec<(String, std::path::PathBuf, String)> = doc
        .resources
        .iter()
        .map(|(id, res)| (id.clone(), res.path.clone(), res.mime.clone()))
        .collect();

    for (id, res_path, mime) in resource_info {
        let is_image = mime.starts_with("image/")
            || res_path.extension().and_then(|e| e.to_str()).map(|ext| {
                matches!(
                    ext.to_lowercase().as_str(),
                    "jpg" | "jpeg" | "png" | "webp" | "gif"
                )
            }).unwrap_or(false);

        if is_image {
            if let Some((bytes, _)) = doc.get_resource(&id) {
                // Store under resource ID
                images.insert(id.clone(), bytes.clone());
                // Store under file name
                if let Some(filename) = res_path.file_name().and_then(|n| n.to_str()) {
                    images.insert(filename.to_string(), bytes.clone());
                }
                // Store under normalized relative path
                if let Some(path_str) = res_path.to_str() {
                    let normalized = path_str.replace('\\', "/");
                    images.insert(normalized, bytes);
                }
            }
        }
    }

    let mut sections = Vec::new();
    let len = doc.get_num_chapters();
    let mut page_num = 1;

    for chapter_idx in 0..len {
        if let Some((content, _mime)) = doc.get_current_str() {
            let blocks = parse_html_section_blocks(&content);
            if !blocks.is_empty() {
                // Check if this chapter is a Cover page (e.g. only contains an image)
                let is_cover_chapter = blocks.len() == 1
                    && matches!(blocks[0], SectionBlock::Image(_))
                    && chapter_idx == 0;

                let chapter_title = if is_cover_chapter {
                    "Cover".to_string()
                } else {
                    extract_heading(&content)
                        .unwrap_or_else(|| format!("Chapter {}", chapter_idx + 1))
                };

                if by_page {
                    let (mut page_sections, next_page) =
                        paginate_epub_blocks(&chapter_title, blocks, page_num);
                    sections.append(&mut page_sections);
                    page_num = next_page;
                } else {
                    let full_content = blocks
                        .iter()
                        .map(|b| match b {
                            SectionBlock::Text(s) => s.clone(),
                            SectionBlock::Table(t) => t.to_text(),
                            SectionBlock::Image(i) => i.to_text(),
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    sections.push(ExtractedSection::with_blocks(
                        chapter_title,
                        full_content,
                        blocks,
                    ));
                }
            }
        }
        if !doc.go_next() {
            break;
        }
    }

    // If EPUB has a cover image but no Cover section was generated in the spine, prepend one
    if images.contains_key("cover") && !sections.iter().any(|s| s.title.to_lowercase().contains("cover")) {
        sections.insert(
            0,
            ExtractedSection::with_blocks(
                "Cover".to_string(),
                format!("[Illustration: {} Cover]", title),
                vec![SectionBlock::Image(ReadingImage {
                    image_id: "cover".to_string(),
                    caption: Some(format!("{} — Cover", title)),
                    alt: Some("Book Cover".to_string()),
                })],
            ),
        );
    }

    if sections.is_empty() {
        return Err("EPUB file contains no readable content".to_string());
    }

    Ok(ParsedDocument {
        title,
        sections,
        images,
    })
}

pub fn extract_epub_sections(path: &Path) -> Result<(String, Vec<ExtractedSection>), String> {
    let parsed = extract_epub_sections_with_mode(path, true)?;
    Ok((parsed.title, parsed.sections))
}

pub fn extract_epub_text(path: &Path) -> Result<(String, String), String> {
    let (title, sections) = extract_epub_sections(path)?;
    let full_text = sections
        .into_iter()
        .map(|s| s.content)
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok((title, full_text))
}

/// Parse HTML text into structured blocks of text paragraphs, tables, and images
pub fn parse_html_section_blocks(raw_html: &str) -> Vec<SectionBlock> {
    // Only parse content within <body>...</body> to ignore <head><title> etc.
    let lower_full = raw_html.to_ascii_lowercase();
    let html = if let Some(body_start) = lower_full.find("<body") {
        if let Some(tag_end) = lower_full[body_start..].find('>') {
            let content_start = body_start + tag_end + 1;
            if let Some(body_close) = lower_full[content_start..].find("</body>") {
                &raw_html[content_start..content_start + body_close]
            } else {
                &raw_html[content_start..]
            }
        } else {
            raw_html
        }
    } else {
        raw_html
    };

    let mut blocks = Vec::new();
    let lower = html.to_ascii_lowercase();
    let mut cur = 0;
    let len = html.len();

    while cur < len {
        // Find next <table, <figure, or standalone <img
        let next_table = lower[cur..].find("<table").map(|i| cur + i);
        let next_figure = lower[cur..].find("<figure").map(|i| cur + i);
        let next_img = lower[cur..].find("<img").map(|i| cur + i);

        // Pick earliest tag
        let mut candidates = Vec::new();
        if let Some(pos) = next_table {
            candidates.push((pos, "table"));
        }
        if let Some(pos) = next_figure {
            candidates.push((pos, "figure"));
        }
        if let Some(pos) = next_img {
            candidates.push((pos, "img"));
        }

        if candidates.is_empty() {
            // Remaining is plain text
            let trailing = strip_html_tags(&html[cur..]);
            for p in crate::reader::split_text_paragraphs(&trailing) {
                blocks.push(SectionBlock::Text(p));
            }
            break;
        }

        candidates.sort_by_key(|&(pos, _)| pos);
        let (tag_start, tag_type) = candidates[0];

        // Process any text preceding this tag
        if tag_start > cur {
            let preceding = strip_html_tags(&html[cur..tag_start]);
            for p in crate::reader::split_text_paragraphs(&preceding) {
                blocks.push(SectionBlock::Text(p));
            }
        }

        match tag_type {
            "table" => {
                let close_str = "</table>";
                if let Some(rel_close) = lower[tag_start..].find(close_str) {
                    let end_pos = tag_start + rel_close + close_str.len();
                    let table_html = &html[tag_start..end_pos];
                    if let Some(table) = parse_html_table(table_html) {
                        blocks.push(SectionBlock::Table(table));
                    }
                    cur = end_pos;
                } else {
                    cur = tag_start + 6;
                }
            }
            "figure" => {
                let close_str = "</figure>";
                if let Some(rel_close) = lower[tag_start..].find(close_str) {
                    let end_pos = tag_start + rel_close + close_str.len();
                    let fig_html = &html[tag_start..end_pos];
                    if let Some(img) = parse_html_figure(fig_html) {
                        blocks.push(SectionBlock::Image(img));
                    }
                    cur = end_pos;
                } else {
                    cur = tag_start + 7;
                }
            }
            "img" => {
                if let Some(rel_close) = lower[tag_start..].find('>') {
                    let end_pos = tag_start + rel_close + 1;
                    let img_html = &html[tag_start..end_pos];
                    if let Some(img) = parse_html_img(img_html) {
                        blocks.push(SectionBlock::Image(img));
                    }
                    cur = end_pos;
                } else {
                    cur = tag_start + 4;
                }
            }
            _ => {
                cur = tag_start + 1;
            }
        }
    }

    blocks
}

/// Parse HTML table into a clean ReadingTable
pub fn parse_html_table(table_html: &str) -> Option<ReadingTable> {
    let lower = table_html.to_ascii_lowercase();

    // 1. Caption
    let mut caption = None;
    if let Some(cap_start) = lower.find("<caption>") {
        if let Some(cap_end) = lower[cap_start..].find("</caption>") {
            let cap_raw = &table_html[cap_start + 9..cap_start + cap_end];
            let cap_clean = strip_html_tags(cap_raw).trim().to_string();
            if !cap_clean.is_empty() {
                let cleaned_lines: Vec<&str> = cap_clean.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                caption = Some(cleaned_lines.join(": "));
            }
        }
    }

    // 2. Extract rows and cells
    let mut raw_rows: Vec<Vec<String>> = Vec::new();
    let mut cur_search = 0;

    while let Some(tr_start_rel) = lower[cur_search..].find("<tr") {
        let tr_start = cur_search + tr_start_rel;
        let tr_tag_end = match lower[tr_start..].find('>') {
            Some(idx) => tr_start + idx + 1,
            None => break,
        };
        let tr_close = match lower[tr_tag_end..].find("</tr>") {
            Some(idx) => tr_tag_end + idx,
            None => break,
        };

        let tr_body = &table_html[tr_tag_end..tr_close];
        let mut cells = Vec::new();
        let mut cur_cell = 0;
        let tr_lower = tr_body.to_ascii_lowercase();

        while cur_cell < tr_body.len() {
            let td_pos = tr_lower[cur_cell..].find("<td").map(|i| cur_cell + i);
            let th_pos = tr_lower[cur_cell..].find("<th").map(|i| cur_cell + i);

            let cell_start = match (td_pos, th_pos) {
                (Some(a), Some(b)) => a.min(b),
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (None, None) => break,
            };

            let is_th = tr_lower[cell_start..].starts_with("<th");
            let close_tag = if is_th { "</th>" } else { "</td>" };

            let cell_tag_end = match tr_lower[cell_start..].find('>') {
                Some(idx) => cell_start + idx + 1,
                None => break,
            };

            let cell_close = match tr_lower[cell_tag_end..].find(close_tag) {
                Some(idx) => cell_tag_end + idx,
                None => break,
            };

            let cell_raw = &tr_body[cell_tag_end..cell_close];
            let cell_text = strip_html_tags(cell_raw).trim().to_string();

            // Check colspan
            let tag_str = &tr_body[cell_start..cell_tag_end].to_ascii_lowercase();
            let colspan = if let Some(cs_pos) = tag_str.find("colspan=") {
                let after = &tag_str[cs_pos + 8..];
                let num_str: String = after
                    .chars()
                    .skip_while(|c| *c == '"' || *c == '\'')
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                num_str.parse::<usize>().unwrap_or(1).max(1)
            } else {
                1
            };

            cells.push((cell_text, colspan));
            cur_cell = cell_close + close_tag.len();
        }

        if !cells.is_empty() {
            let mut flat_cells = Vec::new();
            for (text, span) in cells {
                flat_cells.push(text);
                for _ in 1..span {
                    flat_cells.push(String::new());
                }
            }
            raw_rows.push(flat_cells);
        }

        cur_search = tr_close + 5;
    }

    if raw_rows.is_empty() {
        return None;
    }

    let mut row_idx = 0;

    // Check if row 0 (and row 1) are table caption/number (e.g. TABLE 1-1)
    if caption.is_none() && !raw_rows.is_empty() {
        let first_non_empty: Vec<&str> = raw_rows[0]
            .iter()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim())
            .collect();

        if first_non_empty.len() == 1
            && first_non_empty[0].to_ascii_uppercase().starts_with("TABLE")
        {
            let cap_part1 = first_non_empty[0].to_string();
            row_idx += 1;
            if raw_rows.len() > row_idx {
                let second_non_empty: Vec<&str> = raw_rows[row_idx]
                    .iter()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim())
                    .collect();
                if second_non_empty.len() == 1 {
                    caption = Some(format!("{}: {}", cap_part1, second_non_empty[0]));
                    row_idx += 1;
                } else {
                    caption = Some(cap_part1);
                }
            } else {
                caption = Some(cap_part1);
            }
        }
    }

    // Check if last row is footer (Source: ... / Note: ...)
    let mut footer = None;
    if raw_rows.len() > row_idx {
        let last = &raw_rows[raw_rows.len() - 1];
        let non_empty: Vec<&str> = last
            .iter()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim())
            .collect();
        if non_empty.len() == 1 {
            let text = non_empty[0];
            if text.starts_with("Source:")
                || text.starts_with("Source :")
                || text.starts_with("Note:")
                || text.starts_with("Notes:")
            {
                footer = Some(text.to_string());
                raw_rows.pop();
            }
        }
    }

    let remaining_rows: Vec<Vec<String>> = raw_rows[row_idx..].to_vec();
    if remaining_rows.is_empty() {
        return Some(ReadingTable {
            caption,
            headers: Vec::new(),
            rows: Vec::new(),
            footer,
        });
    }

    let max_cols = remaining_rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut headers = Vec::new();
    let mut data_start = 0;

    // Detect multi-row headers (e.g. Table 1-1 with 2 header rows)
    if remaining_rows.len() >= 2 {
        let r0 = &remaining_rows[0];
        let r1 = &remaining_rows[1];
        let r0_non_empty = r0.iter().filter(|s| !s.trim().is_empty()).count();
        let r1_non_empty = r1.iter().filter(|s| !s.trim().is_empty()).count();

        if r0_non_empty >= 2 && r1_non_empty >= 2 && max_cols > r0_non_empty {
            // Merge subheaders into main headers
            let mut merged = Vec::new();
            let mut r1_iter = r1.iter().filter(|s| !s.trim().is_empty());
            for cell in r0 {
                if !cell.trim().is_empty() {
                    merged.push(cell.clone());
                } else if let Some(sub) = r1_iter.next() {
                    merged.push(sub.clone());
                }
            }
            if merged.len() == max_cols {
                headers = merged;
                data_start = 2;
            }
        }
    }

    if headers.is_empty() && !remaining_rows.is_empty() {
        headers = remaining_rows[0].clone();
        data_start = 1;
    }

    let rows: Vec<Vec<String>> = remaining_rows[data_start..]
        .iter()
        .map(|row| {
            let mut r = row.clone();
            while r.len() < headers.len() {
                r.push(String::new());
            }
            r
        })
        .collect();

    Some(ReadingTable {
        caption,
        headers,
        rows,
        footer,
    })
}

/// Parse HTML <figure> into ReadingImage
pub fn parse_html_figure(figure_html: &str) -> Option<ReadingImage> {
    let lower = figure_html.to_ascii_lowercase();
    let mut caption = None;

    if let Some(cap_start) = lower.find("<figcaption>") {
        if let Some(cap_end) = lower[cap_start..].find("</figcaption>") {
            let cap_raw = &figure_html[cap_start + 12..cap_start + cap_end];
            let cap_clean = strip_html_tags(cap_raw).trim().to_string();
            if !cap_clean.is_empty() {
                let parts: Vec<&str> = cap_clean
                    .lines()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                caption = Some(parts.join(": "));
            }
        }
    }

    if let Some(img_start) = lower.find("<img") {
        let img_end = lower[img_start..]
            .find('>')
            .map(|i| img_start + i + 1)
            .unwrap_or(figure_html.len());
        let img_tag = &figure_html[img_start..img_end];
        let (src, alt) = extract_img_attrs(img_tag);
        if let Some(image_id) = src {
            return Some(ReadingImage {
                image_id,
                caption,
                alt,
            });
        }
    }

    None
}

/// Parse standalone <img> tag into ReadingImage
pub fn parse_html_img(img_tag_html: &str) -> Option<ReadingImage> {
    let (src, alt) = extract_img_attrs(img_tag_html);
    src.map(|image_id| ReadingImage {
        image_id,
        caption: None,
        alt,
    })
}

fn extract_img_attrs(img_html: &str) -> (Option<String>, Option<String>) {
    let lower = img_html.to_ascii_lowercase();
    let mut src = None;
    let mut alt = None;

    if let Some(src_pos) = lower.find("src=") {
        let after = &img_html[src_pos + 4..];
        let quote = after.chars().next().unwrap_or(' ');
        let val = if quote == '"' || quote == '\'' {
            after[1..].split(quote).next().unwrap_or("").trim()
        } else {
            after.split_whitespace().next().unwrap_or("").trim()
        };
        let filename = Path::new(val)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(val);
        if !filename.is_empty() {
            src = Some(filename.to_string());
        }
    }

    if let Some(alt_pos) = lower.find("alt=") {
        let after = &img_html[alt_pos + 4..];
        let quote = after.chars().next().unwrap_or(' ');
        let val = if quote == '"' || quote == '\'' {
            after[1..].split(quote).next().unwrap_or("").trim()
        } else {
            after.split_whitespace().next().unwrap_or("").trim()
        };
        if !val.is_empty() {
            alt = Some(val.to_string());
        }
    }

    (src, alt)
}

/// Paginate structured blocks into comfortable pages (~250 words per page)
pub fn paginate_epub_blocks(
    chapter_title: &str,
    blocks: Vec<SectionBlock>,
    mut page_num: usize,
) -> (Vec<ExtractedSection>, usize) {
    let mut sections = Vec::new();
    let mut current_page_blocks: Vec<SectionBlock> = Vec::new();
    let mut current_page_words = 0;

    for block in blocks {
        let block_words = match &block {
            SectionBlock::Text(s) => s.split_whitespace().count(),
            SectionBlock::Table(t) => t.to_text().split_whitespace().count().min(220),
            SectionBlock::Image(_) => 10,
        };

        let is_standalone = matches!(block, SectionBlock::Table(_) | SectionBlock::Image(_));
        if (current_page_words + block_words > 260 && current_page_words >= 150)
            || (is_standalone && current_page_words >= 160)
        {
            if !current_page_blocks.is_empty() {
                let content = current_page_blocks
                    .iter()
                    .map(|b| match b {
                        SectionBlock::Text(s) => s.clone(),
                        SectionBlock::Table(t) => t.to_text(),
                        SectionBlock::Image(i) => i.to_text(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");

                sections.push(ExtractedSection::with_blocks(
                    format!("Page {} — {}", page_num, chapter_title),
                    content,
                    current_page_blocks,
                ));
                page_num += 1;
                current_page_blocks = Vec::new();
                current_page_words = 0;
            }
        }

        current_page_blocks.push(block);
        current_page_words += block_words;
    }

    if !current_page_blocks.is_empty() {
        let content = current_page_blocks
            .iter()
            .map(|b| match b {
                SectionBlock::Text(s) => s.clone(),
                SectionBlock::Table(t) => t.to_text(),
                SectionBlock::Image(i) => i.to_text(),
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        sections.push(ExtractedSection::with_blocks(
            format!("Page {} — {}", page_num, chapter_title),
            content,
            current_page_blocks,
        ));
        page_num += 1;
    }

    (sections, page_num)
}

/// Legacy text pagination helper (for raw text and backward compatibility)
pub fn paginate_epub_text(
    chapter_title: &str,
    clean_text: &str,
    mut page_num: usize,
) -> (Vec<ExtractedSection>, usize) {
    let mut sections = Vec::new();
    let raw_blocks: Vec<&str> = clean_text.split('\x0C').collect();

    for block in raw_blocks {
        let block_trimmed = block.trim();
        if block_trimmed.is_empty() {
            continue;
        }

        let paras = crate::reader::split_text_paragraphs(block_trimmed);
        if paras.is_empty() {
            continue;
        }

        let mut current_page_paras: Vec<String> = Vec::new();
        let mut current_page_word_count = 0;

        for para in paras {
            let words: Vec<&str> = para.split_whitespace().collect();
            let word_count = words.len();
            if word_count == 0 {
                continue;
            }

            if word_count > 350 {
                if !current_page_paras.is_empty() {
                    let page_content = current_page_paras.join("\n\n");
                    sections.push(ExtractedSection::new(
                        format!("Page {} — {}", page_num, chapter_title),
                        page_content,
                    ));
                    page_num += 1;
                    current_page_paras.clear();
                    current_page_word_count = 0;
                }

                let mut sentence_buf = Vec::new();
                let mut sentence_words = 0;
                for s in para.split_inclusive(|c: char| c == '.' || c == '!' || c == '?') {
                    let s_trimmed = s.trim();
                    if s_trimmed.is_empty() {
                        continue;
                    }
                    let count = s_trimmed.split_whitespace().count();
                    if sentence_words + count > 260 && !sentence_buf.is_empty() {
                        sections.push(ExtractedSection::new(
                            format!("Page {} — {}", page_num, chapter_title),
                            sentence_buf.join(" "),
                        ));
                        page_num += 1;
                        sentence_buf.clear();
                        sentence_words = 0;
                    }
                    sentence_buf.push(s_trimmed.to_string());
                    sentence_words += count;
                }
                if !sentence_buf.is_empty() {
                    current_page_paras.push(sentence_buf.join(" "));
                    current_page_word_count = sentence_words;
                }
                continue;
            }

            if current_page_word_count + word_count > 260 && current_page_word_count >= 160 {
                let page_content = current_page_paras.join("\n\n");
                sections.push(ExtractedSection::new(
                    format!("Page {} — {}", page_num, chapter_title),
                    page_content,
                ));
                page_num += 1;
                current_page_paras.clear();
                current_page_word_count = 0;
            }

            current_page_paras.push(para);
            current_page_word_count += word_count;
        }

        if !current_page_paras.is_empty() {
            let page_content = current_page_paras.join("\n\n");
            sections.push(ExtractedSection::new(
                format!("Page {} — {}", page_num, chapter_title),
                page_content,
            ));
            page_num += 1;
        }
    }

    if sections.is_empty() && !clean_text.trim().is_empty() {
        sections.push(ExtractedSection::new(
            format!("Page {} — {}", page_num, chapter_title),
            clean_text.trim().to_string(),
        ));
        page_num += 1;
    }

    (sections, page_num)
}

fn extract_heading(html: &str) -> Option<String> {
    for tag in ["h1", "h2", "title"] {
        let open_tag = format!("<{}", tag);
        let close_tag = format!("</{}>", tag);
        if let Some(start_pos) = html.to_ascii_lowercase().find(&open_tag) {
            if let Some(content_start) = html[start_pos..].find('>') {
                let actual_start = start_pos + content_start + 1;
                if let Some(end_pos) = html[actual_start..].to_ascii_lowercase().find(&close_tag) {
                    let heading_raw = &html[actual_start..actual_start + end_pos];
                    let clean = strip_html_tags(heading_raw).trim().to_string();
                    if !clean.is_empty() && clean.len() < 120 {
                        return Some(clean);
                    }
                }
            }
        }
    }
    None
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
            let mut tag_full = String::new();
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

            while let Some(&tc) = chars.peek() {
                if tc == '>' {
                    break;
                }
                tag_full.push(tc.to_ascii_lowercase());
                chars.next();
            }

            if tag_name == "script" || tag_name == "style" || tag_name == "head" {
                in_script_or_style = !is_closing;
            }

            if !in_script_or_style && !is_closing {
                let is_pagebreak = tag_full.contains("pagebreak")
                    || tag_full.contains("page-break")
                    || tag_name == "hr";

                if is_pagebreak {
                    result.push('\x0C');
                } else if tag_name == "p"
                    || tag_name == "div"
                    || tag_name == "h1"
                    || tag_name == "h2"
                    || tag_name == "h3"
                    || tag_name == "h4"
                    || tag_name == "h5"
                    || tag_name == "h6"
                    || tag_name == "li"
                    || tag_name == "tr"
                {
                    result.push_str("\n\n");
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
