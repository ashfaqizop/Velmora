//! User preferences, stored as JSON in the app config folder.

use std::path::{Path, PathBuf};

use universeapi::Preferences;

pub fn path(config_dir: &Path) -> PathBuf {
    config_dir.join("preferences.json")
}

/// Saved preferences, or the engine defaults when there are none (or the file is unreadable).
pub fn load(file: &Path) -> Preferences {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(file: &Path, prefs: &Preferences) -> Result<(), String> {
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    // Write then rename so a crash never leaves a half-written file.
    let tmp = file.with_extension("json.tmp");
    std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, file).map_err(|e| e.to_string())
}
