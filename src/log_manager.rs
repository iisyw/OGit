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

/// 创建今日日志文件
fn create_today_log_file(today: &str, commit_message: &str) -> Result<()> {
    let mut file = File::create(TODAY_LOG_FILE)?;
    writeln!(file, "## {}", today)?;
    writeln!(file, "")?;
    writeln!(file, "1. {}", commit_message)?;
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
        writeln!(file, "{}. {}", new_log_number, commit_message)?;
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