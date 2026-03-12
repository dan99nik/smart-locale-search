#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_file_at_line(path: String, line: i64) -> Result<(), String> {
    let editors = ["code", "cursor"];
    for editor in &editors {
        let arg = format!("{}:{}", path, line);
        if let Ok(status) = std::process::Command::new(editor)
            .args(["--goto", &arg])
            .status()
        {
            if status.success() {
                return Ok(());
            }
        }
    }
    open::that(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reveal_in_folder(path: String) -> Result<(), String> {
    let parent = std::path::Path::new(&path)
        .parent()
        .ok_or_else(|| "No parent directory".to_string())?;
    open::that(parent).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
