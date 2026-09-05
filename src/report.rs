use crate::hash::DuplicateGroup;
use anyhow::{bail, Context, Result};
use std::io::{self, Write};

/// 打印每组重复文件：保留文件 + 候选删除列表。
pub fn print_groups(groups: &[DuplicateGroup]) {
    println!("\n发现 {} 组重复文件：\n", groups.len());

    for (i, g) in groups.iter().enumerate() {
        println!("── 组 {} (sha256: {})", i + 1, &g.digest[..12.min(g.digest.len())]);
        println!("  保留: {}", g.keep.display());
        for c in &g.candidates {
            println!("  候选: {}", c.display());
        }
        println!();
    }
}

/// 删除各组的候选文件。无 `--yes` 时会二次确认。
pub fn delete_candidates(groups: &[DuplicateGroup], yes: bool) -> Result<()> {
    let total: usize = groups.iter().map(|g| g.candidates.len()).sum();
    if total == 0 {
        println!("没有可删除的候选文件。");
        return Ok(());
    }

    if !yes {
        print!("即将删除 {} 个候选文件。输入 yes 确认: ", total);
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if answer.trim() != "yes" {
            bail!("已取消删除。");
        }
    }

    let mut deleted = 0usize;
    for g in groups {
        for path in &g.candidates {
            std::fs::remove_file(path)
                .with_context(|| format!("删除失败: {}", path.display()))?;
            println!("已删除: {}", path.display());
            deleted += 1;
        }
    }

    println!("共删除 {} 个文件。", deleted);
    Ok(())
}
