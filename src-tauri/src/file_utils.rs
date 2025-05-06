use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileStats {
    pub size: u64,
    pub modified: u64,
    pub created: u64,
}

#[tauri::command]
pub fn get_file_stats(path: String) -> Result<FileStats, String> {
    let path = Path::new(&path);
    match fs::metadata(path) {
        Ok(metadata) => {
            let size = metadata.len();
            
            // Get the modified time as milliseconds since epoch
            let modified = metadata
                .modified()
                .map(|time| {
                    time.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64
                })
                .unwrap_or(0);
            
            // Get the creation time as milliseconds since epoch
            let created = metadata
                .created()
                .map(|time| {
                    time.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64
                })
                .unwrap_or(0);
            
            Ok(FileStats {
                size,
                modified,
                created,
            })
        },
        Err(e) => Err(format!("Failed to get file stats: {}", e)),
    }
}
