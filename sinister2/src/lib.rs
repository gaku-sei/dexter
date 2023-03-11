use camino::Utf8PathBuf;
use dexter_core::{download_images, get_chapters, get_manga, search, ImageDownloadEvent};
use events::{BackendEvent, UiEvent};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};
use widgets::{manga_search, manga_view, MangaSearch, MangaView};

pub use crate::errors::{Error, Result};

mod errors;
mod events;
mod widgets;

pub fn run() -> Result<()> {
    let options = eframe::NativeOptions::default();
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let (backend_event_sender, backend_event_receiver) = broadcast::channel(32);
    let (ui_event_sender, mut ui_event_receiver) = mpsc::unbounded_channel::<(egui::Context, _)>();

    let _handle = tokio::spawn(async move {
        while let Some((_ctx, event)) = ui_event_receiver.recv().await {
            match event {
                UiEvent::MangaSearch(mangas_search) => {
                    let search_response = search(mangas_search, 50)
                        .await
                        .map_err(|err| {
                            error!("search manga error: {err}");
                            err
                        })
                        .ok();

                    if let Err(err) =
                        backend_event_sender.send(BackendEvent::MangaSearchDone(search_response))
                    {
                        warn!("backend channel closed: {err}");
                    }

                    // ctx.request_repaint();
                }
                UiEvent::DisplayManga(manga_id) => {
                    if let Err(err) =
                        backend_event_sender.send(BackendEvent::DisplayManga(manga_id.clone()))
                    {
                        warn!("backend channel closed: {err}");
                    }
                    // ctx.request_repaint();
                    info!("getting manga {manga_id}");
                    let manga = get_manga(manga_id).await.unwrap();
                    let volumes: [&str; 0] = [];
                    let chapters: [&str; 0] = [];
                    info!("getting chapters");
                    let chapters = get_chapters(manga.data.id.clone(), 60, volumes, chapters)
                        .await
                        .unwrap();
                    info!("got manga and chapters");
                    if let Err(err) =
                        backend_event_sender.send(BackendEvent::MangaReceived((manga, chapters)))
                    {
                        warn!("backend channel closed: {err}");
                    }
                    // ctx.request_repaint();
                }
                UiEvent::ChapterDownload(chapter_id) => {
                    let (tx, mut rx) = mpsc::channel(32);
                    let chapter_id_task = chapter_id.clone();
                    let backend_event_sender = backend_event_sender.clone();
                    tokio::spawn(async move {
                        let mut progress = 0.0;
                        let mut size = 0.0;
                        while let Some(event) = rx.recv().await {
                            match event {
                                ImageDownloadEvent::Init(s) => size = s as f32,
                                ImageDownloadEvent::Done => {
                                    debug!("{chapter_id_task} download progress: 100%");
                                    if let Err(err) = backend_event_sender
                                        .send(BackendEvent::ChapterDownloadProgress(None))
                                    {
                                        warn!("backend channel closed: {err}");
                                    }
                                    // ctx.request_repaint();
                                }
                                ImageDownloadEvent::Download | ImageDownloadEvent::Zip => {
                                    progress += 1.0;
                                    debug!(
                                        "{chapter_id_task} download progress: {:.0}%",
                                        progress / (size * 2.0) * 100.0
                                    );
                                    if let Err(err) = backend_event_sender.send(
                                        BackendEvent::ChapterDownloadProgress(Some((
                                            chapter_id_task.clone(),
                                            progress / (size * 2.0) * 100.0,
                                        ))),
                                    ) {
                                        warn!("backend channel closed: {err}");
                                    }
                                    // ctx.request_repaint();
                                }
                            }
                        }
                    });
                    let cbz = download_images(&chapter_id, 10, tx).await.unwrap();
                    let path = Utf8PathBuf::try_from(home::home_dir().unwrap())
                        .unwrap()
                        .join("Downloads")
                        .join(format!("{chapter_id}.cbz"));
                    info!("{chapter_id}.cbz downloaded");
                    if let Err(err) = cbz.write_to_path(path) {
                        error!("cbz error: {err}");
                    }
                }
            }
        }
    });

    eframe::run_native(
        "Sinister",
        options,
        Box::new(|cc| Box::new(Sinister::new(cc, ui_event_sender, backend_event_receiver))),
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct Sinister {
    manga_search: MangaSearch,
    ui_event_sender: mpsc::UnboundedSender<(egui::Context, UiEvent)>,
    backend_event_receiver: broadcast::Receiver<BackendEvent>,
    current_manga_view: Option<MangaView>,
}

impl Sinister {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        ui_event_sender: mpsc::UnboundedSender<(egui::Context, UiEvent)>,
        backend_event_receiver: broadcast::Receiver<BackendEvent>,
    ) -> Self {
        Self {
            manga_search: MangaSearch::new(
                ui_event_sender.clone(),
                backend_event_receiver.resubscribe(),
            ),
            ui_event_sender,
            backend_event_receiver,
            current_manga_view: None,
        }
    }

    fn handle_backend_event(&mut self, _ctx: egui::Context) {
        if let Ok(event) = self.backend_event_receiver.try_recv() {
            match event {
                BackendEvent::DisplayManga(manga_id) => {
                    self.current_manga_view = Some(MangaView::new(
                        manga_id,
                        self.ui_event_sender.clone(),
                        self.backend_event_receiver.resubscribe(),
                    ));
                    // ctx.request_repaint();
                }
                BackendEvent::MangaReceived((manga, chapters)) => {
                    if let Some(manga_view) = &mut self.current_manga_view {
                        manga_view.set_manga(manga).set_chapters(chapters);
                    } else {
                        error!(
                            "{}: received chapters and manga but none was set",
                            manga.data.id
                        );
                    }
                    // ctx.request_repaint();
                }
                event => {
                    debug!("unhandled backend event: {event:?}");
                }
            }
        }
    }
}

impl eframe::App for Sinister {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Using continuous mode to keep the ui is sync with the state
        ctx.request_repaint();
        self.handle_backend_event(ctx.clone());
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(current_manga_view) = &mut self.current_manga_view {
                ui.add(manga_view(current_manga_view));
            } else {
                ui.add(manga_search(&mut self.manga_search));
            }
        });
    }
}
