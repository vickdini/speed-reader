# ⚡ Speed Reader — Distraction-Free Book Reader in Rust

A high-performance, distraction-free desktop speed reader written in **Rust** designed specifically for readers who struggle with focus, wandering attention, and re-reading sentences. Supports **PDF**, **EPUB**, **TXT**, and **Markdown** documents.

![Speed Reader Demo](https://img.shields.io/badge/Rust-2024%20Edition-orange?logo=rust)
![GUI](https://img.shields.io/badge/GUI-egui%20%2F%20eframe-blue)
![Formats](https://img.shields.io/badge/Formats-PDF%20%7C%20EPUB%20%7C%20TXT%20%7C%20MD-success)

---

## 🎯 Built for Readers Who Get Easily Distracted

When you have trouble focusing, traditional reading forces your eyes to jump back and forth across wide margins (saccades). Each jump is an opportunity for your mind to wander, lose track, or get stuck re-reading the same paragraph.

Speed Reader solves this with two dedicated focus modes:

### 1. ≡ Line-by-Line Mode (Recommended for Focus)
* Shows **one focused thought / line at a time** in large, crisp typography.
* **Semantic chunking**: Breaks text into natural clauses (configurable from 4 to 16 words per line) so every line is a complete, digestible thought.
* **Context lines**: Shows the previous and next lines faintly above and below (with soft opacity) so you never feel disoriented or lose your place in the narrative.
* **Accent visual guide**: A vertical anchor guide grounds your vision on the active line.

### 2. ✦ RSVP Word Mode (Spritz-Style)
* Words flash at a fixed point on screen at your chosen reading speed (60 to 1,200 WPM).
* **Optimal Recognition Point (ORP)**: Calculates the exact eye-fixation point of every word and highlights it in a vivid accent color (coral / amber / cyan) between top and bottom alignment ticks. Your eyes never have to move!
* Upcoming preview line below keeps you contextually aware.

### 3. 🧠 Smart Punctuation Pausing
* Naturally pauses slightly longer on periods (`.`, `!`, `?`) and paragraph ends (2.2x – 2.4x duration) so your brain has time to absorb sentence conclusions.
* Micro-pauses on commas, semicolons, and dashes (`1.5x duration`).
* Eliminates the "rushed feeling" and dramatically improves retention.

### 4. ⛶ Zen / Focus Mode
* Press `F` or `F11` to toggle **Zen Mode**: completely hides all buttons, sliders, progress bars, and header chrome.
* Leaves only you and the active text in pure, distraction-free isolation.

---

## 📚 Supported Formats

| Format | Extension | Capabilities |
| :--- | :--- | :--- |
| **PDF** | `.pdf` | Extracts clean text using `pdf-extract`, removes page artifacts, fixes hyphenated line wraps. |
| **EPUB** | `.epub` | Parses eBooks chapter-by-chapter, strips HTML/CSS tags, decodes special entities (`&mdash;`, quotes, unicode). |
| **Text & Markdown** | `.txt`, `.md`, `.text` | UTF-8 support with lossy fallback for any plain text file. |
| **Built-in Demo** | *Built-in* | Interactive onboarding guide loaded on first launch so you can start reading immediately! |

> **Drag & Drop**: Simply drag and drop any `.pdf`, `.epub`, or `.txt` file anywhere into the window to load it immediately!

---

## ⌨ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| <kbd>Space</kbd> | **Play / Pause** reading |
| <kbd>←</kbd> / <kbd>H</kbd> | Step back 1 line / word |
| <kbd>→</kbd> / <kbd>L</kbd> | Step forward 1 line / word |
| <kbd>Shift</kbd> + <kbd>←</kbd> / <kbd>→</kbd> | Jump 10 lines / words backward / forward |
| <kbd>↑</kbd> / <kbd>K</kbd> | Increase speed (+25 WPM) |
| <kbd>↓</kbd> / <kbd>J</kbd> | Decrease speed (-25 WPM) |
| <kbd>M</kbd> | Switch between **Line Mode** and **Word RSVP Mode** |
| <kbd>F</kbd> / <kbd>F11</kbd> | Toggle **Zen Focus Mode** (hides all UI chrome) |
| <kbd>R</kbd> | Rewind to beginning of book |
| <kbd>Esc</kbd> | Exit Zen Mode / Pause |

---

## 🎨 Themes & Customization

Select your preferred reading ambiance from the top-right dropdown or customize it in settings:
* **Dark Slate**: Rich obsidian blue-gray background (`#11151C`) with soft white typography and coral accents.
* **Warm Sepia**: Gentle parchment cream (`#F8F1E3`) with espresso text and burnt terracotta accents (ideal for daytime reading).
* **Midnight OLED**: Pure black (`#000000`) for zero-light environments and battery saving, with electric cyan accents.
* **High Contrast**: Vivid amber-yellow text on deep black for maximum readability and visual accessibility.

---

## 💾 Automatic Bookmarking & Progress Persistence

* Your reading position (`word_index`, `line_index`, `total_words`, `progress %`, and preferred `WPM`) is **automatically saved** per file to `speed_reader_config.json`.
* When you reopen any previously opened book, Speed Reader automatically resumes at the exact line and speed you left off.
* Use the **Recent Books** dropdown in the header to jump between books instantly.

---

## 🚀 Running the Application

### Option 1: Run the Pre-compiled Release Executable
```bash
cargo run --release
```
Or double-click:
`target\release\speed-reader.exe`

### Option 2: Development Mode
```bash
cargo run
```

### Option 3: Run Unit Tests
```bash
cargo test
```
All tests verify ORP calculations, tokenization, punctuation multipliers, and line chunking.
