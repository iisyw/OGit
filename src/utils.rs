use anyhow::Result;
use dialoguer::{Confirm, Input};

/// 获取用户确认
/// 
/// # 参数
/// * `message` - 要显示的提示消息
/// * `default` - 默认选项
/// 
/// # 返回值
/// 如果用户确认，返回true，否则返回false
pub fn confirm(message: &str, default: bool) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(message)
        .default(default)
        .interact()?)
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
    let input: String = Input::new()
        .with_prompt(format!("{} (默认: {})", message, default))
        .allow_empty(true)
        .interact()?;
    
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
    let title: String = Input::new()
        .with_prompt("请输入提交标题")
        .interact_text()?;
    
    if title.is_empty() {
        return Ok(String::from("Normal Update"));
    }
    
    println!("请输入提交正文内容（每行一条，直接回车结束）：");
    
    let mut content_lines = Vec::new();
    let mut line_index = 1;
    
    loop {
        let line: String = Input::new()
            .with_prompt(format!("正文第{}行", line_index))
            .allow_empty(true)
            .interact_text()?;
        
        if line.is_empty() {
            break;
        }
        
        content_lines.push(format!("- {}", line));
        line_index += 1;
    }
    
    let result = if content_lines.is_empty() {
        title
    } else {
        format!("{}\n\n{}", title, content_lines.join("\n"))
    };
    
    Ok(result)
}

/// 获取当前日期，格式为YYYY/MM/DD
pub fn get_today() -> String {
    let now = chrono::Local::now();
    now.format("%Y/%m/%d").to_string()
} 