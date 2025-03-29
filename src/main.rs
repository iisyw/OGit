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

/// 获取自适应全屏宽度的分隔线
fn get_full_width_separator(character: char, color_func: fn(&str) -> colored::ColoredString) -> String {
    // 获取终端宽度，如果无法获取则默认为80
    let width = termsize::get().map_or(80, |size| size.cols as usize);
    
    // 创建分隔线
    let separator = character.to_string().repeat(width);
    
    // 返回带颜色的分隔线
    color_func(&separator).to_string()
}

/// 居中显示标题，使用全屏宽度
fn print_centered_title(title: &str, color_func: fn(&str) -> colored::ColoredString) {
    let width = termsize::get().map_or(80, |size| size.cols as usize);
    
    // 计算左侧填充以居中标题
    let padding = (width.saturating_sub(title.len())) / 2;
    let left_padding = " ".repeat(padding);
    
    println!("{}{}", left_padding, color_func(title));
}

/// 格式化打印提交标注
fn print_formatted_commit_message(message: &str) {
    if message.contains('\n') {
        // 多行消息，分为标题和正文
        let lines: Vec<&str> = message.split('\n').collect();
        
        // 打印标题
        println!("{} {}", "标题:".bright_cyan(), lines[0]);
        
        // 打印正文 (如果有)
        let mut has_content = false;
        
        // 遍历除标题外的所有行
        for line in lines.iter().skip(1) {
            if !line.trim().is_empty() {
                if !has_content {
                    println!("{}", "正文:".bright_cyan());
                    has_content = true;
                }
                println!("  {}", line);
            }
        }
        
        // 如果没有内容，也显示"正文："但是是空的
        if !has_content && message.contains("[skip ci]") {
            println!("{}", "正文:".bright_cyan());
            println!("  • (无额外内容)");
        }
    } else {
        // 单行消息，只有标题
        println!("{} {}", "标题:".bright_cyan(), message);
    }
}

fn main() -> Result<()> {
    // 创建自适应全屏分割线
    let separator = get_full_width_separator('=', |s| s.bright_green());
    let section_separator = get_full_width_separator('-', |s| s.bright_yellow());
    
    println!("{}", separator);
    print_centered_title("项目提交与推送助手", |s| s.bright_green());
    println!("{}", separator);
    println!();

    // 解析命令行参数
    let mut args = Args::parse();

    // 如果命令行参数中没有提供提交消息，则使用多行输入方式获取
    let commit_message = match &args.commit_message {
        Some(msg) => msg.clone(),
        None => utils::get_multiline_commit_message()?
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
    println!("{}", section_separator);
    print_centered_title("操作概述", |s| s.bright_yellow());
    println!("{}", section_separator);
    println!("{}", "提交标注:".bright_yellow());
    print_formatted_commit_message(&final_commit_message);
    println!();
    
    if args.push {
        println!("{} {}", "将推送到远程仓库:".bright_yellow(), args.remote);
        if has_workflows {
            if ci_enabled {
                println!("{} {}", "CI 构建:".bright_yellow(), "启用".bright_green());
            } else {
                println!("{} {}", "CI 构建:".bright_yellow(), "禁用".bright_red());
            }
        } else {
            println!("{} {}", "CI 构建:".bright_yellow(), "不适用（未检测到工作流配置）".bright_blue());
        }
    } else {
        println!("{} {}", "推送状态:".bright_yellow(), "不推送到远程仓库".bright_red());
        println!("{} {}", "CI 构建:".bright_yellow(), "禁用".bright_red());
    }

    // 确认操作
    println!();
    if !utils::confirm("确认以上设置并继续?", true)? {
        println!("操作已取消。");
        return Ok(());
    }

    // 处理日志文件
    println!();
    println!("{}", section_separator);
    print_centered_title("开始处理日志", |s| s.bright_yellow());
    println!("{}", section_separator);
    log_manager::update_log_files(&final_commit_message).context("更新日志文件时出错")?;

    // 执行Git操作
    if args.push {
        println!();
        println!("{}", section_separator);
        print_centered_title("执行提交和推送", |s| s.bright_yellow());
        println!("{}", section_separator);
        
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
    println!("{}", separator);
    print_centered_title("操作已完成", |s| s.bright_green());
    println!("{}", separator);

    Ok(())
}
