use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Summary of one directory scan.
pub struct ScanResult {
    pub files_scanned: usize,
    pub empty_skipped: usize,
    /// Grouped by file size; only same-size files need hashing.
    pub by_size: HashMap<u64, Vec<PathBuf>>,
}

/// Recursively scan a directory: skip symlinks and empty files, group by size.
pub fn scan_directory(root: &Path) -> Result<ScanResult> {
    if !root.exists() {
        bail!("路径不存在: {}", root.display());
    }
    if !root.is_dir() {
        bail!("路径不是目录: {}", root.display());
    }

    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut files_scanned = 0usize;
    let mut empty_skipped = 0usize;

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !entry.file_type().is_file() {
            continue;
        }

        let meta = fs::metadata(path)
            .with_context(|| format!("读取元数据失败: {}", path.display()))?;
        let len = meta.len();

        if len == 0 {
            empty_skipped += 1;
            continue;
        }

        files_scanned += 1;
        by_size.entry(len).or_default().push(path.to_path_buf());
    }

    Ok(ScanResult {
        files_scanned,
        empty_skipped,
        by_size,
    })
}
