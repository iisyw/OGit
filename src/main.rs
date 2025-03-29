use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

mod git;
mod log_manager;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 提交消息
    #[arg(default_value = None)]
    commit_message: Option<String>,

    /// 是否推送到远程仓库
    #[arg(short, long)]
    push: bool,

    /// 远程仓库名称
    #[arg(short, long, default_value = "github")]
    remote: String,

    /// 是否启用CI构建
    #[arg(short, long)]
    ci: bool,

    /// 是否禁用CI构建
    #[arg(short = 'n', long = "no-ci", alias = "nc")]
    no_ci: bool,
}

fn main() -> Result<()> {
    println!("{}", "====================================".bright_green());
    println!("{}", "         项目提交与推送助手".bright_green());
    println!("{}", "====================================".bright_green());
    println!();

    // 解析命令行参数
    let mut args = Args::parse();

    // 如果命令行参数中没有提供提交消息，则提示用户输入
    let commit_message = match &args.commit_message {
        Some(msg) => msg.clone(),
        None => {
            if let Some(input) = utils::input_with_default("请输入提交标注", "Normal Update")? {
                input
            } else {
                String::from("Normal Update")
            }
        }
    };

    // 检查是否存在.github/workflows文件夹
    let has_workflows = PathBuf::from(".github/workflows").exists();
    if has_workflows {
        println!("{}", "[INFO] 检测到 CI 工作流配置".bright_blue());
    }

    // 如果未通过命令行参数指定，则交互式询问是否推送到远程仓库
    if !args.push {
        args.push = utils::confirm("是否需要推送到远程仓库?", true)?;
    }

    // 如果选择推送到远程仓库，且未通过命令行参数指定远程仓库名称，则询问远程仓库名称
    if args.push && args.remote == "github" {
        if let Some(remote_name) = utils::input_with_default("请输入远程仓库名称", "github")? {
            args.remote = remote_name;
        }
    }

    // 确定CI构建选项
    let ci_enabled = if args.no_ci {
        false
    } else if args.ci {
        true
    } else if has_workflows && args.push {
        // 如果存在workflows且需要推送，则询问是否进行CI构建
        utils::confirm("是否需要进行 CI 构建?", false)?
    } else if !has_workflows {
        // 如果不存在workflows，默认不添加[skip ci]标记
        println!("{}", "[INFO] 未检测到 CI 工作流配置，默认不添加 [skip ci] 标记".bright_blue());
        true
    } else {
        false
    };

    // 如果不需要CI构建，添加[skip ci]到提交信息
    let final_commit_message = if !ci_enabled && has_workflows {
        format!("{} [skip ci]", commit_message)
    } else {
        commit_message
    };

    // 显示操作概述
    println!();
    println!("{}", "----- 操作概述 -----".bright_yellow());
    println!("提交标注: \"{}\"", final_commit_message);
    
    if args.push {
        println!("将推送到远程仓库: {}", args.remote);
        if has_workflows {
            if ci_enabled {
                println!("CI 构建: 启用");
            } else {
                println!("CI 构建: 禁用");
            }
        } else {
            println!("CI 构建: 不适用（未检测到工作流配置）");
        }
    } else {
        println!("不推送到远程仓库");
        println!("CI 构建: 禁用");
    }

    // 确认操作
    println!();
    if !utils::confirm("确认以上设置并继续?", true)? {
        println!("操作已取消。");
        return Ok(());
    }

    // 处理日志文件
    println!();
    println!("{}", "----- 开始处理日志 -----".bright_yellow());
    log_manager::update_log_files(&final_commit_message).context("更新日志文件时出错")?;

    // 执行Git操作
    if args.push {
        println!();
        println!("{}", "----- 执行提交和推送 -----".bright_yellow());
        
        // 提交到本地仓库
        println!("{}", "[INFO] 正在添加文件到暂存区并提交到本地仓库...".bright_blue());
        git::commit(&final_commit_message).context("Git提交操作失败")?;
        println!("{}", "[SUCCESS] Git提交完成".bright_green());
        
        // 推送到远程仓库
        println!("{}", format!("[INFO] 正在推送到远程仓库 [{}]...", args.remote).bright_blue());
        git::push(&args.remote).context("推送操作失败")?;
        println!("{}", format!("[SUCCESS] 成功推送到远程仓库 [{}]", args.remote).bright_green());
    } else {
        println!("{}", "[INFO] Git操作已禁用，仅更新日志。".bright_blue());
    }

    println!();
    println!("{}", "====================================".bright_green());
    println!("{}", "            操作已完成".bright_green());
    println!("{}", "====================================".bright_green());

    Ok(())
}
