use anyhow::{Context, Result};
use git2::{Repository, StatusOptions};
use std::path::Path;
// use std::process::Command;

/// 打开当前目录的Git仓库
fn open_repo() -> Result<Repository> {
    let repo = Repository::open(".").context("无法打开Git仓库，请确保当前目录是一个Git项目")?;
    Ok(repo)
}

/// 提交变更到Git仓库
///
/// # 参数
/// * `commit_message` - 提交消息
///
/// # 返回值
/// 成功返回Ok，失败返回Err
pub fn commit(commit_message: &str) -> Result<()> {
    let repo = open_repo()?;

    // 检查是否有变更需要提交
    if !has_changes()? {
        println!("没有变更需要提交");
        return Ok(());
    }

    // 获取签名
    let signature = repo.signature().context("无法获取Git签名，请检查Git配置 (user.name, user.email)")?;

    // 获取索引（暂存区）
    let mut index = repo.index().context("无法获取Git索引")?;

    // 将所有变更添加到索引
    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)
         .context("无法将文件添加到暂存区")?;
    
    // 写入索引树
    let oid = index.write_tree().context("无法写入索引树")?;
    let tree = repo.find_tree(oid).context("找不到刚刚写入的树")?;

    // 获取父提交
    let head = repo.head().context("无法获取HEAD引用")?;
    let parent_commit = repo.find_commit(head.target().context("无法获取HEAD的OID")?)
                            .context("找不到父提交")?;

    // 创建提交
    repo.commit(
        Some("HEAD"), // 更新HEAD
        &signature,   // 作者
        &signature,   // 提交者
        commit_message,
        &tree,
        &[&parent_commit],
    ).context("创建提交失败")?;

    Ok(())
}

/// 推送到远程仓库
///
/// # 参数
/// * `remote` - 远程仓库名称
///
/// # 返回值
/// 成功返回Ok，失败返回Err
pub fn push(remote_name: &str) -> Result<()> {
    let repo = open_repo()?;
    
    // 获取远程仓库
    let mut remote = repo.find_remote(remote_name)
        .with_context(|| format!("找不到名为 '{}' 的远程仓库", remote_name))?;

    // 设置认证回调
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        let username = username_from_url.unwrap_or("git");
        
        // 1. 尝试通过 SSH Agent 获取凭证
        if let Ok(cred) = git2::Cred::ssh_key_from_agent(username) {
            return Ok(cred);
        }

        // 2. 如果 Agent 失败, 尝试从默认路径加载 SSH 密钥
        //    - ~/.ssh/id_rsa
        if let Ok(cred) = git2::Cred::ssh_key(
            username,
            None, // pubkey
            Path::new(&format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap())),
            None, // passphrase
        ) {
            return Ok(cred);
        }

        // 3. 如果两种方式都失败，返回错误
        Err(git2::Error::from_str("无法通过 SSH Agent 或默认密钥路径进行认证"))
    });

    // 设置推送选项
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    // 获取当前分支名称
    let head = repo.head().context("无法获取HEAD引用")?;
    let branch_name = head.shorthand().context("无法获取分支名称")?;
    
    // 执行推送
    remote.push(
        &[format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)],
        Some(&mut push_options),
    ).context("推送到远程仓库失败")?;

    Ok(())
}

/// 检查是否有修改需要提交
///
/// # 返回值
/// 如果有修改需要提交，返回true，否则返回false
pub fn has_changes() -> Result<bool> {
    let repo = open_repo()?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);

    let statuses = repo
        .statuses(Some(&mut opts))
        .context("无法获取仓库状态")?;

    Ok(!statuses.is_empty())
}