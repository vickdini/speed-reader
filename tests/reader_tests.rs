use speed_reader::reader::{calculate_orp, Document, Token};

#[test]
fn test_orp_calculation() {
    // 0-1 chars -> index 0
    assert_eq!(calculate_orp("a"), 0);
    assert_eq!(calculate_orp("I"), 0);

    // 2-5 chars -> index 1
    assert_eq!(calculate_orp("the"), 1);
    assert_eq!(calculate_orp("book"), 1);
    assert_eq!(calculate_orp("reads"), 1);

    // 6-9 chars -> index 2
    assert_eq!(calculate_orp("reader"), 2);
    assert_eq!(calculate_orp("science"), 2);
    assert_eq!(calculate_orp("focuses"), 2);

    // 10-13 chars -> index 3
    assert_eq!(calculate_orp("distraction"), 3);
    assert_eq!(calculate_orp("comprehend"), 3);

    // > 13 chars -> index 4
    assert_eq!(calculate_orp("comprehensiveness"), 4);
}

#[test]
fn test_orp_splitting() {
    let tok = Token::new("focus", false);
    let (prefix, orp_char, suffix) = tok.split_orp();
    assert_eq!(prefix, "f");
    assert_eq!(orp_char, "o");
    assert_eq!(suffix, "cus");
}

#[test]
fn test_punctuation_pause_multiplier() {
    let normal = Token::new("speed", false);
    let comma = Token::new("speed,", false);
    let period = Token::new("speed.", false);

    assert_eq!(normal.pause_multiplier, 1.0);
    assert!(comma.pause_multiplier > 1.3);
    assert!(period.pause_multiplier > 2.0);
}

#[test]
fn test_document_line_chunking() {
    let text = "First sentence with multiple words. Second sentence begins here and goes on.";
    let doc = Document::from_text("Test Doc".to_string(), None, text, 5);

    assert!(doc.total_words > 10);
    assert!(doc.total_lines >= 2);

    // Verify word and line bi-directional indexing
    let line0 = doc.line_for_word(0);
    assert_eq!(line0, 0);

    let start_word_of_line0 = doc.word_for_line(0);
    assert_eq!(start_word_of_line0, 0);
}

#[test]
fn test_lopdf_methods() {
    let doc = lopdf::Document::new();
    let _ = doc.get_pages();
    let _ = doc.extract_text(&[1]);
}

#[test]
fn test_text_cleaner_hyphen_unwrap() {
    use speed_reader::text_cleaner::clean_text_for_reading;

    let input = "reengineering was a manage-\nment fad of the early 1990s.";
    let output = clean_text_for_reading(input);
    assert!(output.contains("management"), "Expected 'management', got: {}", output);
    assert!(!output.contains("manage-"), "Should not contain hyphen: {}", output);
}

#[test]
fn test_text_cleaner_soft_hyphen() {
    use speed_reader::text_cleaner::clean_text_for_reading;

    let input = "reengi\u{00AD}neering is a concept";
    let output = clean_text_for_reading(input);
    assert_eq!(output.trim(), "reengineering is a concept");
}

#[test]
fn test_text_cleaner_period_spacing() {
    use speed_reader::text_cleaner::clean_text_for_reading;

    let input = "This was a disaster.Reengineering came along.";
    let output = clean_text_for_reading(input);
    assert_eq!(output.trim(), "This was a disaster. Reengineering came along.");
}

#[test]
fn test_text_cleaner_comma_and_glued_words() {
    use speed_reader::text_cleaner::clean_text_for_reading;

    let input = "fast,the economyWithout boundaries was 1,000 strong.";
    let output = clean_text_for_reading(input);
    assert!(output.contains("fast, the"));
    assert!(output.contains("economy Without"));
    assert!(output.contains("1,000")); // Number preserved
}

#[test]
fn test_paragraph_chunking_and_indexing() {
    let multi_para_text = r#"This is paragraph number one. It contains two whole sentences.

Here is paragraph number two. It continues the discussion with further thoughts.

And finally paragraph three concludes the text."#;

    let doc = Document::from_text("Multi Para".to_string(), None, multi_para_text, 6);

    assert_eq!(doc.total_paragraphs, 3);
    assert_eq!(doc.paragraphs.len(), 3);

    // Verify each paragraph has valid non-overlapping word ranges
    assert_eq!(doc.paragraphs[0].word_indices.start, 0);
    assert_eq!(doc.paragraphs[1].word_indices.start, doc.paragraphs[0].word_indices.end);
    assert_eq!(doc.paragraphs[2].word_indices.start, doc.paragraphs[1].word_indices.end);
    assert_eq!(doc.paragraphs[2].word_indices.end, doc.total_words);

    // Verify bi-directional word <-> paragraph lookup
    let p0_word = doc.word_for_paragraph(0);
    assert_eq!(p0_word, 0);
    assert_eq!(doc.paragraph_for_word(p0_word), 0);

    let p1_word = doc.word_for_paragraph(1);
    assert!(p1_word > 0);
    assert_eq!(doc.paragraph_for_word(p1_word), 1);

    let p2_word = doc.word_for_paragraph(2);
    assert_eq!(doc.paragraph_for_word(p2_word), 2);

    // Verify line <-> paragraph lookup
    let p1_line = doc.line_for_paragraph(1);
    assert_eq!(doc.paragraph_for_line(p1_line), 1);
}

#[test]
fn test_demo_document_has_paragraphs() {
    let doc = Document::demo_document(8);
    assert!(doc.total_paragraphs >= 4);
    assert_eq!(doc.sections.len(), 4);
    for sec in &doc.sections {
        assert!(sec.paragraph_count >= 1);
    }
}

#[test]
fn test_reading_mode_serialization() {
    use speed_reader::storage::ReadingMode;

    let serialized = serde_json::to_string(&ReadingMode::Paragraph).expect("serialize");
    assert_eq!(serialized, "\"Paragraph\"");

    let deserialized: ReadingMode = serde_json::from_str(&serialized).expect("deserialize");
    assert_eq!(deserialized, ReadingMode::Paragraph);
}
