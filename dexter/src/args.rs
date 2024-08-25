use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct InteractiveSearch {
    /// Skips manga search and use manga id as reference
    #[clap(long)]
    pub manga_id: Option<String>,
    /// Used with the `--manga-id` and `--chapter-number` to refine chapter search
    #[clap(long)]
    pub volume_number: Option<String>,
    /// Skips chapter search and use this chapter number as reference, the `--manga-id` option must be provided too
    #[clap(long)]
    pub chapter_number: Option<String>,
    /// Accepts the default filename automatically
    #[clap(long, action)]
    pub accepts_default_filename: bool,
    /// Destination directory, defaults to the current directory
    #[clap(long)]
    pub outdir: Option<Utf8PathBuf>,
    /// Language to use
    #[clap(long, default_value = "en")]
    pub language: String,
    /// Max retries if image download fails
    #[clap(long, default_value_t = 3)]
    pub max_download_retries: u32,
}

#[derive(Parser, Debug)]
pub struct Search {
    /// Search for a manga by title
    #[clap(short, long)]
    pub title: String,
    /// Limit how many results are displayed (lower is faster)
    #[clap(short, long, default_value = "5")]
    pub limit: u32,
}

#[derive(Parser, Debug)]
pub struct Chapters {
    /// Display the chapters for a specified manga id
    #[clap(short, long)]
    pub manga_id: String,
    /// Limit how many chapters are displayed (lower is faster)
    #[clap(short, long, default_value = "100")]
    pub limit: u32,
    /// Specify which volume(s) you want to get data from
    #[clap(short, long)]
    pub volumes: Vec<String>,
    /// Specify which chapter(s) you want to get data from
    #[allow(clippy::struct_field_names)]
    #[clap(short, long)]
    pub chapters: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ImageLinks {
    /// Display the image links for a specified chapter id
    #[clap(short, long)]
    pub chapter_id: String,
}

#[derive(Parser, Debug)]
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
    /// Destination directory, defaults to the current directory
    #[clap(long)]
    pub outdir: Option<Utf8PathBuf>,
    /// Max retries if image download fails
    #[clap(long, default_value_t = 3)]
    pub max_download_retries: u32,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Interactive Search
    #[clap(alias = "is")]
    InteractiveSearch(InteractiveSearch),
    /// Search for mangas
    #[clap(alias = "s")]
    Search(Search),
    /// Search for chapters
    #[clap(alias = "c")]
    Chapters(Chapters),
    /// Display links to all the images contained in a chapter
    #[clap(alias = "il")]
    ImageLinks(ImageLinks),
    /// Download and pack all the images contained in a chapter
    #[clap(alias = "d")]
    Download(Download),
}

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Subcommands,
}
