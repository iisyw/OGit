use anyhow::{Context, Result};
use dialoguer::{Confirm, theme::ColorfulTheme};
use rustyline::DefaultEditor;
use colored::Colorize;
use std::fmt::Write as FmtWrite;

/// Conventional Commits 类型定义
const COMMIT_TYPES: &[(&str, &str)] = &[
    ("feat", "新功能 (A new feature)"),
    ("fix", "Bug修复 (A bug fix)"),
    ("docs", "文档变更 (Documentation only changes)"),
    ("style", "代码风格 (Changes that do not affect the meaning of the code)"),
    ("refactor", "代码重构 (A code change that neither fixes a bug nor adds a feature)"),
    ("perf", "性能优化 (A code change that improves performance)"),
    ("test", "测试相关 (Adding missing tests or correcting existing tests)"),
    ("build", "构建系统或外部依赖变更 (Changes that affect the build system or external dependencies)"),
    ("ci", "CI/CD配置文件和脚本的变更 (Changes to our CI configuration files and scripts)"),
    ("chore", "其他不修改 src 或 test 文件的变更 (Other changes that don't modify src or test files)"),
    ("revert", "回退之前的提交 (Reverts a previous commit)"),
];

/// 获取用户确认
/// 
/// # 参数
/// * `message` - 要显示的提示消息
/// * `default` - 默认选项
/// 
/// # 返回值
/// 如果用户确认，返回true，否则返回false
pub fn confirm(message: &str, default: bool) -> Result<bool> {
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(default)
        .interact()?)
}

/// 使用rustyline获取单行输入
fn get_input(prompt: &str) -> Result<String> {
    let mut rl = DefaultEditor::new().context("无法初始化输入编辑器")?;
    
    // 获取输入
    let input = rl.readline(prompt)?;
    
    Ok(input.trim().to_string())
}

/// 获取带默认值的用户输入
/// 
/// # 参数
/// * `message` - 要显示的提示消息
/// * `default` - 默认值
/// 
/// # 返回值
/// 如果用户输入了值，返回该值，否则返回默认值
pub fn input_with_default(message: &str, default: &str) -> Result<Option<String>> {
    let prompt = format!("{} (默认: {}): ", message, default);
    let input = get_input(&prompt)?;
    
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

/// 提交标注内容
struct CommitContent {
    title: String,
    content_lines: Vec<String>,
}

/// 编辑提交标注内容
fn edit_commit_content(content: &mut CommitContent) -> Result<bool> {
    println!("{}", "当前提交标注内容:".bright_yellow());
    println!("{} {}", "标题:".bright_cyan(), content.title);
    
    if !content.content_lines.is_empty() {
        println!("{}", "正文:".bright_cyan());
        for line in content.content_lines.iter() {
            println!("  {}", line);
        }
    }
    
    println!();
    println!("{}", "请选择要编辑的部分:".bright_yellow());
    println!("  0. 返回不修改");
    println!("  1. 编辑标题");
    
    let max_option = content.content_lines.len() + 3;
    
    for i in 0..content.content_lines.len() {
        println!("  {}. 编辑正文第{}行", i + 2, i + 1);
    }
    
    println!("  {}. 添加新的正文行", content.content_lines.len() + 2);
    println!("  {}. 删除最后一行正文", content.content_lines.len() + 3);
    
    let choice = get_input(&format!("请输入选择 (0-{}): ", max_option))?;
    let choice = choice.parse::<usize>().unwrap_or(0);
    
    if choice == 0 {
        return Ok(false);
    } else if choice == 1 {
        // 编辑标题
        println!("{} {}", "当前标题:".bright_cyan(), content.title);
        let new_title = get_input("请输入新标题: ")?;
        if !new_title.is_empty() {
            content.title = new_title;
            println!("{}", "标题已更新".bright_green());
        }
        return Ok(true);
    } else if choice >= 2 && choice <= content.content_lines.len() + 1 {
        // 编辑现有正文行
        let line_index = choice - 2;
        let current_line = &content.content_lines[line_index];
        let line_content = current_line.trim_start_matches("- ");
        
        println!("{} {}", "当前内容:".bright_cyan(), line_content);
        let new_content = get_input("请输入新内容: ")?;
        
        if !new_content.is_empty() {
            content.content_lines[line_index] = format!("- {}", new_content);
            println!("{}", "正文已更新".bright_green());
        }
        return Ok(true);
    } else if choice == content.content_lines.len() + 2 {
        // 添加新的正文行
        let new_content = get_input("请输入新的正文行: ")?;
        if !new_content.is_empty() {
            content.content_lines.push(format!("- {}", new_content));
            println!("{}", "已添加新的正文行".bright_green());
        }
        return Ok(true);
    } else if choice == content.content_lines.len() + 3 && !content.content_lines.is_empty() {
        // 删除最后一行正文
        content.content_lines.pop();
        println!("{}", "已删除最后一行正文".bright_green());
        return Ok(true);
    }
    
    Ok(false)
}

/// 获取多行输入作为提交标注
///
/// # 参数
/// * `default_title` - 可选的默认标题
/// 
/// # 返回值
/// 返回格式化后的提交标注字符串
use dialoguer::Select;

pub fn get_multiline_commit_message(default_title: Option<String>) -> Result<String> {
    let mut commit_content = CommitContent {
        title: String::new(),
        content_lines: Vec::new(),
    };

    // 1. 选择提交类型
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择提交类型")
        .items(&COMMIT_TYPES.iter().map(|(val, desc)| format!("{:<10} - {}", val, desc)).collect::<Vec<_>>())
        .default(0)
        .interact()
        .context("无法获取用户选择")?;
    let commit_type = COMMIT_TYPES[selection].0;

    // 2. 输入简短描述 (如果命令行没有提供)
    let subject = if let Some(title) = default_title {
        title
    } else {
        let mut subj = String::new();
        while subj.is_empty() {
            subj = get_input("请输入简短描述: ")?;
            if subj.is_empty() {
                println!("{}", "简短描述不能为空，请重新输入。".bright_red());
            }
        }
        subj
    };

    // 3. 组合标题
    let title = format!("{}: {}", commit_type, subject);
    commit_content.title = title;
    
    println!("{}", "请输入提交正文内容（每行一条，直接回车结束）".bright_yellow());
    
    let mut line_index = 1;
    
    loop {
        let prompt = format!("正文第{}行: ", line_index);
        let line = get_input(&prompt)?;
        
        if line.is_empty() {
            break;
        }
        
        commit_content.content_lines.push(format!("- {}", line));
        line_index += 1;
    }
    
    // 编辑循环
    loop {
        // 显示当前内容
        let current_message = format_commit_content(&commit_content);
        
        println!();
        println!("{}", "提交标注预览:".bright_yellow());
        println!("{}", current_message);
        println!();
        
        // 确认或编辑
        let edit_option = confirm("需要编辑提交标注吗?", false)?;
        
        if edit_option {
            // 编辑内容
            let edited = edit_commit_content(&mut commit_content)?;
            
            // 如果内容已编辑，继续循环；否则退出
            if !edited {
                break;
            }
        } else {
            // 不需要编辑，退出循环
            break;
        }
    }
    
    // 格式化最终内容
    let result = format_commit_content(&commit_content);
    
    Ok(result)
}

/// 格式化提交内容为字符串
fn format_commit_content(content: &CommitContent) -> String {
    if content.content_lines.is_empty() {
        return content.title.clone();
    } else {
        let mut result = String::new();
        _ = write!(result, "{}", content.title);
        
        for line in &content.content_lines {
            _ = write!(result, "\n{}", line);
        }
        
        result
    }
}

/// 获取当前日期，格式为YYYY/MM/DD
pub fn get_today() -> String {
    let now = chrono::Local::now();
    now.format("%Y/%m/%d").to_string()
}

/// Git Reset 模式定义
const RESET_MODES: &[(&str, &str)] = &[
    ("soft", "保留工作区和暂存区的更改"),
    ("mixed", "保留工作区的更改，但重置暂存区 (默认)"),
    ("hard", "同时丢弃工作区和暂存区的更改 (危险操作)"),
];

/// 交互式选择 Git Reset 模式
///
/// # 返回值
/// 返回选择的模式字符串
pub fn select_reset_mode() -> Result<String> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择回退模式")
        .items(&RESET_MODES.iter().map(|(val, desc)| format!("{:<8} - {}", val, desc)).collect::<Vec<_>>())
        .default(1) // 默认选中 mixed
        .interact()
        .context("无法获取用户选择")?;
    
    Ok(RESET_MODES[selection].0.to_string())
}
