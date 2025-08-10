use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

/// 提交变更到Git仓库
///
/// # 参数
/// * `commit_message` - 提交消息
///
/// # 返回值
/// 成功返回Ok，失败返回Err
pub fn commit(commit_message: &str) -> Result<()> {
    // 检查是否有变更需要提交
    if !has_changes()? {
        println!("没有变更需要提交");
        return Ok(());
    }
    
    // 由于git2库对于一些git操作实现复杂，这里使用命令行git以便简化代码
    println!("{} {}", ">".bright_cyan(), "git add .".bright_yellow());
    let status = Command::new("git")
        .args(["add", "."])
        .status()
        .context("执行'git add .'失败")?;
    
    if !status.success() {
        anyhow::bail!("'git add .'命令执行失败");
    }
    
    println!("{} {}", ">".bright_cyan(), format!("git commit -m \"{}\"", commit_message).bright_yellow());
    let status = Command::new("git")
        .args(["commit", "-m", commit_message])
        .status()
        .context("执行'git commit'失败")?;
    
    if !status.success() {
        anyhow::bail!("'git commit'命令执行失败");
    }
    
    Ok(())
}

/// 推送到远程仓库
///
/// # 参数
/// * `remote` - 远程仓库名称
///
/// # 返回值
/// 成功返回Ok，失败返回Err
pub fn push(remote: &str) -> Result<()> {
    println!("{} {}", ">".bright_cyan(), format!("git push {}", remote).bright_yellow());
    let status = Command::new("git")
        .args(["push", remote])
        .status()
        .context("执行'git push'失败")?;
    
    if !status.success() {
        anyhow::bail!("'git push'命令执行失败");
    }
    
    Ok(())
}

/// 检查是否有修改需要提交
///
/// # 返回值
/// 如果有修改需要提交，返回true，否则返回false
pub fn has_changes() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("执行'git status'失败")?;
    
    Ok(!output.stdout.is_empty())
} 