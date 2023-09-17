#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error: {0}")]
    Generic(String),

    #[error("Glob error: {0}")]
    Glob(#[from] glob::GlobError),

    #[error("Glob pattern error: {0}")]
    GlobPattern(#[from] glob::PatternError),

    #[error("Cbz error: {0}")]
    Image(#[from] cbz::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
