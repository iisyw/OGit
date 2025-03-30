use anyhow::{Context, Result};
use dialoguer::{Confirm, theme::ColorfulTheme};
use rustyline::DefaultEditor;
use colored::Colorize;
use std::fmt::Write as FmtWrite;

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
/// # 返回值
/// 返回格式化后的提交标注字符串
pub fn get_multiline_commit_message() -> Result<String> {
    let mut commit_content = CommitContent {
        title: String::new(),
        content_lines: Vec::new(),
    };
    
    // 获取标题
    commit_content.title = get_input("请输入提交标题: ")?;
    
    if commit_content.title.is_empty() {
        return Ok(String::from("Normal Update"));
    }
    
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