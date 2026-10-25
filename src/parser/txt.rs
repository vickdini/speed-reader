use std::fs;
use std::path::Path;

pub fn extract_txt_text(path: &Path) -> Result<(String, String), String> {
    let bytes = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Attempt standard UTF-8, fallback to lossy UTF-8
    let content = match String::from_utf8(bytes.clone()) {
        Ok(s) => s,
        Err(_) => String::from_utf8_lossy(&bytes).to_string(),
    };

    let title = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled Document")
        .to_string();

    if content.trim().is_empty() {
        return Err("File is empty".to_string());
    }

    Ok((title, content))
}
