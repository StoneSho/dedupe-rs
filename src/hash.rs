use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 一组内容完全相同的文件：保留一份，其余为可删候选。
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub digest: String,
    pub keep: PathBuf,
    pub candidates: Vec<PathBuf>,
}

#[derive(Debug)]
struct FileInfo {
    path: PathBuf,
    modified: SystemTime,
}

/// 对「同大小且多于 1 个」的文件组计算 SHA-256，找出真正的重复组。
pub fn find_duplicate_groups(by_size: HashMap<u64, Vec<PathBuf>>) -> Result<Vec<DuplicateGroup>> {
    let mut groups = Vec::new();

    for (_size, paths) in by_size {
        if paths.len() < 2 {
            continue;
        }

        let mut by_hash: HashMap<String, Vec<FileInfo>> = HashMap::new();

        for path in paths {
            let digest = hash_file(&path)
                .with_context(|| format!("计算哈希失败: {}", path.display()))?;
            let modified = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            by_hash
                .entry(digest)
                .or_default()
                .push(FileInfo { path, modified });
        }

        for (digest, mut files) in by_hash {
            if files.len() < 2 {
                continue;
            }

            // 保留规则：路径更短优先；同长度则更早修改优先
            files.sort_by(|a, b| {
                let pa = a.path.to_string_lossy();
                let pb = b.path.to_string_lossy();
                pa.len()
                    .cmp(&pb.len())
                    .then_with(|| a.modified.cmp(&b.modified))
                    .then_with(|| pa.cmp(&pb))
            });

            let keep = files.remove(0).path;
            let candidates = files.into_iter().map(|f| f.path).collect();
            groups.push(DuplicateGroup {
                digest,
                keep,
                candidates,
            });
        }
    }

    groups.sort_by(|a, b| a.keep.cmp(&b.keep));
    Ok(groups)
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn detects_same_content_different_paths() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("nested/b.txt");
        fs::create_dir_all(b.parent().unwrap()).unwrap();

        for path in [&a, &b] {
            let mut f = fs::File::create(path).unwrap();
            f.write_all(b"hello dedupe-rs").unwrap();
        }

        // 不同内容，不应成组
        let c = dir.path().join("c.txt");
        fs::write(&c, b"different").unwrap();

        let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        for path in [a, b, c] {
            let len = fs::metadata(&path).unwrap().len();
            by_size.entry(len).or_default().push(path);
        }

        let groups = find_duplicate_groups(by_size).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].candidates.len(), 1);
    }
}
