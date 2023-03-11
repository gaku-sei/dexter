use dexter_core::{ChaptersResponse, MangaResponse, SearchResponse};

#[derive(Debug)]
pub enum UiEvent {
    MangaSearch(String),
    DisplayManga(String),
    ChapterDownload(String),
}

#[derive(Debug, Clone)]
pub enum BackendEvent {
    MangaSearchDone(Option<SearchResponse>),
    DisplayManga(String),
    MangaReceived((MangaResponse, ChaptersResponse)),
    ChapterDownloadProgress(Option<(String, f32)>),
}
