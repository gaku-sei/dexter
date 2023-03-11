use dexter_core::SearchData;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

use crate::events::{BackendEvent, UiEvent};

#[derive(Debug)]
pub struct MangaSearch {
    text: String,
    ui_event_sender: mpsc::UnboundedSender<(egui::Context, UiEvent)>,
    loading: bool,
    backend_event_receiver: broadcast::Receiver<BackendEvent>,
    data: Option<Vec<SearchData>>,
}

impl MangaSearch {
    pub fn new(
        ui_event_sender: mpsc::UnboundedSender<(egui::Context, UiEvent)>,
        backend_event_receiver: broadcast::Receiver<BackendEvent>,
    ) -> Self {
        Self {
            text: String::new(),
            ui_event_sender,
            loading: false,
            backend_event_receiver,
            data: None,
        }
    }
}

impl MangaSearch {
    fn handle_backend_event(&mut self, _ctx: egui::Context) {
        if let Ok(event) = self.backend_event_receiver.try_recv() {
            match event {
                BackendEvent::MangaSearchDone(search_response) => {
                    self.loading = false;
                    if let Some(search_response) = search_response {
                        info!("received {} titles", search_response.data.len());
                        self.data = Some(search_response.data);
                    }
                    // ctx.request_repaint();
                }
                event => {
                    debug!("unhandled backend event: {event:?}");
                }
            }
        }
    }

    fn text_mut(&mut self) -> &mut String {
        &mut self.text
    }

    fn search(&mut self, ctx: egui::Context) {
        self.loading = true;
        if let Err(err) = self
            .ui_event_sender
            .send((ctx, UiEvent::MangaSearch(self.text.clone())))
        {
            warn!("ui channel closed: {err}");
        }
    }

    fn display_manga(&self, ctx: egui::Context, manga_id: impl Into<String>) {
        if let Err(err) = self
            .ui_event_sender
            .send((ctx, UiEvent::DisplayManga(manga_id.into())))
        {
            warn!("ui channel closed: {err}");
        }
    }

    fn is_loading(&self) -> bool {
        self.loading
    }

    fn data(&self) -> Option<&[SearchData]> {
        self.data.as_deref()
    }
}

fn manga_search_widget(ui: &mut egui::Ui, manga_search: &mut MangaSearch) -> egui::Response {
    manga_search.handle_backend_event(ui.ctx().clone());

    let response = ui.heading("Hello Sinister!");
    let search_text_edit = ui.add(egui::TextEdit::singleline(manga_search.text_mut()));
    let search_button = ui.button(if manga_search.is_loading() {
        "Loading..."
    } else {
        "Search"
    });

    if !manga_search.is_loading()
        && ((search_text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            || search_button.clicked())
    {
        manga_search.search(ui.ctx().clone());
    }

    if let Some(data) = manga_search.data() {
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
                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        for manga in data {
                            ui.label(&manga.id);
                        }
                    });
                    row.col(|ui| {
                        for manga in data {
                            let response = ui.button(&manga.attributes.title.en);
                            if response.clicked() {
                                manga_search.display_manga(ui.ctx().clone(), &manga.id);
                            }
                        }
                    });
                });
            });
    };

    response
}

pub fn manga_search(manga_search: &mut MangaSearch) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| manga_search_widget(ui, manga_search)
}
