use std::path::{Path, PathBuf};
use std::time::Instant;
use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, Frame, Key, Layout, Margin, RichText,
    ScrollArea, Stroke, StrokeKind, TextEdit,
};

use crate::parser::load_document_sections;
use crate::reader::{Document, ReadingLine, Token};
use crate::storage::{AppConfig, AppTheme, ReadingMode};

pub struct SpeedReaderApp {
    pub config: AppConfig,
    pub document: Document,
    pub word_index: usize,
    pub line_index: usize,
    pub is_playing: bool,
    pub last_tick: Instant,
    pub step_duration_ms: f32,
    pub zen_mode: bool,
    pub show_settings: bool,
    pub show_chapter_modal: bool,
    pub show_library_modal: bool,
    pub chapter_search_filter: String,
    pub page_jump_input: String,
    pub modal_error: Option<String>,
    pub first_frame: bool,
    pub status_message: Option<(String, Instant)>,
}

impl SpeedReaderApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut config = AppConfig::load();

        // Start with demo document or most recent book
        let mut document = Document::demo_document(config.words_per_line);
        let mut word_idx = 0;
        let mut line_idx = 0;

        if let Some(recent) = config.recent_books.first().cloned() {
            if recent.file_path.exists() {
                if let Ok((title, sections)) = load_document_sections(&recent.file_path) {
                    document = Document::from_sections(
                        title,
                        Some(recent.file_path.clone()),
                        sections,
                        config.words_per_line,
                    );
                    word_idx = recent.word_index.min(document.total_words.saturating_sub(1));
                    line_idx = recent.line_index.min(document.total_lines.saturating_sub(1));
                    config.wpm = recent.wpm;
                    config.active_mode = recent.mode;
                }
            }
        }

        Self {
            config,
            document,
            word_index: word_idx,
            line_index: line_idx,
            is_playing: false,
            last_tick: Instant::now(),
            step_duration_ms: 250.0,
            zen_mode: false,
            show_settings: false,
            show_chapter_modal: false,
            show_library_modal: false,
            chapter_search_filter: String::new(),
            page_jump_input: String::new(),
            modal_error: None,
            first_frame: true,
            status_message: None,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn open_file(&mut self, path: PathBuf) {
        match load_document_sections(&path) {
            Ok((title, sections)) => {
                let section_count = sections.len();
                let is_pdf = sections.iter().any(|s| s.title.starts_with("Page "));
                let section_type = if is_pdf { "pages" } else { "chapters" };

                let doc = Document::from_sections(
                    title.clone(),
                    Some(path.clone()),
                    sections,
                    self.config.words_per_line,
                );
                let bookmark = self.config.get_bookmark(&path).cloned();

                self.document = doc;
                if let Some(bm) = bookmark {
                    self.word_index = bm.word_index.min(self.document.total_words.saturating_sub(1));
                    self.line_index = bm.line_index.min(self.document.total_lines.saturating_sub(1));
                    self.config.wpm = bm.wpm;
                    self.config.active_mode = bm.mode;
                    self.set_status(format!(
                        "Resumed '{}' at {:.1}% ({} {})",
                        title,
                        self.progress_pct(),
                        section_count,
                        section_type
                    ));
                } else {
                    self.word_index = 0;
                    self.line_index = 0;
                    self.set_status(format!(
                        "Loaded '{}' ({} {}, {} words)",
                        title,
                        section_count,
                        section_type,
                        self.document.total_words
                    ));
                }
                self.is_playing = false;
                self.save_current_bookmark();
            }
            Err(e) => {
                self.set_status(format!("Error loading book: {}", e));
            }
        }
    }

    pub fn save_current_bookmark(&mut self) {
        if let Some(path) = &self.document.file_path {
            self.config.update_bookmark(
                path,
                &self.document.title,
                self.word_index,
                self.line_index,
                self.document.total_words,
                self.document.total_lines,
                self.config.wpm,
                self.config.active_mode,
            );
        }
    }

    pub fn remove_book(&mut self, path: &Path) {
        self.config.remove_bookmark(path);
        self.set_status("Removed book from library");
    }

    pub fn clear_all_books(&mut self) {
        self.config.clear_bookmarks();
        self.set_status("Cleared reading library");
    }

    pub fn jump_to_section(&mut self, section_idx: usize) {
        if let Some(sec) = self.document.sections.get(section_idx) {
            self.word_index = sec.start_word_index;
            self.line_index = sec.start_line_index;
            self.is_playing = false;
            self.set_status(format!("Jumped to {}", sec.title));
            self.save_current_bookmark();
        }
    }

    pub fn jump_to_page_input(&mut self) -> Result<String, String> {
        let query = self.page_jump_input.trim();
        if query.is_empty() {
            let err = "Please enter a page or chapter number".to_string();
            self.modal_error = Some(err.clone());
            return Err(err);
        }

        // 1. Try direct 1-based page number
        if let Ok(num) = query.parse::<usize>() {
            if num >= 1 && num <= self.document.sections.len() {
                self.jump_to_section(num - 1);
                self.modal_error = None;
                return Ok(format!("Jumped to {}", self.document.sections[num - 1].title));
            }
        }

        // 2. Try matching printed book page number or title query (e.g. "215" in "Page 222 — One Company's Experience: Deere 215")
        let query_lower = query.to_lowercase();
        let mut matched_idx = None;
        for (idx, sec) in self.document.sections.iter().enumerate() {
            if sec.title.to_lowercase().contains(&query_lower) {
                matched_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = matched_idx {
            self.jump_to_section(idx);
            self.modal_error = None;
            return Ok(format!("Jumped to {}", self.document.sections[idx].title));
        }

        let err = format!(
            "Page/Chapter '{}' not found (document has 1 to {} sections)",
            query,
            self.document.sections.len()
        );
        self.modal_error = Some(err.clone());
        Err(err)
    }

    pub fn jump_to_prev_section(&mut self) {
        let cur_idx = self.document.current_section_index(self.word_index);
        if cur_idx > 0 {
            self.jump_to_section(cur_idx - 1);
        }
    }

    pub fn jump_to_next_section(&mut self) {
        let cur_idx = self.document.current_section_index(self.word_index);
        if cur_idx + 1 < self.document.sections.len() {
            self.jump_to_section(cur_idx + 1);
        }
    }

    pub fn progress_pct(&self) -> f32 {
        if self.document.total_words == 0 {
            return 0.0;
        }
        match self.config.active_mode {
            ReadingMode::Line => {
                if self.document.total_lines == 0 {
                    0.0
                } else {
                    (self.line_index as f32 / self.document.total_lines as f32) * 100.0
                }
            }
            ReadingMode::WordRsvp => {
                (self.word_index as f32 / self.document.total_words as f32) * 100.0
            }
        }
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.document.tokens.get(self.word_index)
    }

    pub fn current_line(&self) -> Option<&ReadingLine> {
        self.document.lines.get(self.line_index)
    }

    pub fn prev_line(&self) -> Option<&ReadingLine> {
        if self.line_index > 0 {
            self.document.lines.get(self.line_index - 1)
        } else {
            None
        }
    }

    pub fn next_line(&self) -> Option<&ReadingLine> {
        self.document.lines.get(self.line_index + 1)
    }

    pub fn calculate_step_duration(&self) -> f32 {
        let base_word_ms = 60_000.0 / (self.config.wpm as f32).max(50.0);
        match self.config.active_mode {
            ReadingMode::WordRsvp => {
                let multiplier = if self.config.smart_pause {
                    self.current_token().map(|t| t.pause_multiplier).unwrap_or(1.0)
                } else {
                    1.0
                };
                base_word_ms * multiplier
            }
            ReadingMode::Line => {
                if let Some(line) = self.current_line() {
                    let word_count = (line.word_indices.end - line.word_indices.start).max(1) as f32;
                    let multiplier = if self.config.smart_pause {
                        line.pause_multiplier
                    } else {
                        1.0
                    };
                    base_word_ms * word_count * multiplier
                } else {
                    base_word_ms * 6.0
                }
            }
        }
    }

    pub fn step_forward(&mut self) {
        match self.config.active_mode {
            ReadingMode::Line => {
                if self.line_index + 1 < self.document.total_lines {
                    self.line_index += 1;
                    self.word_index = self.document.word_for_line(self.line_index);
                } else {
                    self.is_playing = false;
                    self.set_status("Reached end of book");
                }
            }
            ReadingMode::WordRsvp => {
                if self.word_index + 1 < self.document.total_words {
                    self.word_index += 1;
                    self.line_index = self.document.line_for_word(self.word_index);
                } else {
                    self.is_playing = false;
                    self.set_status("Reached end of book");
                }
            }
        }
    }

    pub fn step_backward(&mut self) {
        match self.config.active_mode {
            ReadingMode::Line => {
                if self.line_index > 0 {
                    self.line_index -= 1;
                    self.word_index = self.document.word_for_line(self.line_index);
                }
            }
            ReadingMode::WordRsvp => {
                if self.word_index > 0 {
                    self.word_index -= 1;
                    self.line_index = self.document.line_for_word(self.word_index);
                }
            }
        }
    }

    pub fn jump_steps(&mut self, count: isize) {
        match self.config.active_mode {
            ReadingMode::Line => {
                let new_idx = (self.line_index as isize + count)
                    .max(0)
                    .min(self.document.total_lines.saturating_sub(1) as isize) as usize;
                self.line_index = new_idx;
                self.word_index = self.document.word_for_line(new_idx);
            }
            ReadingMode::WordRsvp => {
                let new_idx = (self.word_index as isize + count)
                    .max(0)
                    .min(self.document.total_words.saturating_sub(1) as isize) as usize;
                self.word_index = new_idx;
                self.line_index = self.document.line_for_word(new_idx);
            }
        }
    }

    pub fn jump_to_progress(&mut self, pct: f32) {
        let clamped = (pct / 100.0).clamp(0.0, 1.0);
        match self.config.active_mode {
            ReadingMode::Line => {
                let target = ((self.document.total_lines as f32) * clamped).floor() as usize;
                self.line_index = target.min(self.document.total_lines.saturating_sub(1));
                self.word_index = self.document.word_for_line(self.line_index);
            }
            ReadingMode::WordRsvp => {
                let target = ((self.document.total_words as f32) * clamped).floor() as usize;
                self.word_index = target.min(self.document.total_words.saturating_sub(1));
                self.line_index = self.document.line_for_word(self.word_index);
            }
        }
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
        if self.is_playing {
            self.last_tick = Instant::now();
            self.step_duration_ms = self.calculate_step_duration();
        } else {
            self.save_current_bookmark();
        }
    }

    pub fn rebuild_document_lines(&mut self) {
        if let Some(path) = self.document.file_path.clone() {
            if let Ok((title, sections)) = load_document_sections(&path) {
                let cur_word = self.word_index;
                self.document = Document::from_sections(
                    title,
                    Some(path),
                    sections,
                    self.config.words_per_line,
                );
                self.word_index = cur_word.min(self.document.total_words.saturating_sub(1));
                self.line_index = self.document.line_for_word(self.word_index);
            }
        } else {
            let cur_word = self.word_index;
            self.document = Document::demo_document(self.config.words_per_line);
            self.word_index = cur_word.min(self.document.total_words.saturating_sub(1));
            self.line_index = self.document.line_for_word(self.word_index);
        }
    }

    // Color theme definitions
    pub fn get_theme_colors(&self) -> ThemeColors {
        match self.config.theme {
            AppTheme::DarkSlate => ThemeColors {
                bg: Color32::from_rgb(17, 21, 28),
                surface: Color32::from_rgb(26, 32, 44),
                surface_highlight: Color32::from_rgb(38, 48, 65),
                border: Color32::from_rgb(46, 56, 74),
                text_primary: Color32::from_rgb(235, 240, 245),
                text_secondary: Color32::from_rgb(138, 150, 168),
                text_faint: Color32::from_rgb(85, 96, 114),
                accent: Color32::from_rgb(255, 110, 80), // Vibrant Coral
                accent_glow: Color32::from_rgba_unmultiplied(255, 110, 80, 40),
                progress_bg: Color32::from_rgb(30, 37, 50),
                progress_fill: Color32::from_rgb(255, 110, 80),
            },
            AppTheme::WarmSepia => ThemeColors {
                bg: Color32::from_rgb(248, 241, 227),
                surface: Color32::from_rgb(240, 230, 210),
                surface_highlight: Color32::from_rgb(232, 220, 196),
                border: Color32::from_rgb(215, 200, 175),
                text_primary: Color32::from_rgb(52, 40, 30),
                text_secondary: Color32::from_rgb(120, 105, 90),
                text_faint: Color32::from_rgb(175, 160, 145),
                accent: Color32::from_rgb(195, 60, 30), // Burnt Terracotta
                accent_glow: Color32::from_rgba_unmultiplied(195, 60, 30, 40),
                progress_bg: Color32::from_rgb(220, 208, 186),
                progress_fill: Color32::from_rgb(195, 60, 30),
            },
            AppTheme::MidnightOled => ThemeColors {
                bg: Color32::from_rgb(0, 0, 0),
                surface: Color32::from_rgb(10, 10, 10),
                surface_highlight: Color32::from_rgb(22, 22, 22),
                border: Color32::from_rgb(34, 34, 34),
                text_primary: Color32::from_rgb(255, 255, 255),
                text_secondary: Color32::from_rgb(160, 160, 160),
                text_faint: Color32::from_rgb(70, 70, 70),
                accent: Color32::from_rgb(0, 229, 255), // Electric Cyan
                accent_glow: Color32::from_rgba_unmultiplied(0, 229, 255, 45),
                progress_bg: Color32::from_rgb(20, 20, 20),
                progress_fill: Color32::from_rgb(0, 229, 255),
            },
            AppTheme::HighContrast => ThemeColors {
                bg: Color32::from_rgb(5, 5, 5),
                surface: Color32::from_rgb(15, 15, 15),
                surface_highlight: Color32::from_rgb(30, 30, 30),
                border: Color32::from_rgb(255, 220, 0),
                text_primary: Color32::from_rgb(255, 240, 50), // High-vis Yellow
                text_secondary: Color32::from_rgb(190, 180, 50),
                text_faint: Color32::from_rgb(90, 85, 30),
                accent: Color32::from_rgb(255, 50, 50), // Pure Red
                accent_glow: Color32::from_rgba_unmultiplied(255, 50, 50, 50),
                progress_bg: Color32::from_rgb(30, 30, 30),
                progress_fill: Color32::from_rgb(255, 220, 0),
            },
        }
    }
}

pub struct ThemeColors {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_highlight: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_faint: Color32,
    pub accent: Color32,
    pub accent_glow: Color32,
    pub progress_bg: Color32,
    pub progress_fill: Color32,
}

impl eframe::App for SpeedReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_frame {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            self.first_frame = false;
        }

        let colors = self.get_theme_colors();

        // Check for dropped files (drag & drop support)
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped_files {
            if let Some(path) = file.path {
                self.open_file(path);
                break;
            }
        }

        // Handle keyboard shortcuts
        let input = ctx.input(|i| {
            (
                i.key_pressed(Key::Space),
                i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::H),
                i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::L),
                i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::K),
                i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::J),
                i.key_pressed(Key::F) || i.key_pressed(Key::F11),
                i.key_pressed(Key::M),
                i.key_pressed(Key::R),
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::P),
                i.key_pressed(Key::B),
                i.key_pressed(Key::OpenBracket),
                i.key_pressed(Key::CloseBracket),
                i.modifiers.shift,
            )
        });

        let (space, left, right, up, down, key_f, key_m, key_r, esc, key_p, key_b, open_bracket, close_bracket, shift) = input;

        if space {
            self.toggle_play();
        }
        if left {
            if shift {
                self.jump_steps(-10);
            } else {
                self.step_backward();
            }
        }
        if right {
            if shift {
                self.jump_steps(10);
            } else {
                self.step_forward();
            }
        }
        if up {
            self.config.wpm = (self.config.wpm + 25).min(1200);
            self.set_status(format!("Speed: {} WPM", self.config.wpm));
            self.config.save();
        }
        if down {
            self.config.wpm = self.config.wpm.saturating_sub(25).max(60);
            self.set_status(format!("Speed: {} WPM", self.config.wpm));
            self.config.save();
        }
        if key_f {
            self.zen_mode = !self.zen_mode;
        }
        if key_m {
            self.config.active_mode = match self.config.active_mode {
                ReadingMode::Line => ReadingMode::WordRsvp,
                ReadingMode::WordRsvp => ReadingMode::Line,
            };
            self.set_status(format!(
                "Mode: {}",
                match self.config.active_mode {
                    ReadingMode::Line => "Line-by-Line",
                    ReadingMode::WordRsvp => "Word RSVP",
                }
            ));
            self.config.save();
        }
        if key_r {
            self.word_index = 0;
            self.line_index = 0;
            self.set_status("Rewound to start");
        }
        if key_p {
            self.show_chapter_modal = !self.show_chapter_modal;
        }
        if key_b {
            self.show_library_modal = !self.show_library_modal;
        }
        if open_bracket {
            self.jump_to_prev_section();
        }
        if close_bracket {
            self.jump_to_next_section();
        }
        if esc {
            if self.show_chapter_modal {
                self.show_chapter_modal = false;
            } else if self.show_library_modal {
                self.show_library_modal = false;
            } else if self.show_settings {
                self.show_settings = false;
            } else if self.zen_mode {
                self.zen_mode = false;
            } else if self.is_playing {
                self.toggle_play();
            }
        }

        // Pacing timer logic
        if self.is_playing {
            let elapsed_ms = self.last_tick.elapsed().as_secs_f32() * 1000.0;
            if elapsed_ms >= self.step_duration_ms {
                self.step_forward();
                self.last_tick = Instant::now();
                self.step_duration_ms = self.calculate_step_duration();

                // Auto-save every 30 seconds
                self.save_current_bookmark();
            }
            // Continuous repaint during playback
            ctx.request_repaint();
        }

        // Apply dark background styling to egui visuals
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(colors.text_primary);
        visuals.panel_fill = colors.bg;
        visuals.window_fill = colors.surface;
        visuals.widgets.noninteractive.bg_fill = colors.surface;
        visuals.widgets.inactive.bg_fill = colors.surface_highlight;
        visuals.widgets.hovered.bg_fill = colors.surface_highlight;
        visuals.widgets.active.bg_fill = colors.border;
        visuals.extreme_bg_color = colors.progress_bg;
        visuals.selection.bg_fill = colors.progress_fill;
        visuals.selection.stroke = Stroke::new(1.0_f32, colors.accent);
        ctx.set_visuals(visuals);

        // Top Navigation Header (Hidden in Zen Mode)
        if !self.zen_mode {
            egui::TopBottomPanel::top("top_panel")
                .frame(Frame::NONE.fill(colors.bg).inner_margin(Margin::symmetric(16, 12)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        // Logo / Title
                        ui.heading(
                            RichText::new("⚡ SPEED READER")
                                .color(colors.accent)
                                .strong()
                                .size(18.0),
                        );

                        ui.add_space(12.0);

                        // Open File Button
                        if ui.button(RichText::new("📁 Open Book").color(colors.text_primary)).clicked() {
                            if let Some(file) = rfd::FileDialog::new()
                                .add_filter("Books & Documents", &["pdf", "epub", "txt", "md"])
                                .add_filter("PDF Documents", &["pdf"])
                                .add_filter("EPUB eBooks", &["epub"])
                                .add_filter("Text Files", &["txt", "md"])
                                .pick_file()
                            {
                                self.open_file(file);
                            }
                        }

                        // Library / Saved Books Button with badge
                        let library_label = format!("📚 Library ({})", self.config.recent_books.len());
                        if ui.button(RichText::new(library_label).color(colors.text_secondary)).clicked() {
                            self.show_library_modal = !self.show_library_modal;
                        }

                        ui.add_space(8.0);

                        // Chapter / Page Selector Button
                        let cur_sec_idx = self.document.current_section_index(self.word_index);
                        let total_secs = self.document.sections.len();
                        let cur_sec_title = self.document.sections.get(cur_sec_idx)
                            .map(|s| s.title.as_str())
                            .unwrap_or("Start");

                        let section_btn_text = format!(
                            "📑 {} ({}/{})",
                            truncate_str(cur_sec_title, 24),
                            cur_sec_idx + 1,
                            total_secs
                        );

                        // Prev section button
                        if ui.button("⏮").on_hover_text("Previous Chapter / Page ([)").clicked() {
                            self.jump_to_prev_section();
                        }

                        // Clickable section badge
                        if ui.button(RichText::new(section_btn_text).color(colors.accent)).on_hover_text("Select Chapter / Page (P)").clicked() {
                            self.show_chapter_modal = !self.show_chapter_modal;
                            self.chapter_search_filter.clear();
                            self.page_jump_input = (cur_sec_idx + 1).to_string();
                        }

                        // Next section button
                        if ui.button("⏭").on_hover_text("Next Chapter / Page (])").clicked() {
                            self.jump_to_next_section();
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            // Zen mode button
                            if ui.button(RichText::new("⛶ Zen [F]").color(colors.text_secondary)).clicked() {
                                self.zen_mode = true;
                            }

                            // Settings Toggle Button
                            let settings_label = if self.show_settings { "⚙ Close" } else { "⚙ Settings" };
                            if ui.button(RichText::new(settings_label).color(colors.text_secondary)).clicked() {
                                self.show_settings = !self.show_settings;
                            }

                            // Theme Selector
                            egui::ComboBox::from_id_salt("theme_selector")
                                .selected_text(RichText::new(self.config.theme.name()).color(colors.text_secondary))
                                .show_ui(ui, |ui| {
                                    let themes = [
                                        AppTheme::DarkSlate,
                                        AppTheme::WarmSepia,
                                        AppTheme::MidnightOled,
                                        AppTheme::HighContrast,
                                    ];
                                    for t in themes {
                                        if ui.selectable_label(self.config.theme == t, t.name()).clicked() {
                                            self.config.theme = t;
                                            self.config.save();
                                        }
                                    }
                                });

                            // Mode Switcher Buttons
                            ui.horizontal(|ui| {
                                let line_active = self.config.active_mode == ReadingMode::Line;
                                let rsvp_active = self.config.active_mode == ReadingMode::WordRsvp;

                                if ui.selectable_label(line_active, "≡ Line Mode").clicked() {
                                    self.config.active_mode = ReadingMode::Line;
                                    self.config.save();
                                }
                                if ui.selectable_label(rsvp_active, "✦ Word RSVP").clicked() {
                                    self.config.active_mode = ReadingMode::WordRsvp;
                                    self.config.save();
                                }
                            });
                        });
                    });
                });
        }

        // Bottom Controls Bar (Hidden in Zen Mode)
        if !self.zen_mode {
            egui::TopBottomPanel::bottom("bottom_panel")
                .frame(Frame::NONE.fill(colors.bg).inner_margin(Margin::symmetric(24, 14)))
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        // Progress bar with scrubber slider
                        ui.horizontal(|ui| {
                            let mut pct = self.progress_pct();
                            ui.label(RichText::new(format!("{:.1}%", pct)).color(colors.text_secondary).size(12.0));

                            let slider = egui::Slider::new(&mut pct, 0.0..=100.0)
                                .show_value(false)
                                .trailing_fill(true);
                            let response = ui.add_sized([ui.available_width() - 150.0, 16.0], slider);
                            if response.changed() {
                                self.jump_to_progress(pct);
                            }

                            let cur_sec_idx = self.document.current_section_index(self.word_index);
                            let sec_type = self.document.section_type_label();
                            let stats_text = match self.config.active_mode {
                                ReadingMode::Line => format!("{}/{} lines | {} {}/{}", self.line_index + 1, self.document.total_lines, sec_type, cur_sec_idx + 1, self.document.sections.len()),
                                ReadingMode::WordRsvp => format!("{}/{} words | {} {}/{}", self.word_index + 1, self.document.total_words, sec_type, cur_sec_idx + 1, self.document.sections.len()),
                            };
                            ui.label(RichText::new(stats_text).color(colors.text_secondary).size(11.0));
                        });

                        ui.add_space(8.0);

                        // Control Buttons & WPM Adjuster
                        ui.horizontal(|ui| {
                            // Rewind 10
                            if ui.button(RichText::new("⏪ -10").size(14.0)).clicked() {
                                self.jump_steps(-10);
                            }

                            // Step back
                            if ui.button(RichText::new("◀").size(14.0)).clicked() {
                                self.step_backward();
                            }

                            // Play / Pause prominent button
                            let play_icon = if self.is_playing { "⏸ PAUSE" } else { "▶ PLAY" };
                            let play_btn = egui::Button::new(
                                RichText::new(play_icon)
                                    .color(colors.bg)
                                    .strong()
                                    .size(16.0),
                            )
                            .fill(colors.accent)
                            .corner_radius(6)
                            .min_size(vec2(120.0, 36.0));

                            if ui.add(play_btn).clicked() {
                                self.toggle_play();
                            }

                            // Step forward
                            if ui.button(RichText::new("▶").size(14.0)).clicked() {
                                self.step_forward();
                            }

                            // Forward 10
                            if ui.button(RichText::new("+10 ⏩").size(14.0)).clicked() {
                                self.jump_steps(10);
                            }

                            ui.add_space(20.0);

                            // Speed Controls
                            ui.label(RichText::new("Speed:").color(colors.text_secondary));

                            if ui.button("-25").clicked() {
                                self.config.wpm = self.config.wpm.saturating_sub(25).max(60);
                                self.config.save();
                            }

                            let mut wpm = self.config.wpm;
                            let wpm_slider = egui::Slider::new(&mut wpm, 60..=1000)
                                .suffix(" WPM")
                                .step_by(10.0);
                            if ui.add_sized([160.0, 24.0], wpm_slider).changed() {
                                self.config.wpm = wpm;
                                self.config.save();
                            }

                            if ui.button("+25").clicked() {
                                self.config.wpm = (self.config.wpm + 25).min(1200);
                                self.config.save();
                            }

                            // Speed presets
                            ui.add_space(10.0);
                            for (label, speed) in [("250", 250), ("350", 350), ("450", 450), ("600", 600)] {
                                if ui.selectable_label(self.config.wpm == speed, label).clicked() {
                                    self.config.wpm = speed;
                                    self.config.save();
                                }
                            }

                            // Status / shortcut hint at far right
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if let Some((msg, created)) = &self.status_message {
                                    if created.elapsed().as_secs() < 4 {
                                        ui.label(RichText::new(msg).color(colors.accent).size(12.0));
                                    }
                                } else {
                                    ui.label(
                                        RichText::new("Space: Play/Pause | Arrows: Nav | [ / ]: Chapters | F: Zen")
                                            .color(colors.text_faint)
                                            .size(11.0),
                                    );
                                }
                            });
                        });
                    });
                });
        }

        // Chapter / Page Selection Modal Window
        if self.show_chapter_modal {
            let mut close_modal = false;
            let sec_type = self.document.section_type_label();
            let total_sections = self.document.sections.len();
            let modal_title = format!("📑 Jump to {} (Total: {})", sec_type, total_sections);

            egui::Window::new(modal_title)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(true)
                .default_size([520.0, 500.0])
                .frame(Frame::NONE.fill(colors.surface).inner_margin(Margin::same(18)).corner_radius(12))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new(format!("Select {} to Start", sec_type)).color(colors.text_primary).size(16.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("✕ Close").clicked() {
                                close_modal = true;
                            }
                        });
                    });

                    ui.separator();
                    ui.add_space(6.0);

                    // Direct Page / Chapter Number Jump input
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Enter {} # :", sec_type)).color(colors.text_secondary));
                        let edit = TextEdit::singleline(&mut self.page_jump_input).desired_width(80.0);
                        let response = ui.add(edit);
                        let enter_pressed = response.has_focus() && ctx.input(|i| i.key_pressed(Key::Enter));
                        let go_clicked = ui.button(RichText::new("Go ➔").color(colors.accent).strong()).clicked();

                        if enter_pressed || go_clicked {
                            if self.jump_to_page_input().is_ok() {
                                close_modal = true;
                            }
                        }

                        ui.label(RichText::new(format!("(1 to {})", total_sections)).color(colors.text_faint).size(12.0));
                    });

                    // Quick Page Slider for fast scrubbing
                    if total_sections > 1 {
                        let cur_sec_idx = self.document.current_section_index(self.word_index);
                        let mut slider_val = (cur_sec_idx + 1) as u32;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Quick Slider:").color(colors.text_secondary));
                            if ui.add(egui::Slider::new(&mut slider_val, 1..=total_sections as u32).suffix(format!(" / {}", total_sections))).changed() {
                                self.jump_to_section((slider_val as usize).saturating_sub(1));
                                self.modal_error = None;
                            }
                        });
                    }

                    // Display error if jump failed
                    if let Some(err) = &self.modal_error {
                        ui.label(RichText::new(format!("⚠ {}", err)).color(Color32::from_rgb(255, 90, 90)).size(12.0));
                    }

                    ui.add_space(8.0);

                    // Search Filter
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔍 Search:").color(colors.text_secondary));
                        ui.add(
                            TextEdit::singleline(&mut self.chapter_search_filter)
                                .hint_text("Filter chapters, pages, or headings...")
                                .desired_width(ui.available_width() - 20.0),
                        );
                    });

                    ui.add_space(8.0);
                    ui.separator();

                    // Scrollable list of sections
                    let cur_sec_idx = self.document.current_section_index(self.word_index);
                    let filter = self.chapter_search_filter.to_lowercase();

                    ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                        let mut target_jump = None;
                        for (idx, sec) in self.document.sections.iter().enumerate() {
                            if !filter.is_empty() && !sec.title.to_lowercase().contains(&filter) {
                                continue;
                            }

                            let is_current = idx == cur_sec_idx;
                            let label_text = format!("{}. {}", idx + 1, sec.title);

                            ui.horizontal(|ui| {
                                let btn_text = if is_current {
                                    RichText::new(format!("● {}", label_text)).color(colors.accent).strong()
                                } else {
                                    RichText::new(label_text).color(colors.text_primary)
                                };

                                if ui.button(btn_text).clicked() {
                                    target_jump = Some(idx);
                                }

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("{} words", sec.word_count))
                                            .color(colors.text_faint)
                                            .size(11.0),
                                    );
                                });
                            });
                        }

                        if let Some(jump_idx) = target_jump {
                            self.jump_to_section(jump_idx);
                            self.modal_error = None;
                            close_modal = true;
                        }
                    });
                });

            if close_modal {
                self.show_chapter_modal = false;
                self.modal_error = None;
            }
        }

        // Library / Saved Books Manager Modal Window
        if self.show_library_modal {
            let mut close_library = false;
            let mut book_to_open = None;
            let mut book_to_remove = None;
            let mut clear_all_triggered = false;

            egui::Window::new("📚 Reading Library & History")
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(true)
                .default_size([560.0, 480.0])
                .frame(Frame::NONE.fill(colors.surface).inner_margin(Margin::same(18)).corner_radius(12))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(
                            RichText::new(format!("Recent Books ({})", self.config.recent_books.len()))
                                .color(colors.text_primary)
                                .size(16.0),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("✕ Close").clicked() {
                                close_library = true;
                            }

                            if !self.config.recent_books.is_empty() {
                                if ui.button(RichText::new("Clear All History").color(Color32::from_rgb(255, 90, 90))).clicked() {
                                    clear_all_triggered = true;
                                }
                            }
                        });
                    });

                    ui.separator();
                    ui.add_space(8.0);

                    if self.config.recent_books.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(RichText::new("No recent books in your library.").color(colors.text_secondary).size(15.0));
                            ui.label(RichText::new("Drag and drop any PDF, EPUB, or TXT file into the window to add it!").color(colors.text_faint).size(12.0));
                            ui.add_space(40.0);
                        });
                    } else {
                        ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
                            for book in &self.config.recent_books {
                                let pct = if book.total_words > 0 {
                                    (book.word_index as f32 / book.total_words as f32) * 100.0
                                } else {
                                    0.0
                                };

                                let is_active = self.document.file_path.as_ref() == Some(&book.file_path);

                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            let title_rich = if is_active {
                                                RichText::new(format!("● {}", book.title)).color(colors.accent).strong().size(14.0)
                                            } else {
                                                RichText::new(&book.title).color(colors.text_primary).strong().size(14.0)
                                            };
                                            ui.label(title_rich);

                                            let path_str = book.file_path.to_string_lossy();
                                            ui.label(RichText::new(truncate_str(&path_str, 55)).color(colors.text_faint).size(11.0));

                                            let stats = format!("{:.1}% read (word {} of {})", pct, book.word_index + 1, book.total_words);
                                            ui.label(RichText::new(stats).color(colors.text_secondary).size(11.0));
                                        });

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            // Remove book button
                                            if ui.button(RichText::new("🗑 Remove").color(Color32::from_rgb(255, 90, 90))).on_hover_text("Remove this book from library").clicked() {
                                                book_to_remove = Some(book.file_path.clone());
                                            }

                                            // Resume / Open button
                                            let open_text = if is_active { "Reading Now" } else { "▶ Open" };
                                            if ui.button(RichText::new(open_text).color(colors.accent)).clicked() {
                                                book_to_open = Some(book.file_path.clone());
                                            }
                                        });
                                    });
                                });
                                ui.add_space(4.0);
                            }
                        });
                    }
                });

            if let Some(path) = book_to_remove {
                self.remove_book(&path);
            }
            if clear_all_triggered {
                self.clear_all_books();
            }
            if let Some(path) = book_to_open {
                self.open_file(path);
                close_library = true;
            }
            if close_library {
                self.show_library_modal = false;
            }
        }

        // Settings Dialog / Side Drawer
        if self.show_settings && !self.zen_mode {
            egui::SidePanel::right("settings_panel")
                .frame(Frame::NONE.fill(colors.surface).inner_margin(Margin::same(20)))
                .min_width(270.0)
                .show(ctx, |ui| {
                    ui.heading(RichText::new("⚙ Reading Settings").color(colors.text_primary).size(18.0));
                    ui.separator();
                    ui.add_space(8.0);

                    // Font Size Slider
                    ui.label(RichText::new("Font Size").color(colors.text_secondary));
                    let mut size = self.config.font_size;
                    if ui.add(egui::Slider::new(&mut size, 22.0..=64.0).suffix(" px")).changed() {
                        self.config.font_size = size;
                        self.config.save();
                    }

                    ui.add_space(12.0);

                    // Words per line in Line Mode
                    ui.label(RichText::new("Words per Line (Line Mode)").color(colors.text_secondary));
                    let mut wpl = self.config.words_per_line;
                    if ui.add(egui::Slider::new(&mut wpl, 4..=16).suffix(" words")).changed() {
                        self.config.words_per_line = wpl;
                        self.rebuild_document_lines();
                        self.config.save();
                    }

                    ui.add_space(12.0);

                    // Smart Pause on punctuation
                    let mut sp = self.config.smart_pause;
                    if ui.checkbox(&mut sp, "Smart Punctuation Pause").changed() {
                        self.config.smart_pause = sp;
                        self.config.save();
                    }
                    ui.label(
                        RichText::new("Adds micro-pauses at commas, periods, and line endings to prevent distraction and boost comprehension.")
                            .color(colors.text_faint)
                            .size(11.0),
                    );

                    ui.add_space(12.0);

                    // Show Context in Line Mode
                    let mut sc = self.config.show_context_lines;
                    if ui.checkbox(&mut sc, "Show Context Lines (Faded)").changed() {
                        self.config.show_context_lines = sc;
                        self.config.save();
                    }
                    ui.label(
                        RichText::new("Displays previous and next lines faintly above and below to maintain reading orientation.")
                            .color(colors.text_faint)
                            .size(11.0),
                    );

                    ui.add_space(16.0);
                    ui.separator();

                    // Keyboard shortcuts reference
                    ui.add_space(8.0);
                    ui.heading(RichText::new("⌨ Shortcuts").color(colors.text_primary).size(14.0));
                    let shortcuts = [
                        ("Space", "Play / Pause"),
                        ("← / H", "Step Back"),
                        ("→ / L", "Step Forward"),
                        ("Shift + ←/→", "Jump 10 Words/Lines"),
                        ("[ / ]", "Prev / Next Chapter or Page"),
                        ("P", "Select Chapter / Page"),
                        ("B", "Manage Library & History"),
                        ("↑ / ↓", "+25 / -25 WPM Speed"),
                        ("M", "Toggle Line / RSVP Mode"),
                        ("F / F11", "Toggle Zen Focus Mode"),
                        ("R", "Rewind to Start"),
                        ("Esc", "Close Modals / Exit Zen"),
                    ];
                    for (k, desc) in shortcuts {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(k).strong().color(colors.accent).size(11.0));
                            ui.label(RichText::new(desc).color(colors.text_secondary).size(11.0));
                        });
                    }
                });
        }

        // Central Reading Viewport (The Core Experience)
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(colors.bg))
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                let center = rect.center();

                match self.config.active_mode {
                    ReadingMode::Line => {
                        self.render_line_mode(ui, rect, center, &colors);
                    }
                    ReadingMode::WordRsvp => {
                        self.render_rsvp_mode(ui, rect, center, &colors);
                    }
                }

                // If in Zen Mode, show tiny discreet exit button
                if self.zen_mode {
                    let exit_btn_rect = egui::Rect::from_min_size(rect.min + vec2(16.0, 16.0), vec2(140.0, 30.0));
                    if ui.put(exit_btn_rect, egui::Button::new(RichText::new("✕ Exit Zen [Esc]").color(colors.text_faint).size(12.0))).clicked() {
                        self.zen_mode = false;
                    }
                }
            });
    }
}

impl SpeedReaderApp {
    fn render_line_mode(&mut self, ui: &mut egui::Ui, rect: egui::Rect, center: egui::Pos2, colors: &ThemeColors) {
        let font_size = self.config.font_size;
        let line_spacing = font_size * 1.6;

        let painter = ui.painter();

        // Focused reader card frame in center
        let card_width = (rect.width() * 0.85).min(900.0);
        let card_height = (font_size * 4.2).max(180.0);
        let card_rect = egui::Rect::from_center_size(center, vec2(card_width, card_height));

        // Subtle soft glow outline
        painter.rect_stroke(card_rect.expand(2.0), CornerRadius::same(14), Stroke::new(1.0_f32, colors.accent_glow), StrokeKind::Outside);
        // Draw card background
        painter.rect_filled(card_rect, CornerRadius::same(12), colors.surface);
        painter.rect_stroke(card_rect, CornerRadius::same(12), Stroke::new(1.0_f32, colors.border), StrokeKind::Inside);

        // Draw subtle accent guide line on the left side of the active reading card
        let guide_rect = egui::Rect::from_min_max(
            card_rect.left_top() + vec2(0.0, 10.0),
            card_rect.left_bottom() + vec2(4.0, -10.0),
        );
        painter.rect_filled(guide_rect, CornerRadius::same(2), colors.accent);

        // Render previous line (Context)
        if self.config.show_context_lines {
            if let Some(prev) = self.prev_line() {
                let prev_pos = center - vec2(0.0, line_spacing);
                painter.text(
                    prev_pos,
                    Align2::CENTER_CENTER,
                    &prev.text,
                    FontId::proportional(font_size * 0.72),
                    colors.text_faint,
                );
            }
        }

        // Render CURRENT ACTIVE LINE (Prominent, High Contrast, Centered)
        if let Some(curr) = self.current_line() {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                &curr.text,
                FontId::proportional(font_size),
                colors.text_primary,
            );
        } else {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                "Ready to read. Press Space to start.",
                FontId::proportional(font_size * 0.75),
                colors.text_secondary,
            );
        }

        // Render next line (Context)
        if self.config.show_context_lines {
            if let Some(next) = self.next_line() {
                let next_pos = center + vec2(0.0, line_spacing);
                painter.text(
                    next_pos,
                    Align2::CENTER_CENTER,
                    &next.text,
                    FontId::proportional(font_size * 0.72),
                    colors.text_faint,
                );
            }
        }
    }

    fn render_rsvp_mode(&mut self, ui: &mut egui::Ui, rect: egui::Rect, center: egui::Pos2, colors: &ThemeColors) {
        let font_size = self.config.font_size * 1.35; // Larger typography for RSVP
        let painter = ui.painter();

        // RSVP Display Box
        let box_width = (rect.width() * 0.8).min(750.0);
        let box_height = (font_size * 3.0).max(200.0);
        let box_rect = egui::Rect::from_center_size(center, vec2(box_width, box_height));

        // Subtle soft glow outline
        painter.rect_stroke(box_rect.expand(2.0), CornerRadius::same(14), Stroke::new(1.0_f32, colors.accent_glow), StrokeKind::Outside);
        // Background box
        painter.rect_filled(box_rect, CornerRadius::same(12), colors.surface);
        painter.rect_stroke(box_rect, CornerRadius::same(12), Stroke::new(1.0_f32, colors.border), StrokeKind::Inside);

        // Spritz-style Optimal Recognition Point (ORP) Anchor Guides (Top and Bottom Ticks)
        let anchor_x = center.x;
        let tick_length = 14.0;
        let tick_y_top = center.y - (font_size * 0.75);
        let tick_y_bottom = center.y + (font_size * 0.75);

        // Top anchor tick
        painter.line_segment(
            [
                egui::pos2(anchor_x, tick_y_top),
                egui::pos2(anchor_x, tick_y_top + tick_length),
            ],
            Stroke::new(2.5_f32, colors.accent),
        );

        // Bottom anchor tick
        painter.line_segment(
            [
                egui::pos2(anchor_x, tick_y_bottom - tick_length),
                egui::pos2(anchor_x, tick_y_bottom),
            ],
            Stroke::new(2.5_f32, colors.accent),
        );

        if let Some(token) = self.current_token() {
            let (prefix, orp_char, suffix) = token.split_orp();
            let font = FontId::proportional(font_size);

            // Measure widths to align the ORP character directly under anchor_x
            let prefix_galley = painter.layout_no_wrap(prefix.to_string(), font.clone(), colors.text_primary);
            let orp_galley = painter.layout_no_wrap(orp_char.to_string(), font.clone(), colors.accent);
            let suffix_galley = painter.layout_no_wrap(suffix.to_string(), font.clone(), colors.text_primary);

            let prefix_w = prefix_galley.size().x;
            let orp_w = orp_galley.size().x;

            // Positioning so orp_char is centered horizontally at anchor_x
            let orp_x = anchor_x - (orp_w / 2.0);
            let prefix_x = orp_x - prefix_w;
            let suffix_x = orp_x + orp_w;
            let y_baseline = center.y - (prefix_galley.size().y / 2.0);

            // Draw Prefix
            painter.galley(egui::pos2(prefix_x, y_baseline), prefix_galley, colors.text_primary);
            // Draw Highlighted ORP character
            painter.galley(egui::pos2(orp_x, y_baseline), orp_galley, colors.accent);
            // Draw Suffix
            painter.galley(egui::pos2(suffix_x, y_baseline), suffix_galley, colors.text_primary);

            // Subtle upcoming preview line below the box
            let upcoming_text: String = self.document.tokens
                .iter()
                .skip(self.word_index + 1)
                .take(6)
                .map(|t| t.raw.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            if !upcoming_text.is_empty() {
                painter.text(
                    box_rect.center_bottom() + vec2(0.0, 24.0),
                    Align2::CENTER_TOP,
                    format!("upcoming: {}", upcoming_text),
                    FontId::proportional(13.0),
                    colors.text_faint,
                );
            }
        } else {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                "Ready to read. Press Space to start.",
                FontId::proportional(font_size * 0.6),
                colors.text_secondary,
            );
        }
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}
