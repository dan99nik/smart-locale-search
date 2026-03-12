pub fn detect_default_roots() -> Vec<String> {
    let mut roots = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let candidates: &[&str] = if cfg!(target_os = "macos") {
            &["Documents", "Desktop", "Downloads", "Projects", "Developer"]
        } else if cfg!(target_os = "windows") {
            &["Documents", "Desktop", "Downloads", "Projects"]
        } else {
            &["Documents", "Desktop", "Downloads", "Projects"]
        };

        for name in candidates {
            let dir = home.join(name);
            if dir.is_dir() {
                if let Some(s) = dir.to_str() {
                    roots.push(s.to_string());
                }
            }
        }

        if roots.is_empty() {
            if let Some(s) = home.to_str() {
                roots.push(s.to_string());
            }
        }
    }

    roots
}
