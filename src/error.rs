use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化/反序列化错误: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("时间处理错误: {0}")]
    Chrono(#[from] chrono::ParseError),

    #[error("读写锁错误: {0}")]
    Poison(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("非法输入: {0}")]
    InvalidInput(String),

    #[error("状态错误: {0}")]
    State(String),

    #[error("其他错误: {0}")]
    Other(String),
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self { Self::InvalidInput(e.to_string()) }
}

pub type Result<T> = std::result::Result<T, Error>;
