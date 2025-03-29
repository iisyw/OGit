use anyhow::{Context, Result};
use colored::Colorize;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// 日志文件常量
const TODAY_LOG_FILE: &str = "TodayDevelopment.md";
const MAIN_LOG_FILE: &str = "Development.md";

/// 更新日志文件
/// 
/// # 参数
/// * `commit_message` - 提交消息
/// 
/// # 返回值
/// 成功返回Ok，失败返回Err
pub fn update_log_files(commit_message: &str) -> Result<()> {
    let today = crate::utils::get_today();
    
    // 处理主日志文件
    check_or_create_main_log_file().context("检查或创建主日志文件失败")?;
    
    // 处理今日日志文件
    if !Path::new(TODAY_LOG_FILE).exists() {
        create_today_log_file(&today, commit_message).context("创建今日日志文件失败")?;
    } else {
        update_today_log_file(&today, commit_message).context("更新今日日志文件失败")?;
    }
    
    Ok(())
}

/// 检查或创建主日志文件
fn check_or_create_main_log_file() -> Result<()> {
    if !Path::new(MAIN_LOG_FILE).exists() {
        // 创建主日志文件并添加标题
        let mut file = File::create(MAIN_LOG_FILE)?;
        writeln!(file, "# 开发日志")?;
        println!("{}", format!("[INFO] 已创建主日志文件: {}", MAIN_LOG_FILE).bright_blue());
    }
    Ok(())
}

/// 获取格式化的提交消息，以适应Markdown格式
fn format_commit_message_for_markdown(commit_message: &str) -> String {
    // 先检查提交消息是否已经是多行格式
    if commit_message.contains('\n') {
        // 多行格式，需要缩进保持Markdown的列表结构
        let lines: Vec<&str> = commit_message.split('\n').collect();
        
        // 如果只有一行，则直接返回
        if lines.len() == 1 {
            return commit_message.to_string();
        }
        
        // 处理多行格式，第一行是标题，其余是内容
        let title = lines[0];
        
        // 重新组织内容，确保正确的Markdown格式
        let mut formatted = String::new();
        formatted.push_str(title);
        
        // 添加内容，不添加空行
        for line in lines.iter().skip(1) {
            // 跳过空行
            if line.trim().is_empty() {
                continue;
            }
            
            // 添加换行符
            formatted.push('\n');
            
            // 添加内容
            formatted.push_str(line);
        }
        
        formatted
    } else {
        // 单行格式，直接返回
        commit_message.to_string()
    }
}

/// 创建今日日志文件
fn create_today_log_file(today: &str, commit_message: &str) -> Result<()> {
    let mut file = File::create(TODAY_LOG_FILE)?;
    writeln!(file, "## {}", today)?;
    writeln!(file, "")?;
    
    // 格式化提交消息并写入
    let formatted_message = format_commit_message_for_markdown(commit_message);
    writeln!(file, "1. {}", formatted_message)?;
    
    println!("{}", format!("[INFO] 已创建日志并添加到 {}", TODAY_LOG_FILE).bright_blue());
    Ok(())
}

/// 更新今日日志文件
fn update_today_log_file(today: &str, commit_message: &str) -> Result<()> {
    // 检查日期是否匹配并计算日志条目数
    let (date_match, log_count) = check_log_file_date(today)?;
    
    if date_match {
        // 日期匹配，追加新日志
        let new_log_number = log_count + 1;
        let mut file = OpenOptions::new().append(true).open(TODAY_LOG_FILE)?;
        
        // 格式化提交消息并写入
        let formatted_message = format_commit_message_for_markdown(commit_message);
        
        // 对于多行消息，我们需要确保正确缩进
        // 将格式化的消息按行分割
        let lines: Vec<&str> = formatted_message.split('\n').collect();
        
        if lines.len() == 1 {
            // 单行消息，直接添加
            writeln!(file, "{}. {}", new_log_number, formatted_message)?;
        } else {
            // 多行消息，需要缩进后续行以保持Markdown列表格式
            writeln!(file, "{}. {}", new_log_number, lines[0])?; // 写入第一行
            
            // 写入后续行，需要保持适当的缩进
            for line in lines.iter().skip(1) {
                if line.trim().is_empty() {
                    // 跳过空行
                    continue;
                } else {
                    // 非空行，保持缩进
                    writeln!(file, "   {}", line)?;
                }
            }
        }
        
        println!("{}", format!("[SUCCESS] 已更新: {}", TODAY_LOG_FILE).bright_green());
    } else {
        // 日期不匹配，将今日日志内容追加到主日志
        println!("{}", "[INFO] 检测到日期不匹配，正在合并日志...".bright_blue());
        
        // 将今日日志内容追加到主日志文件
        if Path::new(MAIN_LOG_FILE).exists() {
            let today_content = fs::read_to_string(TODAY_LOG_FILE)?;
            let mut main_file = OpenOptions::new().append(true).open(MAIN_LOG_FILE)?;
            writeln!(main_file, "")?;
            write!(main_file, "{}", today_content)?;
        } else {
            fs::copy(TODAY_LOG_FILE, MAIN_LOG_FILE)?;
        }
        
        // 创建新的今日日志
        create_today_log_file(today, commit_message)?;
        println!("{}", format!("[SUCCESS] 已创建新日志: {}", TODAY_LOG_FILE).bright_green());
    }
    
    Ok(())
}

/// 检查日志文件日期
/// 
/// # 参数
/// * `today` - 今天的日期
/// 
/// # 返回值
/// 返回一个元组，第一个元素表示日期是否匹配，第二个元素表示日志条目数
fn check_log_file_date(today: &str) -> Result<(bool, usize)> {
    let file = File::open(TODAY_LOG_FILE)?;
    let reader = BufReader::new(file);
    
    let mut date_match = false;
    let mut log_count = 0;
    
    for line in reader.lines() {
        let line = line?;
        
        // 检查第一行是否包含今天的日期
        if !date_match && line.contains(&format!("## {}", today)) {
            date_match = true;
        }
        
        // 计数日志条目
        if date_match && line.trim().starts_with(|c: char| c.is_digit(10)) {
            log_count += 1;
        }
    }
    
    Ok((date_match, log_count))
} 