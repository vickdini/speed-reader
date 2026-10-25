# ⚡ Speed Reader — Distraction-Free Book Reader in Rust

A high-performance, distraction-free desktop speed reader written in **Rust** designed specifically for readers who struggle with focus, wandering attention, and re-reading sentences. Supports **PDF**, **EPUB**, **TXT**, and **Markdown** documents.

![Speed Reader Demo](https://img.shields.io/badge/Rust-2024%20Edition-orange?logo=rust)
![GUI](https://img.shields.io/badge/GUI-egui%20%2F%20eframe-blue)
![Formats](https://img.shields.io/badge/Formats-PDF%20%7C%20EPUB%20%7C%20TXT%20%7C%20MD-success)

---

## 🎯 Built for Readers Who Get Easily Distracted

When you have trouble focusing, traditional reading forces your eyes to jump back and forth across wide margins (saccades). Each jump is an opportunity for your mind to wander, lose track, or get stuck re-reading the same paragraph.

Speed Reader solves this with dedicated focus features:

### 1. ≡ Line-by-Line Mode (Recommended for Focus)
* Shows **one focused thought / line at a time** in large, crisp typography.
* **Semantic chunking**: Breaks text into natural clauses (configurable from 4 to 16 words per line) so every line is a complete, digestible thought.
* **Context lines**: Shows the previous and next lines faintly above and below (with soft opacity) so you never feel disoriented or lose your place in the narrative.
* **Accent visual guide**: A vertical anchor guide grounds your vision on the active line.

### 2. ¶ Paragraph Mode (Deep Reading & Context)
* Displays a **complete, cohesive paragraph at a time** within an elegant, centered card.
* **Pacing timer & live progress bar**: Dynamically calculates the paragraph reading duration based on word count and your selected WPM, with a smooth progress bar showing when the next paragraph will appear.
* **Context previews**: Faint preview snippets of the previous and next paragraphs above and below keep you anchored in the narrative flow.
* **Header stats**: Shows active paragraph number, current chapter/page, word count, and estimated read time.

### 3. ✦ RSVP Word Mode (Spritz-Style)
* Words flash at a fixed point on screen at your chosen reading speed (60 to 1,200 WPM).
* **Optimal Recognition Point (ORP)**: Calculates the exact eye-fixation point of every word and highlights it in a vivid accent color (coral / amber / cyan) between top and bottom alignment ticks. Your eyes never have to move!
* Upcoming preview line below keeps you contextually aware.

### 4. 📑 PDF Page & EPUB Chapter Selector
* **Start at any Page or Chapter**: Click the section badge in the header or press <kbd>P</kbd> to open the navigator.
* **Direct Page Jump**: Enter any page number (e.g., jump straight to Page 24 of 80).
* **Search Filter**: Search chapter titles with real-time filtering.
* **Quick Steppers**: Jump to previous / next chapter with <kbd>[</kbd> and <kbd>]</kbd> or the header buttons.

### 5. 📚 Reading Library & File Removal
* **Recent Books Manager**: Click **📚 Library** or press <kbd>B</kbd> to view your reading shelf.
* **Remove Files**: Remove books you are no longer interested in with the red **🗑 Remove** button.
* **Clear History**: Clear old books at any time with one click.
* **Resume Progress**: Shows your exact percentage, current word/paragraph, and last-read timestamp.

### 6. 🧠 Smart Punctuation Pausing
* Naturally pauses slightly longer on periods (`.`, `!`, `?`) and paragraph ends (2.2x – 2.4x duration) so your brain has time to absorb sentence conclusions.
* Micro-pauses on commas, semicolons, and dashes (`1.5x duration`).
* Eliminates the "rushed feeling" and dramatically improves retention.

### 7. ⛶ Zen / Full Focus Mode
* Starts **maximized** by default to fill your screen.
* Press <kbd>F</kbd> or <kbd>F11</kbd> to toggle **Zen Mode**: completely hides all buttons, sliders, progress bars, and header chrome.
* Leaves only you and the active text in pure, distraction-free isolation.

---

## 📚 Supported Formats

| Format | Extension | Capabilities |
| :--- | :--- | :--- |
| **PDF** | `.pdf` | Extracts clean text by page, allowing you to select and jump to any page number. |
| **EPUB** | `.epub` | Parses eBooks chapter-by-chapter, extracts headings, strips HTML tags, and decodes entities. |
| **Text & Markdown** | `.txt`, `.md`, `.text` | Automatically detects markdown headings (`#`) or comfortable semantic sections. |
| **Built-in Demo** | *Built-in* | Interactive onboarding guide loaded on first launch so you can start reading immediately! |

> **Drag & Drop**: Simply drag and drop any `.pdf`, `.epub`, or `.txt` file anywhere into the window to load it immediately!

---

## ⌨ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| <kbd>Space</kbd> | **Play / Pause** reading |
| <kbd>←</kbd> / <kbd>H</kbd> | Step back 1 item (line, paragraph, or word) |
| <kbd>→</kbd> / <kbd>L</kbd> | Step forward 1 item (line, paragraph, or word) |
| <kbd>Shift</kbd> + <kbd>←</kbd> / <kbd>→</kbd> | Jump 10 lines / paragraphs / words backward / forward |
| <kbd>[</kbd> / <kbd>]</kbd> | Jump to **Previous / Next Chapter or Page** |
| <kbd>P</kbd> | Open **Chapter / Page Selector** (Direct page jump + list) |
| <kbd>B</kbd> | Open **Library Manager** (Resume or **Remove Books**) |
| <kbd>↑</kbd> / <kbd>K</kbd> | Increase speed (+25 WPM) |
| <kbd>↓</kbd> / <kbd>J</kbd> | Decrease speed (-25 WPM) |
| <kbd>M</kbd> | Cycle through **Line Mode**, **Paragraph Mode**, and **Word RSVP Mode** |
| <kbd>F</kbd> / <kbd>F11</kbd> | Toggle **Zen Focus Mode** (hides all UI chrome) |
| <kbd>R</kbd> | Rewind to beginning of book |
| <kbd>Esc</kbd> | Close modals / Exit Zen Mode / Pause |

---

## 🎨 Themes & Customization

Select your preferred reading ambiance from the top-right dropdown or customize it in settings:
* **Dark Slate**: Rich obsidian blue-gray background (`#11151C`) with soft white typography and coral accents.
* **Warm Sepia**: Gentle parchment cream (`#F8F1E3`) with espresso text and burnt terracotta accents (ideal for daytime reading).
* **Midnight OLED**: Pure black (`#000000`) for zero-light environments and battery saving, with electric cyan accents.
* **High Contrast**: Vivid amber-yellow text on deep black for maximum readability and visual accessibility.

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
