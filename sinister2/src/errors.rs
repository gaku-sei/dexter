#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("eframe error: {0}")]
    Eframe(#[from] eframe::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
