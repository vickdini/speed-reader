use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingTable {
    pub caption: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub footer: Option<String>,
}

impl ReadingTable {
    pub fn to_text(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref cap) = self.caption {
            parts.push(cap.clone());
        }
        if !self.headers.is_empty() {
            parts.push(self.headers.join(" | "));
        }
        for row in &self.rows {
            parts.push(row.join(" | "));
        }
        if let Some(ref foot) = self.footer {
            parts.push(foot.clone());
        }
        parts.join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingImage {
    pub image_id: String,
    pub caption: Option<String>,
    pub alt: Option<String>,
}

impl ReadingImage {
    pub fn to_text(&self) -> String {
        let desc = self.caption.as_deref().or(self.alt.as_deref()).unwrap_or("Illustration");
        format!("[Illustration: {}]", desc)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionBlock {
    Text(String),
    Table(ReadingTable),
    Image(ReadingImage),
}

#[derive(Debug, Clone)]
pub struct ExtractedSection {
    pub title: String,
    pub content: String,
    pub blocks: Vec<SectionBlock>,
}

impl ExtractedSection {
    pub fn new(title: String, content: String) -> Self {
        Self {
            title,
            content,
            blocks: Vec::new(),
        }
    }

    pub fn with_blocks(title: String, content: String, blocks: Vec<SectionBlock>) -> Self {
        Self {
            title,
            content,
            blocks,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: String,
    pub sections: Vec<ExtractedSection>,
    pub images: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub raw: String,
    pub orp_idx: usize,
    pub pause_multiplier: f32,
    pub is_paragraph_break: bool,
}

impl Token {
    pub fn new(raw: &str, is_paragraph_break: bool) -> Self {
        let orp_idx = calculate_orp(raw);
        let pause_multiplier = calculate_pause_multiplier(raw, is_paragraph_break);

        Self {
            raw: raw.to_string(),
            orp_idx,
            pause_multiplier,
            is_paragraph_break,
        }
    }

    /// Split raw token into: prefix (before ORP), ORP character, suffix (after ORP)
    pub fn split_orp(&self) -> (&str, &str, &str) {
        let char_indices: Vec<(usize, char)> = self.raw.char_indices().collect();
        if char_indices.is_empty() {
            return ("", "", "");
        }

        let idx = self.orp_idx.min(char_indices.len().saturating_sub(1));
        let (byte_start, ch) = char_indices[idx];
        let byte_end = byte_start + ch.len_utf8();

        let prefix = &self.raw[..byte_start];
        let orp_char = &self.raw[byte_start..byte_end];
        let suffix = &self.raw[byte_end..];

        (prefix, orp_char, suffix)
    }
}

/// Calculate the Optimal Recognition Point (ORP) index (0-indexed char position)
pub fn calculate_orp(word: &str) -> usize {
    let char_count = word.chars().count();
    match char_count {
        0 | 1 => 0,
        2..=5 => 1,
        6..=9 => 2,
        10..=13 => 3,
        _ => 4,
    }
}

fn calculate_pause_multiplier(word: &str, is_paragraph_break: bool) -> f32 {
    let trimmed = word.trim_end_matches(|c: char| c == '"' || c == '\'' || c == ')' || c == '»');
    let mut multiplier: f32 = 1.0;

    if is_paragraph_break {
        multiplier = 2.4;
    } else if trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') || trimmed.ends_with("...") {
        multiplier = 2.2;
    } else if trimmed.ends_with(',') || trimmed.ends_with(';') || trimmed.ends_with(':') || trimmed.ends_with('—') || trimmed.ends_with('-') {
        multiplier = 1.5;
    }

    if word.chars().count() > 10 {
        multiplier *= 1.15;
    }

    multiplier
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadingLine {
    pub text: String,
    pub word_indices: std::ops::Range<usize>,
    pub pause_multiplier: f32,
    pub is_paragraph_start: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadingParagraph {
    pub text: String,
    pub word_indices: std::ops::Range<usize>,
    pub line_indices: std::ops::Range<usize>,
    pub pause_multiplier: f32,
    pub table: Option<ReadingTable>,
    pub image: Option<ReadingImage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSection {
    pub title: String,
    pub start_word_index: usize,
    pub start_line_index: usize,
    #[serde(default)]
    pub start_paragraph_index: usize,
    pub word_count: usize,
    pub line_count: usize,
    #[serde(default)]
    pub paragraph_count: usize,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub title: String,
    pub file_path: Option<PathBuf>,
    pub tokens: Vec<Token>,
    pub lines: Vec<ReadingLine>,
    pub paragraphs: Vec<ReadingParagraph>,
    pub sections: Vec<DocumentSection>,
    pub total_words: usize,
    pub total_lines: usize,
    pub total_paragraphs: usize,
    pub images: HashMap<String, Vec<u8>>,
}

/// Helper function to split cleaned text into distinct paragraph blocks
pub fn split_text_paragraphs(cleaned: &str) -> Vec<String> {
    let mut raw_paras: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in cleaned.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current.trim().is_empty() {
                raw_paras.push(current.trim().to_string());
                current.clear();
            }
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(trimmed);
        }
    }
    if !current.trim().is_empty() {
        raw_paras.push(current.trim().to_string());
    }

    raw_paras
}

impl Document {
    pub fn from_sections(
        title: String,
        file_path: Option<PathBuf>,
        extracted_sections: Vec<ExtractedSection>,
        words_per_line: usize,
    ) -> Self {
        Self::from_parsed_document(
            ParsedDocument {
                title,
                sections: extracted_sections,
                images: HashMap::new(),
            },
            file_path,
            words_per_line,
        )
    }

    pub fn from_parsed_document(
        parsed: ParsedDocument,
        file_path: Option<PathBuf>,
        words_per_line: usize,
    ) -> Self {
        let mut tokens = Vec::new();
        let mut lines = Vec::new();
        let mut paragraphs = Vec::new();
        let mut sections = Vec::new();

        for sec in parsed.sections {
            let start_word_idx = tokens.len();
            let start_line_idx = lines.len();
            let start_para_idx = paragraphs.len();

            let blocks = if !sec.blocks.is_empty() {
                sec.blocks
            } else {
                let cleaned = crate::text_cleaner::clean_text_for_reading(&sec.content);
                split_text_paragraphs(&cleaned)
                    .into_iter()
                    .map(SectionBlock::Text)
                    .collect()
            };

            for block in blocks {
                match block {
                    SectionBlock::Text(paragraph_text) => {
                        let words: Vec<&str> = paragraph_text.split_whitespace().collect();
                        if words.is_empty() {
                            continue;
                        }

                        let para_start_word_idx = tokens.len();
                        let para_start_line_idx = lines.len();
                        let num_words = words.len();
                        let mut current_line_words = Vec::new();
                        let mut line_start_idx = tokens.len();

                        for (i, &word) in words.iter().enumerate() {
                            let is_last_word_of_p = i == num_words - 1;
                            let token = Token::new(word, is_last_word_of_p);
                            tokens.push(token);
                            current_line_words.push(word);

                            let reached_chunk_size = current_line_words.len() >= words_per_line;
                            let is_sentence_end = word.ends_with('.') || word.ends_with('!') || word.ends_with('?');

                            let should_break_line = is_last_word_of_p
                                || reached_chunk_size
                                || (current_line_words.len() >= words_per_line.saturating_sub(2) && is_sentence_end);

                            if should_break_line {
                                let line_end_idx = tokens.len();
                                let line_text = current_line_words.join(" ");
                                let pause_mult = if is_last_word_of_p {
                                    2.4
                                } else if is_sentence_end {
                                    2.0
                                } else if word.ends_with(',') || word.ends_with(';') {
                                    1.4
                                } else {
                                    1.0
                                };

                                let is_para_start = lines.len() == para_start_line_idx;
                                lines.push(ReadingLine {
                                    text: line_text,
                                    word_indices: line_start_idx..line_end_idx,
                                    pause_multiplier: pause_mult,
                                    is_paragraph_start: is_para_start,
                                });

                                current_line_words.clear();
                                line_start_idx = line_end_idx;
                            }
                        }

                        let para_end_word_idx = tokens.len();
                        let para_end_line_idx = lines.len();

                        paragraphs.push(ReadingParagraph {
                            text: paragraph_text,
                            word_indices: para_start_word_idx..para_end_word_idx,
                            line_indices: para_start_line_idx..para_end_line_idx,
                            pause_multiplier: 1.15,
                            table: None,
                            image: None,
                        });
                    }
                    SectionBlock::Table(table) => {
                        let table_text = table.to_text();
                        let words: Vec<&str> = table_text.split_whitespace().collect();
                        let para_start_word_idx = tokens.len();
                        let para_start_line_idx = lines.len();
                        let num_words = words.len();
                        let mut current_line_words = Vec::new();
                        let mut line_start_idx = tokens.len();

                        if num_words > 0 {
                            for (i, &word) in words.iter().enumerate() {
                                let is_last_word_of_p = i == num_words - 1;
                                let token = Token::new(word, is_last_word_of_p);
                                tokens.push(token);
                                current_line_words.push(word);

                                if is_last_word_of_p || current_line_words.len() >= words_per_line {
                                    let line_end_idx = tokens.len();
                                    let line_text = current_line_words.join(" ");
                                    let is_para_start = lines.len() == para_start_line_idx;
                                    lines.push(ReadingLine {
                                        text: line_text,
                                        word_indices: line_start_idx..line_end_idx,
                                        pause_multiplier: 1.5,
                                        is_paragraph_start: is_para_start,
                                    });
                                    current_line_words.clear();
                                    line_start_idx = line_end_idx;
                                }
                            }
                        } else {
                            let token = Token::new("[Table]", true);
                            tokens.push(token);
                            lines.push(ReadingLine {
                                text: "[Table]".to_string(),
                                word_indices: para_start_word_idx..tokens.len(),
                                pause_multiplier: 1.5,
                                is_paragraph_start: true,
                            });
                        }

                        let para_end_word_idx = tokens.len();
                        let para_end_line_idx = lines.len();

                        paragraphs.push(ReadingParagraph {
                            text: table_text,
                            word_indices: para_start_word_idx..para_end_word_idx,
                            line_indices: para_start_line_idx..para_end_line_idx,
                            pause_multiplier: 1.5,
                            table: Some(table),
                            image: None,
                        });
                    }
                    SectionBlock::Image(image) => {
                        let img_text = image.to_text();
                        let words: Vec<&str> = img_text.split_whitespace().collect();
                        let para_start_word_idx = tokens.len();
                        let para_start_line_idx = lines.len();
                        let num_words = words.len();
                        let mut current_line_words = Vec::new();
                        let mut line_start_idx = tokens.len();

                        if num_words > 0 {
                            for (i, &word) in words.iter().enumerate() {
                                let is_last_word_of_p = i == num_words - 1;
                                let token = Token::new(word, is_last_word_of_p);
                                tokens.push(token);
                                current_line_words.push(word);

                                if is_last_word_of_p || current_line_words.len() >= words_per_line {
                                    let line_end_idx = tokens.len();
                                    let line_text = current_line_words.join(" ");
                                    let is_para_start = lines.len() == para_start_line_idx;
                                    lines.push(ReadingLine {
                                        text: line_text,
                                        word_indices: line_start_idx..line_end_idx,
                                        pause_multiplier: 1.5,
                                        is_paragraph_start: is_para_start,
                                    });
                                    current_line_words.clear();
                                    line_start_idx = line_end_idx;
                                }
                            }
                        } else {
                            let token = Token::new("[Image]", true);
                            tokens.push(token);
                            lines.push(ReadingLine {
                                text: "[Image]".to_string(),
                                word_indices: para_start_word_idx..tokens.len(),
                                pause_multiplier: 1.5,
                                is_paragraph_start: true,
                            });
                        }

                        let para_end_word_idx = tokens.len();
                        let para_end_line_idx = lines.len();

                        paragraphs.push(ReadingParagraph {
                            text: img_text,
                            word_indices: para_start_word_idx..para_end_word_idx,
                            line_indices: para_start_line_idx..para_end_line_idx,
                            pause_multiplier: 1.5,
                            table: None,
                            image: Some(image),
                        });
                    }
                }
            }

            let word_count = tokens.len() - start_word_idx;
            let line_count = lines.len() - start_line_idx;
            let para_count = paragraphs.len() - start_para_idx;

            if word_count > 0 || para_count > 0 {
                sections.push(DocumentSection {
                    title: sec.title,
                    start_word_index: start_word_idx,
                    start_line_index: start_line_idx,
                    start_paragraph_index: start_para_idx,
                    word_count,
                    line_count,
                    paragraph_count: para_count,
                });
            }
        }

        // If no sections were created (e.g. empty content), make a default one
        if sections.is_empty() {
            sections.push(DocumentSection {
                title: "Introduction".to_string(),
                start_word_index: 0,
                start_line_index: 0,
                start_paragraph_index: 0,
                word_count: tokens.len(),
                line_count: lines.len(),
                paragraph_count: paragraphs.len(),
            });
        }

        let total_words = tokens.len();
        let total_lines = lines.len();
        let total_paragraphs = paragraphs.len();

        Self {
            title: parsed.title,
            file_path,
            tokens,
            lines,
            paragraphs,
            sections,
            total_words,
            total_lines,
            total_paragraphs,
            images: parsed.images,
        }
    }

    pub fn from_text(title: String, file_path: Option<PathBuf>, raw_text: &str, words_per_line: usize) -> Self {
        let extracted = vec![ExtractedSection::new(
            "Main Content".to_string(),
            raw_text.to_string(),
        )];
        Self::from_sections(title, file_path, extracted, words_per_line)
    }

    /// Find the section index that contains the given word index
    pub fn current_section_index(&self, word_idx: usize) -> usize {
        for (i, sec) in self.sections.iter().enumerate().rev() {
            if word_idx >= sec.start_word_index {
                return i;
            }
        }
        0
    }

    /// Return reference to current section
    pub fn current_section(&self, word_idx: usize) -> Option<&DocumentSection> {
        let idx = self.current_section_index(word_idx);
        self.sections.get(idx)
    }

    /// Returns "Page" if the document is structured as pages (PDF), or "Chapter"
    pub fn section_type_label(&self) -> &'static str {
        if self.sections.iter().any(|s| s.title.starts_with("Page ")) {
            "Page"
        } else {
            "Chapter"
        }
    }

    /// Find the corresponding line index for a given word index
    pub fn line_for_word(&self, word_idx: usize) -> usize {
        for (i, line) in self.lines.iter().enumerate() {
            if line.word_indices.contains(&word_idx) {
                return i;
            }
        }
        self.lines.len().saturating_sub(1)
    }

    /// Find the start word index for a given line index
    pub fn word_for_line(&self, line_idx: usize) -> usize {
        if line_idx < self.lines.len() {
            self.lines[line_idx].word_indices.start
        } else {
            self.tokens.len().saturating_sub(1)
        }
    }

    /// Find the corresponding paragraph index for a given word index
    pub fn paragraph_for_word(&self, word_idx: usize) -> usize {
        for (i, para) in self.paragraphs.iter().enumerate() {
            if para.word_indices.contains(&word_idx) {
                return i;
            }
        }
        self.paragraphs.len().saturating_sub(1)
    }

    /// Find the start word index for a given paragraph index
    pub fn word_for_paragraph(&self, para_idx: usize) -> usize {
        if para_idx < self.paragraphs.len() {
            self.paragraphs[para_idx].word_indices.start
        } else {
            self.tokens.len().saturating_sub(1)
        }
    }

    /// Find the corresponding paragraph index for a given line index
    pub fn paragraph_for_line(&self, line_idx: usize) -> usize {
        for (i, para) in self.paragraphs.iter().enumerate() {
            if para.line_indices.contains(&line_idx) {
                return i;
            }
        }
        self.paragraphs.len().saturating_sub(1)
    }

    /// Find the start line index for a given paragraph index
    pub fn line_for_paragraph(&self, para_idx: usize) -> usize {
        if para_idx < self.paragraphs.len() {
            self.paragraphs[para_idx].line_indices.start
        } else {
            self.lines.len().saturating_sub(1)
        }
    }

    /// Sample demo document with chapters for immediate testing
    pub fn demo_document(words_per_line: usize) -> Self {
        let title = "Speed Reader: The Distraction-Free Guide".to_string();
        let sections = vec![
            ExtractedSection::new(
                "Chapter 1: The Challenge of Wandering Attention".to_string(),
                r#"Welcome to your new distraction-free speed reader.

If you struggle with focus and find yourself reading the same sentence three times, this tool was built specifically for you.

Traditional reading forces your eyes to jump back and forth across wide margins. Each jump is an opportunity for your mind to wander, get distracted, or lose track of where you were."#.to_string(),
            ),
            ExtractedSection::new(
                "Chapter 2: Line-by-Line & Paragraph Pacing".to_string(),
                r#"With Line-by-Line mode, you see only one focused thought at a time. The world fades away, leaving just you and the active words.

With Paragraph mode, you absorb whole cohesive thoughts with elegant context above and below, pacing effortlessly through literature.

Press M to toggle between Line, Paragraph, and RSVP modes anytime. Each line or paragraph pauses naturally according to punctuation and comprehension."#.to_string(),
            ),
            ExtractedSection::new(
                "Chapter 3: Optimal Recognition Points & RSVP".to_string(),
                r#"With RSVP mode, words flash in the exact same spot at an adjustable pace.

An Optimal Recognition Point anchors your vision between vertical alignment ticks so your eyes never need to move.

Take control of your pace. Press Space to play or pause. Use the Up and Down arrow keys to adjust your speed anytime."#.to_string(),
            ),
            ExtractedSection::new(
                "Chapter 4: Navigation, Bookmarks & Zen Focus".to_string(),
                r#"Press F or F11 at any time to enter Zen Focus Mode. All buttons and bars vanish, leaving pure reading bliss.

Use the chapter navigator to jump to any page or chapter instantly.

Drop any PDF, EPUB, or TXT file into this window to begin your journey."#.to_string(),
            ),
        ];

        Self::from_sections(title, None, sections, words_per_line)
    }
}
