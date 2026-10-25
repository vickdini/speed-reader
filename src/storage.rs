use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadingMode {
    Line,
    WordRsvp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    DarkSlate,
    WarmSepia,
    MidnightOled,
    HighContrast,
}

impl AppTheme {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DarkSlate => "Dark Slate",
            Self::WarmSepia => "Warm Sepia",
            Self::MidnightOled => "Midnight OLED",
            Self::HighContrast => "High Contrast",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookBookmark {
    pub file_path: PathBuf,
    pub title: String,
    pub word_index: usize,
    pub line_index: usize,
    pub total_words: usize,
    pub total_lines: usize,
    pub wpm: u32,
    pub mode: ReadingMode,
    pub last_read: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: AppTheme,
    pub font_size: f32,
    pub wpm: u32,
    pub words_per_line: usize,
    pub smart_pause: bool,
    pub show_context_lines: bool,
    pub active_mode: ReadingMode,
    pub recent_books: Vec<BookBookmark>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: AppTheme::DarkSlate,
            font_size: 38.0,
            wpm: 280,
            words_per_line: 8,
            smart_pause: true,
            show_context_lines: true,
            active_mode: ReadingMode::Line,
            recent_books: Vec::new(),
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        if let Ok(mut dir) = std::env::current_exe() {
            dir.pop();
            dir.push("speed_reader_config.json");
            return dir;
        }
        PathBuf::from("speed_reader_config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&data) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, data);
        }
    }

    pub fn update_bookmark(&mut self, path: &Path, title: &str, word_idx: usize, line_idx: usize, total_words: usize, total_lines: usize, wpm: u32, mode: ReadingMode) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Remove existing entry for the same path
        self.recent_books.retain(|b| b.file_path != path);

        // Insert at the front
        self.recent_books.insert(0, BookBookmark {
            file_path: path.to_path_buf(),
            title: title.to_string(),
            word_index: word_idx,
            line_index: line_idx,
            total_words,
            total_lines,
            wpm,
            mode,
            last_read: now,
        });

        // Limit to 15 recent books
        if self.recent_books.len() > 15 {
            self.recent_books.truncate(15);
        }

        self.save();
    }

    pub fn get_bookmark(&self, path: &Path) -> Option<&BookBookmark> {
        self.recent_books.iter().find(|b| b.file_path == path)
    }
}
