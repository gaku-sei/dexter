#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("send image download event error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<crate::api::archive_download::Event>),

    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("cbz error: {0}")]
    Cbz(#[from] eco_cbz::Error),

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("reqwest middleware error: {0}")]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),

    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
