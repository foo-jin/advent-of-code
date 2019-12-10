#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn std::error::Error>::from(format!($($tt)*))) }
}

#[macro_export]
macro_rules! format_err {
    ($($tt:tt)*) => { Box::<std::error::Error>::from(format!($($tt)*)) }
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
