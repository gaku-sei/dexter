#[cfg(feature = "html5ever")]
pub use html5ever_parser::convert_to_imgs;
#[cfg(not(feature = "html5ever"))]
pub use tl_parser::convert_to_imgs;

#[cfg(feature = "html5ever")]
mod html5ever_parser;
#[cfg(not(feature = "html5ever"))]
mod tl_parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MobiVersion {
    Mobi6,
    Mobi8,
}

impl TryFrom<u32> for MobiVersion {
    type Error = anyhow::Error;

    fn try_from(version: u32) -> std::result::Result<Self, Self::Error> {
        match version {
            6 => Ok(Self::Mobi6),
            8 => Ok(Self::Mobi8),
            _ => anyhow::bail!("invalid version {version}"),
        }
    }
}
