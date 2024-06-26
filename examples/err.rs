use anyhow::Context;
use std::{fs, mem::size_of, num::ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("Serialize json error: {0}")]
    Serialize(#[from] serde_json::Error),

    // 第 0 个参数用 debug 格式化
    #[error("Error: {0:?}")]
    BigError(Box<BigError>),

    #[error("Custom error: {0}")]
    Custom(String),
}

#[allow(unused)]
#[derive(Debug)]
pub struct BigError {
    a: String,
    b: Vec<String>,
    c: [u8; 64],
    d: u64,
}

fn main() -> Result<(), anyhow::Error> {
    println!("size of anyhow::Error is {}", size_of::<anyhow::Error>());
    println!("size of std::io::Error is {}", size_of::<std::io::Error>());
    println!(
        "size of std::num::ParseIntError is {}",
        size_of::<ParseIntError>()
    );
    println!(
        "size of serde_json::Error is {}",
        size_of::<serde_json::Error>()
    );
    println!("size of string is {}", size_of::<String>());
    let filename = "no-existent-file.txt";

    // Error 大小由数据中最长的字段决定的，正常是最大长度 + tag，这里 string 是 24 字节
    // tag 复用了 string 最前面的字节，所以还是 24
    println!("size of MyError is {}", size_of::<MyError>());

    let _fd = fs::File::open(filename).context(format!("Failed to open file: {}", filename))?;

    fail_with_error()?;
    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}
