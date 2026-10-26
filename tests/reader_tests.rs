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

    // Test Page mode serialization
    let page_serialized = serde_json::to_string(&ReadingMode::Page).expect("serialize page");
    assert_eq!(page_serialized, "\"Page\"");
    let page_deserialized: ReadingMode = serde_json::from_str(&page_serialized).expect("deserialize page");
    assert_eq!(page_deserialized, ReadingMode::Page);
}

#[test]
fn test_epub_text_pagination_standard() {
    use speed_reader::parser::epub::paginate_epub_text;

    // Generate ~600 words across 4 paragraphs
    let p1 = "Word ".repeat(150);
    let p2 = "Term ".repeat(150);
    let p3 = "Read ".repeat(150);
    let p4 = "Book ".repeat(150);
    let sample = format!("{}\n\n{}\n\n{}\n\n{}", p1, p2, p3, p4);

    let (sections, next_page) = paginate_epub_text("Chapter 1: Beginnings", &sample, 1);

    // Should break into multiple readable pages (each ~250-300 words)
    assert!(sections.len() >= 2, "Expected at least 2 pages, got: {}", sections.len());
    assert_eq!(next_page, sections.len() + 1);

    // Verify all titles start with "Page " so section_type_label() identifies them as pages
    for (i, sec) in sections.iter().enumerate() {
        let expected_prefix = format!("Page {} — Chapter 1: Beginnings", i + 1);
        assert_eq!(sec.title, expected_prefix);
        assert!(!sec.content.is_empty());
    }

    // Verify Document created from these sections recognizes section_type_label as "Page"
    let doc = Document::from_sections("EPUB Book".to_string(), None, sections, 8);
    assert_eq!(doc.section_type_label(), "Page");
}

#[test]
fn test_epub_text_pagination_with_explicit_pagebreaks() {
    use speed_reader::parser::epub::paginate_epub_text;

    let text_with_breaks = "First page content with enough words to read.\x0CSecond page after explicit break tag.\x0CThird page after another break.";
    let (sections, _) = paginate_epub_text("Chapter 2", text_with_breaks, 1);

    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].title, "Page 1 — Chapter 2");
    assert_eq!(sections[1].title, "Page 2 — Chapter 2");
    assert_eq!(sections[2].title, "Page 3 — Chapter 2");
}

#[test]
fn test_app_config_epub_by_page_default() {
    use speed_reader::storage::AppConfig;

    let cfg = AppConfig::default();
    assert!(cfg.epub_by_page, "epub_by_page should be true by default");
}

#[test]
fn test_innovator_solution_epub() {
    let mut doc = epub::doc::EpubDoc::new("The Innovator's Solution.epub").unwrap();
    let cover = doc.get_cover();
    assert!(cover.is_some());
    let (cover_bytes, cover_mime) = cover.unwrap();
    println!("Cover size: {}, mime: {}", cover_bytes.len(), cover_mime);
    assert!(!cover_bytes.is_empty());

    let resource_ids: Vec<(String, std::path::PathBuf, String)> = doc.resources
        .iter()
        .map(|(id, res)| (id.clone(), res.path.clone(), res.mime.clone()))
        .collect();
    for (id, path, mime) in resource_ids {
        println!("Res: {} -> {:?}, mime: {}", id, path, mime);
        let bytes = doc.get_resource(&id);
        println!("   loaded bytes: {:?}", bytes.as_ref().map(|b| b.0.len()));
    }
}

#[test]
fn test_image_texture_decode() {
    let mut doc = epub::doc::EpubDoc::new("The Innovator's Solution.epub").unwrap();
    let (cover_bytes, _) = doc.get_cover().unwrap();
    let dynamic_img = image::load_from_memory(&cover_bytes).unwrap();
    let rgba = dynamic_img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw());
    assert_eq!(color_image.size, size);
}

#[test]
fn test_innovator_solution_epub_tables_and_images() {
    use speed_reader::parser::epub::extract_epub_sections_with_mode;
    use std::path::Path;

    let path = Path::new("The Innovator's Solution.epub");
    assert!(path.exists());

    let parsed = extract_epub_sections_with_mode(path, true).expect("Failed to extract EPUB");

    // 1. Verify Cover image in images map and section
    assert!(parsed.images.contains_key("cover"), "Cover image key should exist in images map");
    assert!(parsed.images.contains_key("9781647826796.jpg"), "Cover image filename should exist in images map");

    // First section should be Cover with an Image block
    let first_sec = &parsed.sections[0];
    assert!(first_sec.title.to_lowercase().contains("cover"), "First section should be Cover, got: {}", first_sec.title);
    assert!(!first_sec.blocks.is_empty());
    let has_cover_block = first_sec.blocks.iter().any(|b| matches!(b, speed_reader::reader::SectionBlock::Image(_)));
    assert!(has_cover_block, "First section should contain an Image block");

    // 2. Verify Table 1-1 in Chapter 1
    let mut found_table_1_1 = false;
    for sec in &parsed.sections {
        for block in &sec.blocks {
            if let speed_reader::reader::SectionBlock::Table(table) = block {
                if let Some(ref cap) = table.caption {
                    if cap.contains("TABLE 1-1") {
                        found_table_1_1 = true;
                        assert!(cap.contains("Portion of selected firms"), "Caption should contain title, got: {}", cap);
                        assert_eq!(table.rows.len(), 19, "Table 1-1 should have 19 data rows");
                        assert!(
                            table.rows.iter().any(|r| r.iter().any(|c| c.contains("Dell Computer"))),
                            "Table should contain Dell Computer row"
                        );
                        assert!(
                            table.footer.as_ref().map(|f| f.contains("Deloitte Consulting")).unwrap_or(false),
                            "Table should have Deloitte Consulting footer"
                        );
                    }
                }
            }
        }
    }
    assert!(found_table_1_1, "Should find Table 1-1 in parsed sections");

    // 3. Build Document and verify paragraph structures
    let doc = Document::from_parsed_document(parsed, Some(path.to_path_buf()), 8);
    assert!(doc.total_paragraphs > 0);

    // Verify Paragraph 0 is the cover image
    let para0 = &doc.paragraphs[0];
    assert!(para0.image.is_some(), "Paragraph 0 should have image metadata");

    // Verify a paragraph contains Table 1-1
    let table_para = doc.paragraphs.iter().find(|p| {
        p.table.as_ref().map(|t| t.caption.as_deref().unwrap_or("").contains("TABLE 1-1")).unwrap_or(false)
    });
    assert!(table_para.is_some(), "Document paragraphs should contain Table 1-1");
}

#[test]
fn test_paragraph_line_stepping() {
    use speed_reader::app::SpeedReaderApp;
    use speed_reader::storage::ReadingMode;

    let p1 = "First line of words here for reading speed. Second line of words in the paragraph.";
    let p2 = "Another paragraph with multiple lines. This is line two of the second paragraph.";
    let sample = format!("{}\n\n{}", p1, p2);

    let doc = Document::from_text("Test Stepping".to_string(), None, &sample, 6);
    assert_eq!(doc.total_paragraphs, 2);
    assert!(doc.paragraphs[0].line_indices.len() >= 2);

    let mut app = SpeedReaderApp {
        config: speed_reader::storage::AppConfig {
            active_mode: ReadingMode::Paragraph,
            ..Default::default()
        },
        document: doc,
        word_index: 0,
        line_index: 0,
        paragraph_index: 0,
        is_playing: false,
        last_tick: std::time::Instant::now(),
        step_duration_ms: 250.0,
        zen_mode: false,
        show_settings: false,
        show_chapter_modal: false,
        show_library_modal: false,
        chapter_search_filter: String::new(),
        page_jump_input: String::new(),
        modal_error: None,
        first_frame: false,
        status_message: None,
        scroll_accumulator: 0.0,
        last_scroll_time: std::time::Instant::now(),
        image_textures: std::collections::HashMap::new(),
    };

    // Step forward moves line by line within paragraph 0
    let start_line = app.line_index;
    app.paragraph_line_step_forward();
    assert_eq!(app.paragraph_index, 0);
    assert_eq!(app.line_index, start_line + 1);

    // Step backward returns to previous line
    app.paragraph_line_step_backward();
    assert_eq!(app.paragraph_index, 0);
    assert_eq!(app.line_index, start_line);

    // Stepping to the end of paragraph 0 and one more advances to paragraph 1
    let p0_end_line = app.document.paragraphs[0].line_indices.end;
    while app.line_index + 1 < p0_end_line {
        app.paragraph_line_step_forward();
    }
    assert_eq!(app.line_index, p0_end_line - 1);

    // Step forward at boundary advances to paragraph 1 first line
    app.paragraph_line_step_forward();
    assert_eq!(app.paragraph_index, 1);
    assert_eq!(app.line_index, app.document.paragraphs[1].line_indices.start);

    // Step backward at boundary returns to paragraph 0 last line
    app.paragraph_line_step_backward();
    assert_eq!(app.paragraph_index, 0);
    assert_eq!(app.line_index, p0_end_line - 1);
}


