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
* **Line-by-line scrolling**: Scroll up and down line-by-line smoothly with the **mouse wheel** or the <kbd>↑</kbd> and <kbd>↓</kbd> arrow keys.

### 2. 📄 Page Mode (Book Page View with Line-by-Line Navigation)
* Displays a **complete, beautifully formatted book page** with clean typography.
* **Active line focus**: Highlights the current reading line with a vivid accent guide and soft glow while keeping surrounding page lines readable for natural context.
* **Mouse wheel & arrow scrolling**: Effortlessly scroll line-by-line up and down through the page with either the **mouse wheel** or the <kbd>↑</kbd> and <kbd>↓</kbd> arrow keys.
* **Interactive line jump**: Click directly on any line on the page to jump focus instantly.
* **Seamless page turns**: Automatically turns to the next page when scrolling past the bottom line or when playing at your selected speed.

### 3. ¶ Paragraph Mode (Deep Reading, Tables & Images)
* Displays a **complete, cohesive paragraph at a time** within an elegant, centered card.
* **Full Table Support**: Structured HTML tables (such as Table 1-1, comparison matrices, and financial tables) are preserved as cohesive, interactive units with bold accent headers, striped rows, captions, and source footers, with both horizontal and vertical scrolling. No more shredded single-word cells!
* **Embedded & Cover Images**: Displays book cover images (e.g. from EPUB cover metadata or cover pages) and in-chapter illustrations/figures with aspect ratio preservation, rounded corners, subtle borders, and captions.
* **Pacing timer & live progress bar**: Dynamically calculates reading duration based on word count and your selected WPM, with a smooth progress bar showing when the next block will appear.
* **Context previews**: Faint preview snippets of previous and next paragraphs above and below keep you anchored in the narrative flow.
* **Header stats**: Shows active paragraph or table/image badge (`📊 TABLE`, `🖼 IMAGE`), current chapter/page, word count, and estimated read time.

### 4. ✦ RSVP Word Mode (Spritz-Style)
* Words flash at a fixed point on screen at your chosen reading speed (60 to 1,200 WPM).
* **Optimal Recognition Point (ORP)**: Calculates the exact eye-fixation point of every word and highlights it in a vivid accent color (coral / amber / cyan) between top and bottom alignment ticks. Your eyes never have to move!
* Upcoming preview line below keeps you contextually aware.

### 5. 📑 EPUB & PDF Page Navigation
* **Read EPUB Documents by Page**: EPUB chapters are automatically segmented into comfortable, standard book pages (~250 words per page) with page numbering and chapter subtitles.
* **Direct Page Jump**: Jump straight to any page number (e.g. Page 24 of 80) via <kbd>P</kbd> or the header badge.
* **Search Filter**: Search chapter titles or page numbers with real-time filtering.
* **Quick Steppers**: Jump to previous / next page or chapter with <kbd>[</kbd> and <kbd>]</kbd>, <kbd>PageUp</kbd> / <kbd>PageDown</kbd>, or on-screen buttons.

### 6. 📚 Reading Library & File Removal
* **Recent Books Manager**: Click **📚 Library** or press <kbd>B</kbd> to view your reading shelf.
* **Remove Files**: Remove books you are no longer interested in with the red **🗑 Remove** button.
* **Clear History**: Clear old books at any time with one click.
* **Resume Progress**: Shows your exact percentage, current word/line/paragraph, and last-read timestamp.

### 7. 🧠 Smart Punctuation Pausing
* Naturally pauses slightly longer on periods (`.`, `!`, `?`) and paragraph ends (2.2x – 2.4x duration) so your brain has time to absorb sentence conclusions.
* Micro-pauses on commas, semicolons, and dashes (`1.5x duration`).
* Eliminates the "rushed feeling" and dramatically improves retention.

### 8. ⛶ Zen / Full Focus Mode
* Starts **maximized** by default to fill your screen.
* Press <kbd>F</kbd> or <kbd>F11</kbd> to toggle **Zen Mode**: completely hides all buttons, sliders, progress bars, and header chrome.
* Leaves only you and the active text in pure, distraction-free isolation.

---

## 📚 Supported Formats

| Format | Extension | Capabilities |
| :--- | :--- | :--- |
| **EPUB** | `.epub` | Reads by page or chapter; automatically paginates eBooks into readable pages (~250 words), extracts headings, and supports line-by-line scrolling. |
| **PDF** | `.pdf` | Extracts clean text by page, allowing you to select and jump to any page number. |
| **Text & Markdown** | `.txt`, `.md`, `.text` | Automatically detects markdown headings (`#`) or comfortable semantic sections. |
| **Built-in Demo** | *Built-in* | Interactive onboarding guide loaded on first launch so you can start reading immediately! |

> **Drag & Drop**: Simply drag and drop any `.pdf`, `.epub`, or `.txt` file anywhere into the window to load it immediately!

---

## ⌨ Keyboard & Mouse Controls

| Shortcut | Action |
| :--- | :--- |
| <kbd>Space</kbd> | **Play / Pause** reading |
| **Mouse Wheel** (Up / Down) | **Scroll line by line up and down** |
| <kbd>↑</kbd> / <kbd>↓</kbd> | **Scroll line by line up and down** |
| <kbd>←</kbd> / <kbd>→</kbd> or <kbd>H</kbd> / <kbd>L</kbd> | Step backward / forward 1 item |
| <kbd>Shift</kbd> + <kbd>←</kbd> / <kbd>→</kbd> | Jump 10 lines / paragraphs / words backward / forward |
| <kbd>PageUp</kbd> / <kbd>PageDown</kbd> | Previous / Next **Page** |
| <kbd>[</kbd> / <kbd>]</kbd> | Jump to **Previous / Next Page or Chapter** |
| <kbd>P</kbd> | Open **Page / Chapter Selector** (Direct page jump + list) |
| <kbd>B</kbd> | Open **Library Manager** (Resume or **Remove Books**) |
| <kbd>Shift</kbd> + <kbd>↑</kbd> / <kbd>↓</kbd> or <kbd>+</kbd> / <kbd>-</kbd> | Increase / Decrease speed (+/- 25 WPM) |
| <kbd>K</kbd> / <kbd>J</kbd> | Increase / Decrease speed (+/- 25 WPM) |
| <kbd>M</kbd> | Cycle through **Line**, **Page**, **Paragraph**, and **Word RSVP** modes |
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
