use napi_derive::napi;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

/// Log level enum that matches your required structure
#[napi]
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
  Error,
  Warn,
  Info,
}

/// Log entry structure that will be sent to the JS side
#[napi(object)]
#[derive(Debug, Clone)]
pub struct LogEntry {
  pub message: String,
  #[napi(ts_type = " 'error' | 'warn' | 'info' ")]
  pub level: String, // "error", "warn", or "info" for JS compatibility
}

impl LogEntry {
  pub fn new(message: String, level: LogLevel) -> Self {
    let level_str = match level {
      LogLevel::Error => "error".to_string(),
      LogLevel::Warn => "warn".to_string(),
      LogLevel::Info => "info".to_string(),
    };

    Self {
      message,
      level: level_str,
    }
  }
}

/// Thread-safe global logger that collects logs from anywhere in the codebase
#[derive(Debug)]
pub struct Logger {
  logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl Logger {
  pub fn new() -> Self {
    Self {
      logs: Arc::new(Mutex::new(Vec::new())),
    }
  }

  /// Add a log entry to the collection
  pub fn log(&self, message: String, level: LogLevel) {
    let entry = LogEntry::new(message, level);
    if let Ok(mut logs) = self.logs.lock() {
      logs.push(entry);
    }
  }

  /// Get all collected logs and clear the collection
  pub fn drain_logs(&self) -> Vec<LogEntry> {
    if let Ok(mut logs) = self.logs.lock() {
      logs.drain(..).collect()
    } else {
      Vec::new()
    }
  }

  /// Get all collected logs without clearing them
  pub fn get_logs(&self) -> Vec<LogEntry> {
    if let Ok(logs) = self.logs.lock() {
      logs.clone()
    } else {
      Vec::new()
    }
  }

  /// Clear all logs
  pub fn clear_logs(&self) {
    if let Ok(mut logs) = self.logs.lock() {
      logs.clear();
    }
  }
}

impl Default for Logger {
  fn default() -> Self {
    Self::new()
  }
}

/// Global logger instance using once_cell for thread-safe lazy initialization
static GLOBAL_LOGGER: Lazy<Logger> = Lazy::new(Logger::new);

/// Get a reference to the global logger
pub fn get_logger() -> &'static Logger {
  &GLOBAL_LOGGER
}

/// Convenience function to log an error
pub fn log_error(message: impl Into<String>) {
  get_logger().log(message.into(), LogLevel::Error);
}

/// Convenience function to log a warning
pub fn log_warn(message: impl Into<String>) {
  get_logger().log(message.into(), LogLevel::Warn);
}

/// Convenience function to log info
pub fn log_info(message: impl Into<String>) {
  get_logger().log(message.into(), LogLevel::Info);
}

/// Convenience macros for easy logging throughout the codebase
#[macro_export]
macro_rules! log_error {
  ($($arg:tt)*) => {
    $crate::logger::log_error(format!($($arg)*))
  };
}

#[macro_export]
macro_rules! log_warn {
  ($($arg:tt)*) => {
    $crate::logger::log_warn(format!($($arg)*))
  };
}

#[macro_export]
macro_rules! log_info {
  ($($arg:tt)*) => {
    $crate::logger::log_info(format!($($arg)*))
  };
}
