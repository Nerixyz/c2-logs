use colored::Colorize;

pub fn info(text: &str) {
    print!("{}: ", "info".bold().green());
    println!("{}", text.dimmed());
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)+) => {
        $crate::logging::info(&format!($($arg)+));
    };
}
