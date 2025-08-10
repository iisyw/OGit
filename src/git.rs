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
/// * `force` - 是否强制推送
///
/// # 返回值
/// 成功返回Ok，失败返回Err
pub fn push(remote: &str, force: bool) -> Result<()> {
    let mut command = Command::new("git");
    command.arg("push").arg(remote);

    if force {
        command.arg("--force-with-lease");
        println!("{} {}", ">".bright_cyan(), format!("git push {} --force-with-lease", remote).bright_yellow());
    } else {
        println!("{} {}", ">".bright_cyan(), format!("git push {}", remote).bright_yellow());
    }

    let status = command.status().context("执行'git push'失败")?;

    if !status.success() {
        anyhow::bail!("'git push'命令执行失败");
    }

    Ok(())
}

/// 回退到指定的提交
///
/// # 参数
/// * `mode` - 回退模式 (soft, mixed, hard)
/// * `target` - 回退目标
///
/// # 返回值
/// 成功返回Ok，失败返回Err
pub fn reset(mode: &str, target: &str) -> Result<()> {
    println!("{} {}", ">".bright_cyan(), format!("git reset --{} {}", mode, target).bright_yellow());
    let status = Command::new("git")
        .args(["reset", &format!("--{}", mode), target])
        .status()
        .context("执行'git reset'失败")?;

    if !status.success() {
        anyhow::bail!("'git reset'命令执行失败");
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

/// 检查本地分支是否与远程分支存在分歧
///
/// # 参数
/// * `remote` - 远程仓库名称
///
/// # 返回值
/// 如果存在分歧，返回true，否则返回false
pub fn is_diverged(remote: &str) -> Result<bool> {
    // 获取当前分支名称
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("获取当前分支名称失败")?;
    if !branch_output.status.success() {
        anyhow::bail!("无法获取当前分支名称");
    }
    let branch_name = String::from_utf8(branch_output.stdout)?.trim().to_string();

    // 更新远程分支信息
    Command::new("git")
        .args(["remote", "update", remote])
        .output()
        .context("更新远程分支信息失败")?;

    // 获取本地HEAD
    let local_head_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .context("获取本地HEAD失败")?;
    let local_head = String::from_utf8(local_head_output.stdout)?.trim().to_string();

    // 获取远程分支的HEAD
    let remote_head_output = Command::new("git")
        .args(["rev-parse", &format!("{}/{}", remote, branch_name)])
        .output()
        .context("获取远程分支HEAD失败")?;
    let remote_head = String::from_utf8(remote_head_output.stdout)?.trim().to_string();
    
    // 如果本地和远程的HEAD相同，则没有分歧
    if local_head == remote_head {
        return Ok(false);
    }

    // 检查本地commit是否是远程commit的祖先
    let merge_base_output = Command::new("git")
        .args(["merge-base", "HEAD", &format!("{}/{}", remote, branch_name)])
        .output()
        .context("获取merge-base失败")?;
    let merge_base = String::from_utf8(merge_base_output.stdout)?.trim().to_string();

    // 如果merge-base既不是本地HEAD也不是远程HEAD，说明分支已分叉
    // 如果merge-base是本地HEAD，说明本地落后于远程
    Ok(merge_base != remote_head)
}
