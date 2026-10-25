use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::parser::ExtractedSection;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSection {
    pub title: String,
    pub start_word_index: usize,
    pub start_line_index: usize,
    pub word_count: usize,
    pub line_count: usize,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub title: String,
    pub file_path: Option<PathBuf>,
    pub tokens: Vec<Token>,
    pub lines: Vec<ReadingLine>,
    pub sections: Vec<DocumentSection>,
    pub total_words: usize,
    pub total_lines: usize,
}

impl Document {
    pub fn from_sections(
        title: String,
        file_path: Option<PathBuf>,
        extracted_sections: Vec<ExtractedSection>,
        words_per_line: usize,
    ) -> Self {
        let mut tokens = Vec::new();
        let mut lines = Vec::new();
        let mut sections = Vec::new();

        for sec in extracted_sections {
            let start_word_idx = tokens.len();
            let start_line_idx = lines.len();

            let cleaned = crate::text_cleaner::clean_text_for_reading(&sec.content);
            let paragraphs: Vec<&str> = cleaned.split('\n').collect();

            for paragraph in paragraphs {
                let words: Vec<&str> = paragraph.split_whitespace().collect();
                if words.is_empty() {
                    continue;
                }

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

                        lines.push(ReadingLine {
                            text: line_text,
                            word_indices: line_start_idx..line_end_idx,
                            pause_multiplier: pause_mult,
                            is_paragraph_start: lines.is_empty() || lines.len() == start_line_idx,
                        });

                        current_line_words.clear();
                        line_start_idx = line_end_idx;
                    }
                }
            }

            let word_count = tokens.len() - start_word_idx;
            let line_count = lines.len() - start_line_idx;

            if word_count > 0 {
                sections.push(DocumentSection {
                    title: sec.title,
                    start_word_index: start_word_idx,
                    start_line_index: start_line_idx,
                    word_count,
                    line_count,
                });
            }
        }

        // If no sections were created (e.g. empty content), make a default one
        if sections.is_empty() {
            sections.push(DocumentSection {
                title: "Introduction".to_string(),
                start_word_index: 0,
                start_line_index: 0,
                word_count: tokens.len(),
                line_count: lines.len(),
            });
        }

        let total_words = tokens.len();
        let total_lines = lines.len();

        Self {
            title,
            file_path,
            tokens,
            lines,
            sections,
            total_words,
            total_lines,
        }
    }

    pub fn from_text(title: String, file_path: Option<PathBuf>, raw_text: &str, words_per_line: usize) -> Self {
        let extracted = vec![ExtractedSection {
            title: "Main Content".to_string(),
            content: raw_text.to_string(),
        }];
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

    /// Sample demo document with chapters for immediate testing
    pub fn demo_document(words_per_line: usize) -> Self {
        let title = "Speed Reader: The Distraction-Free Guide".to_string();
        let sections = vec![
            ExtractedSection {
                title: "Chapter 1: The Challenge of Wandering Attention".to_string(),
                content: r#"Welcome to your new distraction-free speed reader.

If you struggle with focus and find yourself reading the same sentence three times, this tool was built specifically for you.

Traditional reading forces your eyes to jump back and forth across wide margins. Each jump is an opportunity for your mind to wander, get distracted, or lose track of where you were."#.to_string(),
            },
            ExtractedSection {
                title: "Chapter 2: How Line-by-Line Pacing Works".to_string(),
                content: r#"With Line-by-Line mode, you see only one focused thought at a time. The world fades away, leaving just you and the active words.

Notice the gentle context lines above and below. They keep you anchored in the story without distracting your central vision.

Each line is paced according to natural thought clauses, pausing longer at periods and commas so your mind retains what it reads."#.to_string(),
            },
            ExtractedSection {
                title: "Chapter 3: Optimal Recognition Points & RSVP".to_string(),
                content: r#"With RSVP mode, words flash in the exact same spot at an adjustable pace.

An Optimal Recognition Point anchors your vision between vertical alignment ticks so your eyes never need to move.

Take control of your pace. Press Space to play or pause. Use the Up and Down arrow keys to adjust your speed anytime."#.to_string(),
            },
            ExtractedSection {
                title: "Chapter 4: Navigation, Bookmarks & Zen Focus".to_string(),
                content: r#"Press F or F11 at any time to enter Zen Focus Mode. All buttons and bars vanish, leaving pure reading bliss.

Use the chapter navigator to jump to any page or chapter instantly.

Drop any PDF, EPUB, or TXT file into this window to begin your journey."#.to_string(),
            },
        ];

        Self::from_sections(title, None, sections, words_per_line)
    }
}
