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
