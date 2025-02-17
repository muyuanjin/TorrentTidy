use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

// 全局变量存储日志文件路径
static LOG_FILE_PATH: OnceLock<Mutex<Option<String>>> = OnceLock::new();

// 设置日志文件路径
pub fn set_log_file(path: String) {
    LOG_FILE_PATH.get_or_init(|| Mutex::new(Some(path)));
}

pub trait LogUnwrap<T> {
    /// 解包该值，如果失败，则记录错误消息。
    fn log_unwrap(self,msg: &str) -> T;
}
impl<T, E: std::fmt::Debug> LogUnwrap<T> for Result<T, E> {
    fn log_unwrap(self,msg: &str) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                crate::log!("{}: {:?}", msg, err);
                panic!("{}: {:?}", msg, err);
            }
        }
    }
}
impl<T> LogUnwrap<T> for Option<T> {
    fn log_unwrap(self,msg: &str) -> T {
        match self {
            Some(val) => val,
            None => {
                crate::log!("{}", msg);
                panic!("{}", msg);
            }
        }
    }
}


// 定义 log 宏
#[macro_export]
macro_rules! log {
    ($msg:expr) => {
        $crate::logger::log_message($msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::logger::log_message(&format!($fmt, $($arg)*));
    };
}

// 实际的日志记录函数
pub fn log_message(message: &str) {
    // 打印到控制台
    println!("{}", message);

    // 尝试获取日志文件路径
    if let Some(log_file_mutex) = LOG_FILE_PATH.get() {
        let log_file = log_file_mutex.lock().unwrap();
        if let Some(ref path) = *log_file {
            // 打开文件并追加日志
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(path)
                .expect("Failed to open log file");

            writeln!(file, "{}", message).expect("Failed to write to log file");
        }
    }
}
