use clap::{crate_version, Clap, Subcommand};

#[derive(Clap, Debug)]
pub struct Search {
    /// Search for a manga by title
    #[clap(short, long)]
    pub title: String,
    /// Limit how many results are displayed (lower is faster)
    #[clap(short, long, default_value = "5")]
    pub limit: u16,
}

#[derive(Clap, Debug)]
pub struct Chapters {
    /// Display the chapters for a specified manga id
    #[clap(short, long)]
    pub manga_id: String,
    /// Limit how many chapters are displayed (lower is faster)
    #[clap(short, long, default_value = "100")]
    pub limit: u16,
    /// Specify which volume(s) you want to get data from
    #[clap(short, long)]
    pub volumes: Vec<String>,
    /// Specify which chapter(s) you want to get data from
    #[clap(short, long)]
    pub chapters: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct ImageLinks {
    /// Display the image links for a specified chapter id
    #[clap(short, long)]
    pub chapter_id: String,
}

#[derive(Clap, Debug)]
pub struct Download {
    /// Download and pack all the images for the provided chapter id
    #[clap(short, long)]
    pub chapter_id: String,
    /// Filename of the downloaded file archived
    #[clap(short, long, default_value = "chapter.cbz")]
    pub filename: String,
    /// Open the downloaded archive
    #[clap(short, long)]
    pub open: bool,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Search for mangas
    Search(Search),
    /// Search for chapters
    Chapters(Chapters),
    /// Display links to all the images contained in a chapter
    ImageLinks(ImageLinks),
    /// Download and pack all the images contained in a chapter
    Download(Download),
}

#[derive(Clap, Debug)]
#[clap(name = "dexter", version = crate_version!())]
pub struct Options {
    /// Search mangas
    #[clap(subcommand)]
    pub command: Subcommands,
}
