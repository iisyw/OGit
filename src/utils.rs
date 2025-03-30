use anyhow::{Context, Result};
use dialoguer::{Confirm, theme::ColorfulTheme};
use rustyline::DefaultEditor;
use colored::Colorize;

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

/// 获取多行输入作为提交标注
///
/// # 返回值
/// 返回格式化后的提交标注字符串
pub fn get_multiline_commit_message() -> Result<String> {
    // 获取标题
    let title = get_input("请输入提交标题: ")?;
    
    if title.is_empty() {
        return Ok(String::from("Normal Update"));
    }
    
    println!("{}", "请输入提交正文内容（每行一条，直接回车结束）".bright_yellow());
    
    let mut content_lines = Vec::new();
    let mut line_index = 1;
    
    loop {
        let prompt = format!("正文第{}行: ", line_index);
        let line = get_input(&prompt)?;
        
        if line.is_empty() {
            break;
        }
        
        content_lines.push(format!("- {}", line));
        line_index += 1;
    }
    
    let result = if content_lines.is_empty() {
        title
    } else {
        format!("{}\n{}", title, content_lines.join("\n"))
    };
    
    Ok(result)
}

/// 获取当前日期，格式为YYYY/MM/DD
pub fn get_today() -> String {
    let now = chrono::Local::now();
    now.format("%Y/%m/%d").to_string()
}