use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("protocol error: {0}")]
    Protocol(String),

    /// The router rejected a command (`!trap`).
    #[error("router error: {message}")]
    Trap {
        message: String,
        category: Option<String>,
    },

    /// The router closed the session (`!fatal`) or the connection dropped.
    #[error("fatal: {0}")]
    Fatal(String),

    #[error("connection closed")]
    Closed,

    #[error("login failed: {0}")]
    Login(String),
}
