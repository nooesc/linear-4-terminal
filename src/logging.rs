use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref LOG_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);
}

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("linear-cli")
        .join("logs");
    
    create_dir_all(&log_dir)?;
    
    let log_file = log_dir.join(format!("linear-{}.log", Local::now().format("%Y%m%d-%H%M%S")));
    
    *LOG_FILE.lock().unwrap() = Some(log_file.clone());
    
    log_info(&format!("Logging initialized to: {}", log_file.display()));
    
    Ok(())
}

pub fn log_error(message: &str) {
    log_with_level("ERROR", message);
}

pub fn log_info(message: &str) {
    log_with_level("INFO", message);
}

pub fn log_debug(message: &str) {
    log_with_level("DEBUG", message);
}

pub fn log_panic_info(info: &std::panic::PanicInfo) {
    let mut message = String::from("PANIC: ");
    
    if let Some(location) = info.location() {
        message.push_str(&format!("at {}:{}:{} - ", 
            location.file(), 
            location.line(), 
            location.column()
        ));
    }
    
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        message.push_str(s);
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        message.push_str(s);
    } else {
        message.push_str("Unknown panic payload");
    }
    
    log_error(&message);
    
    // Also log the backtrace
    let backtrace = std::backtrace::Backtrace::capture();
    log_debug(&format!("Backtrace:\n{}", backtrace));
}

fn log_with_level(level: &str, message: &str) {
    if let Some(log_file) = LOG_FILE.lock().unwrap().as_ref() {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
        {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let _ = writeln!(file, "[{}] {} - {}", timestamp, level, message);
        }
    }
    
    // Don't print to stderr as it interferes with the TUI
    // eprintln!("[{}] {}", level, message);
}

pub fn get_log_file_path() -> Option<PathBuf> {
    LOG_FILE.lock().unwrap().clone()
}