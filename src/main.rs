mod hash;
mod report;
mod scan;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Find duplicate files with a size-then-SHA-256 pipeline.
/// Default is dry-run; deletion requires `--delete`.
#[derive(Debug, Parser)]
#[command(name = "dedupe-rs", version, about, long_about = None)]
struct Cli {
    /// Directory to scan
    path: PathBuf,

    /// Delete candidate duplicates (default: report only)
    #[arg(long)]
    delete: bool,

    /// Skip confirmation when deleting (use with `--delete`)
    #[arg(long)]
    yes: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let scan_result = scan::scan_directory(&cli.path)?;
    println!(
        "已扫描 {} 个文件（跳过空文件 {} 个）",
        scan_result.files_scanned, scan_result.empty_skipped
    );

    let groups = hash::find_duplicate_groups(scan_result.by_size)?;

    if groups.is_empty() {
        println!("未发现内容重复的文件。");
        return Ok(());
    }

    report::print_groups(&groups);

    if cli.delete {
        report::delete_candidates(&groups, cli.yes)?;
    } else {
        println!("\n当前为 dry-run。若确认要删除候选文件，请加上 `--delete`（建议先备份）。");
    }

    Ok(())
}
