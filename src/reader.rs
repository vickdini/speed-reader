use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub struct Document {
    pub title: String,
    pub file_path: Option<PathBuf>,
    pub tokens: Vec<Token>,
    pub lines: Vec<ReadingLine>,
    pub total_words: usize,
    pub total_lines: usize,
}

impl Document {
    pub fn from_text(title: String, file_path: Option<PathBuf>, raw_text: &str, words_per_line: usize) -> Self {
        let mut tokens = Vec::new();
        let mut lines = Vec::new();

        let paragraphs: Vec<&str> = raw_text.split('\n').collect();

        for paragraph in paragraphs {
            let words: Vec<&str> = paragraph.split_whitespace().collect();
            if words.is_empty() {
                continue;
            }

            let num_words = words.len();
            let mut current_line_words = Vec::new();
            let mut line_start_idx = tokens.len();
            let is_p_start = true;

            for (i, &word) in words.iter().enumerate() {
                let is_last_word_of_p = i == num_words - 1;
                let token = Token::new(word, is_last_word_of_p);
                tokens.push(token);
                current_line_words.push(word);

                let reached_chunk_size = current_line_words.len() >= words_per_line;
                let is_sentence_end = word.ends_with('.') || word.ends_with('!') || word.ends_with('?');
                
                // Break line on chunk size or on sentence boundaries if chunk size is near
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
                        is_paragraph_start: is_p_start && lines.is_empty(),
                    });

                    current_line_words.clear();
                    line_start_idx = line_end_idx;
                }
            }
        }

        let total_words = tokens.len();
        let total_lines = lines.len();

        Self {
            title,
            file_path,
            tokens,
            lines,
            total_words,
            total_lines,
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

    /// Sample demo document for testing immediately without loading a file
    pub fn demo_document(words_per_line: usize) -> Self {
        let title = "Speed Reader: The Distraction-Free Guide".to_string();
        let text = r#"Welcome to your new distraction-free speed reader.

If you struggle with focus and find yourself reading the same sentence three times, this tool was built specifically for you.

Traditional reading forces your eyes to jump back and forth across wide margins. Each jump is an opportunity for your mind to wander, get distracted, or lose track of where you were.

With Line-by-Line mode, you see only one focused thought at a time. The world fades away, leaving just you and the words.

With RSVP mode, words appear in the exact same spot at an adjustable pace. An Optimal Recognition Point anchors your vision so your eyes never need to move.

Take control of your pace. Press Space to play or pause. Use the Up and Down arrow keys to adjust your speed anytime.

Drop any PDF, EPUB, or TXT file into this window to begin your journey."#;

        Self::from_text(title, None, text, words_per_line)
    }
}
