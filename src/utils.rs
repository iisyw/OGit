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

/// 获取当前日期，格式为YYYY/MM/DD
pub fn get_today() -> String {
    let now = chrono::Local::now();
    now.format("%Y/%m/%d").to_string()
} 