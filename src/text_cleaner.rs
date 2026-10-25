pub fn clean_text_for_reading(input: &str) -> String {
    // Phase 1: Normalize Unicode hyphens, soft hyphens, and replacement characters
    let mut step1 = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];
        match c {
            // Soft hyphen or replacement char embedded in a word
            '\u{00AD}' | '\u{FFFD}' | '\u{0002}' | '\u{001F}' => {
                let prev_is_alpha = i > 0 && chars[i - 1].is_alphabetic();
                let next_is_alpha = i + 1 < len && chars[i + 1].is_alphabetic();
                if prev_is_alpha && next_is_alpha {
                    // Skip/drop the soft hyphen so letters join
                } else if !c.is_alphanumeric() {
                    step1.push(' ');
                }
            }
            '\u{200B}' | '\u{200C}' | '\u{200D}' => {
                // Zero-width spaces: drop
            }
            '\u{00A0}' => {
                step1.push(' ');
            }
            '\u{2010}' | '\u{2011}' => {
                step1.push('-');
            }
            _ => {
                step1.push(c);
            }
        }
        i += 1;
    }

    // Phase 2: Handle hyphenated line-breaks
    let mut step2 = String::with_capacity(step1.len());
    let raw_lines: Vec<&str> = step1.lines().collect();
    let mut line_idx = 0;

    while line_idx < raw_lines.len() {
        let current_line = raw_lines[line_idx].trim();
        if current_line.is_empty() {
            step2.push('\n');
            line_idx += 1;
            continue;
        }

        // Check if line ends with a hyphen attached to a word
        if let Some(without_hyphen) = current_line.strip_suffix('-') {
            let last_char = without_hyphen.chars().last().unwrap_or(' ');
            if last_char.is_alphabetic() && line_idx + 1 < raw_lines.len() {
                let next_line = raw_lines[line_idx + 1].trim();
                if let Some(first_space_idx) = next_line.find(char::is_whitespace) {
                    let first_word = &next_line[..first_space_idx];
                    let remainder = &next_line[first_space_idx..].trim_start();

                    if first_word.chars().all(|ch| ch.is_alphabetic()) {
                        // Merge prefix and first word
                        step2.push_str(without_hyphen);
                        step2.push_str(first_word);
                        step2.push(' ');
                        step2.push_str(remainder);
                        step2.push('\n');
                        line_idx += 2; // Consumed next line!
                        continue;
                    }
                } else if !next_line.is_empty() && next_line.chars().all(|ch| ch.is_alphabetic()) {
                    // Entire next line is just the second half of the word
                    step2.push_str(without_hyphen);
                    step2.push_str(next_line);
                    step2.push('\n');
                    line_idx += 2; // Consumed next line!
                    continue;
                }
            }
        }

        step2.push_str(current_line);
        step2.push('\n');
        line_idx += 1;
    }

    // Phase 3: Separate glued words after punctuation and camelCase merges
    let mut step3 = String::with_capacity(step2.len() + 100);
    let chars3: Vec<char> = step2.chars().collect();
    let len3 = chars3.len();
    let mut j = 0;

    while j < len3 {
        let ch = chars3[j];
        step3.push(ch);

        if j + 1 < len3 {
            let next_ch = chars3[j + 1];

            // Rule 3A: Sentence ending punctuation without space: . ! ? followed by uppercase letter or quote
            if (ch == '.' || ch == '!' || ch == '?') && (next_ch.is_ascii_uppercase() || next_ch == '"' || next_ch == '“') {
                // Check if part of a known abbreviation like "U.S." or "e.g." or "i.e."
                if !is_abbreviation(&chars3, j) {
                    step3.push(' ');
                }
            }
            // Rule 3B: Comma, colon, semicolon without space: , ; : followed by letter or quote
            else if (ch == ',' || ch == ';' || ch == ':') && (next_ch.is_alphabetic() || next_ch == '"' || next_ch == '“') {
                // Protect numbers like 1,000 or time like 12:30
                let prev_is_digit = j > 0 && chars3[j - 1].is_ascii_digit();
                let next_is_digit = next_ch.is_ascii_digit();
                if !(prev_is_digit && next_is_digit) {
                    step3.push(' ');
                }
            }
            // Rule 3C: Lowercase letter immediately followed by uppercase letter (camelCase glue from PDF)
            // e.g. "economyWithout", "corporationAnd"
            else if ch.is_ascii_lowercase() && next_ch.is_ascii_uppercase() {
                // Ensure not preceded by uppercase (like McDonald)
                let prev_is_upper = j > 0 && chars3[j - 1].is_ascii_uppercase();
                if !prev_is_upper {
                    step3.push(' ');
                }
            }
        }

        j += 1;
    }

    step3
}

/// Helper to avoid splitting abbreviations like "U.S.", "e.g.", "i.e."
fn is_abbreviation(chars: &[char], period_idx: usize) -> bool {
    // Check if preceded by single uppercase letter (e.g. U.S.)
    if period_idx > 0 && chars[period_idx - 1].is_ascii_uppercase() {
        if period_idx >= 2 && chars[period_idx - 2] == '.' {
            return true;
        }
        if period_idx + 2 < chars.len() && chars[period_idx + 1].is_ascii_uppercase() && chars[period_idx + 2] == '.' {
            return true;
        }
    }

    // Check "e.g." or "i.e." or "vs."
    let start = period_idx.saturating_sub(4);
    let slice: String = chars[start..=period_idx].iter().collect();
    let lower = slice.to_ascii_lowercase();
    if lower.ends_with("e.g.") || lower.ends_with("i.e.") || lower.ends_with("vs.") || lower.ends_with("dr.") || lower.ends_with("mr.") || lower.ends_with("mrs.") || lower.ends_with("etc.") {
        return true;
    }

    false
}
