use std::ops::Not;

use dexter_core::{ChaptersResponse, MangaResponse};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, warn};

use crate::events::{BackendEvent, UiEvent};

#[derive(Debug)]
pub struct MangaView {
    id: String,
    manga: Option<MangaResponse>,
    chapters: Option<ChaptersResponse>,
    ui_event_sender: mpsc::UnboundedSender<(egui::Context, UiEvent)>,
    backend_event_receiver: broadcast::Receiver<BackendEvent>,
    chapter_download_progress: Option<(String, f32)>,
}

impl MangaView {
    pub fn new(
        manga_id: impl Into<String>,
        ui_event_sender: mpsc::UnboundedSender<(egui::Context, UiEvent)>,
        backend_event_receiver: broadcast::Receiver<BackendEvent>,
    ) -> Self {
        Self {
            id: manga_id.into(),
            manga: None,
            chapters: None,
            ui_event_sender,
            backend_event_receiver,
            chapter_download_progress: None,
        }
    }

    pub fn set_manga(&mut self, manga: MangaResponse) -> &mut Self {
        self.manga = Some(manga);
        self
    }

    pub fn set_chapters(&mut self, chapters: ChaptersResponse) -> &mut Self {
        self.chapters = Some(chapters);
        self
    }

    fn download_chapter(&self, ctx: egui::Context, chapter_id: &str) {
        if let Err(err) = self
            .ui_event_sender
            .send((ctx, UiEvent::ChapterDownload(chapter_id.to_string())))
        {
            warn!("ui channel closed: {err}");
        }
    }

    fn handle_backend_event(&mut self, _ctx: egui::Context) {
        if let Ok(event) = self.backend_event_receiver.try_recv() {
            match event {
                BackendEvent::ChapterDownloadProgress(chapter_download_progress) => {
                    self.chapter_download_progress = chapter_download_progress;
                    // ctx.request_repaint();
                }
                event => {
                    debug!("unhandled backend event: {event:?}");
                }
            }
        }
    }
}

pub fn manga_view_widget(ui: &mut egui::Ui, manga_view: &mut MangaView) -> egui::Response {
    manga_view.handle_backend_event(ui.ctx().clone());

    let response = ui.label(format!("Manga {}", manga_view.id));

    if let Some(manga) = &manga_view.manga {
        ui.label(&manga.data.attributes.title.en);
    }

    if let Some((chapter_id, progress)) = &manga_view.chapter_download_progress {
        ui.label(format!("Downloading {chapter_id}: {progress:.0}%"));
    }

    if let Some(chapters) = &manga_view.chapters {
        egui_extras::TableBuilder::new(ui)
            .column(egui_extras::Column::initial(500.0))
            .column(egui_extras::Column::initial(500.0))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("ID");
                });
                header.col(|ui| {
                    ui.heading("Title");
                });
            })
            .body(|mut body| {
                body.row(40.0, |mut row| {
                    row.col(|ui| {
                        for chapter in &chapters.data {
                            ui.label(&chapter.id);
                        }
                    });
                    row.col(|ui| {
                        for chapter in &chapters.data {
                            let response = ui.button(
                                if let Some(title) = chapter
                                    .attributes
                                    .title
                                    .as_deref()
                                    .and_then(|title| title.is_empty().not().then_some(title))
                                {
                                    title
                                } else {
                                    "unknown"
                                },
                            );
                            if response.clicked() {
                                manga_view.download_chapter(ui.ctx().clone(), &chapter.id);
                            }
                        }
                    });
                });
            });
    }

    response
}

pub fn manga_view(manga_view: &mut MangaView) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| manga_view_widget(ui, manga_view)
}
