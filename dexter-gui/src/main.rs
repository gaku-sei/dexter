use dexter_core::{get_chapters, search};
use sixtyfps::{invoke_from_event_loop, ModelHandle, VecModel};
use std::{rc::Rc, sync::Arc};
use tokio::sync::mpsc;

sixtyfps::include_modules!();

#[tokio::main]
async fn main() {
    let (tx_mangas, mut rx_mangas) = mpsc::channel::<Vec<Manga>>(100);

    let tx_mangas = Arc::new(tx_mangas);

    let (tx_chapters, mut rx_chapters) = mpsc::channel::<Vec<Chapter>>(100);

    let tx_chapters = Arc::new(tx_chapters);

    let main_window = MainWindow::new();

    main_window.on_search_mangas(move |title| {
        let tx_mangas = tx_mangas.clone();

        tokio::spawn(async move {
            let search_response = search(title, 10).await;

            if tx_mangas
                .send(
                    search_response
                        .unwrap()
                        .data
                        .into_iter()
                        .map(|data| Manga {
                            title: data.attributes.title.en.into(),
                            id: data.id.into(),
                        })
                        .collect(),
                )
                .await
                .is_err()
            {
                println!("An error occured");
            };
        });
    });

    main_window.on_search_chapters(move |manga_id| {
        let tx_chapters = tx_chapters.clone();

        tokio::spawn(async move {
            let chapter_response = get_chapters(&manga_id, 10, Vec::new(), Vec::new()).await;

            if tx_chapters
                .send(
                    chapter_response
                        .unwrap()
                        .data
                        .into_iter()
                        .map(|data| Chapter {
                            title: data.attributes.title.into(),
                        })
                        .collect(),
                )
                .await
                .is_err()
            {
                println!("An error occured");
            };
        });
    });

    let main_window_weak = main_window.as_weak();

    tokio::spawn(async move {
        while let Some(mangas) = rx_mangas.recv().await {
            let main_window_weak_clone = main_window_weak.clone();

            invoke_from_event_loop(move || {
                let model = Rc::new(VecModel::from(mangas));

                main_window_weak_clone
                    .unwrap()
                    .set_mangas(ModelHandle::new(model))
            });
        }
    });

    let main_window_weak = main_window.as_weak();

    tokio::spawn(async move {
        while let Some(chapters) = rx_chapters.recv().await {
            let main_window_weak_clone = main_window_weak.clone();

            invoke_from_event_loop(move || {
                let model = Rc::new(VecModel::from(chapters));

                main_window_weak_clone
                    .unwrap()
                    .set_chapters(ModelHandle::new(model))
            });
        }
    });

    main_window.run();
}
