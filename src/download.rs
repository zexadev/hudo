use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};

/// 异步下载文件到 cache_dir，返回本地文件路径
/// 如果文件已存在则跳过下载
pub async fn download(url: &str, cache_dir: &Path, filename: &str) -> Result<PathBuf> {
    let dest = cache_dir.join(filename);

    // 缓存命中，跳过下载
    if dest.exists() {
        println!("  {} 使用缓存: {}", console::style("↓").cyan(), filename);
        return Ok(dest);
    }

    std::fs::create_dir_all(cache_dir)
        .with_context(|| format!("无法创建缓存目录: {}", cache_dir.display()))?;

    println!("  {} {}", console::style("↓").cyan(), console::style(url).dim());

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("请求失败: {}", url))?
        .error_for_status()
        .with_context(|| format!("HTTP 错误: {}", url))?;

    // 写入临时文件，下载完成后再重命名，避免中断导致损坏
    let tmp_dest = cache_dir.join(format!("{}.tmp", filename));
    let result = download_to_tmp(&tmp_dest, resp).await;

    if let Err(e) = result {
        std::fs::remove_file(&tmp_dest).ok();
        return Err(e);
    }

    // 重命名为正式文件
    std::fs::rename(&tmp_dest, &dest)
        .with_context(|| format!("重命名临时文件失败: {}", tmp_dest.display()))?;

    println!("  {} {}", console::style("✓").green(), filename);
    Ok(dest)
}

/// 下载内容到临时文件
async fn download_to_tmp(tmp_dest: &Path, resp: reqwest::Response) -> Result<()> {
    let total_size = resp.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {bar:40.cyan/blue}  {bytes}/{total_bytes}  {eta}")
            .unwrap()
            .progress_chars("━╸─"),
    );

    let mut file = std::fs::File::create(tmp_dest)
        .with_context(|| format!("无法创建临时文件: {}", tmp_dest.display()))?;

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("下载数据流错误")?;
        std::io::Write::write_all(&mut file, &chunk).context("写入文件失败")?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_and_clear();
    Ok(())
}

/// 解压 zip 文件到目标目录
#[allow(dead_code)]
pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dest_dir)
        .with_context(|| format!("无法创建解压目录: {}", dest_dir.display()))?;

    let file = std::fs::File::open(zip_path)
        .with_context(|| format!("无法打开 zip 文件: {}", zip_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("无效的 zip 文件: {}", zip_path.display()))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).context("读取 zip 条目失败")?;
        let name = entry.name().to_string();

        let out_path = dest_dir.join(&name);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let mut outfile = std::fs::File::create(&out_path)
                .with_context(|| format!("无法创建文件: {}", out_path.display()))?;
            std::io::copy(&mut entry, &mut outfile)
                .with_context(|| format!("解压文件失败: {}", name))?;
        }
    }

    Ok(())
}

/// 找到目录下唯一的子目录（用于 zip 解压后有一层顶层目录的情况）
pub fn find_single_subdir(dir: &Path) -> Option<PathBuf> {
    let entries: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    if entries.len() == 1 {
        Some(entries[0].path())
    } else {
        None
    }
}

/// 运行 exe 安装程序（如 rustup-init.exe）
pub fn run_installer(exe_path: &Path, args: &[&str]) -> Result<()> {
    let status = std::process::Command::new(exe_path)
        .args(args)
        .status()
        .with_context(|| format!("无法启动安装程序: {}", exe_path.display()))?;

    if !status.success() {
        anyhow::bail!(
            "安装程序退出码: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}
