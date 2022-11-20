#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    io::{Read, Seek},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use bytes::Bytes;
use cbz::{Cbz, CbzRead, CbzReader};
use iced::{
    executor, image, slider, window, Application, Column, Command, Element, Settings, Subscription,
    Text,
};
use iced_native::Event;

#[derive(Debug)]
struct CbzReaderReady {
    image_handle: image::Handle,
    image_viewer: image::viewer::State,
    index: usize,
    slider: slider::State,
}

#[derive(Debug)]
enum CbzReaderState {
    Init,
    Ready(CbzReaderReady),
}

#[derive(Debug)]
pub struct CbzReaderView<'a, R> {
    cbz: Arc<Mutex<CbzReader<'a, R>>>,
    cbz_len: usize,
    state: CbzReaderState,
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),
    SetImage(Bytes),
    SetImageError,
    SetIndex(u32),
}

#[derive(Debug)]
pub struct Flags<'a, R> {
    cbz: Arc<Mutex<CbzReader<'a, R>>>,
}

impl<'a, R> CbzReaderView<'a, R>
where
    R: Read + Seek,
{
    #[allow(clippy::unused_async)]
    async fn read_from_cbz(cbz: Arc<Mutex<CbzReader<'a, R>>>, index: usize) -> Result<Bytes> {
        let mut cbz = cbz.lock().unwrap();

        let bytes = cbz.read_to_bytes_by_index(index)?;

        Ok(bytes)
    }

    fn handle_cbz_bytes(result: Result<Bytes>) -> Message {
        match result {
            Ok(bytes) => Message::SetImage(bytes),
            Err(_) => Message::SetImageError,
        }
    }
}

impl<R> Application for CbzReaderView<'static, R>
where
    R: 'static + Read + Seek + Send,
{
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags<'static, R>;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let cbz_len = flags.cbz.lock().unwrap().len();

        let cbz_reader = Self {
            cbz: flags.cbz,
            cbz_len,
            state: CbzReaderState::Init,
        };

        let read_first_file_future = Self::read_from_cbz(cbz_reader.cbz.clone(), 0);

        (
            cbz_reader,
            Command::perform(read_first_file_future, Self::handle_cbz_bytes),
        )
    }

    fn title(&self) -> String {
        String::from("CbzReader - Iced")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::EventOccurred(Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key_code: iced::keyboard::KeyCode::Right,
                ..
            })) => {
                if let CbzReaderState::Ready(ref mut ready) = self.state {
                    if ready.index < self.cbz_len - 1 {
                        ready.index += 1;
                    }

                    return Command::perform(
                        Self::read_from_cbz(self.cbz.clone(), ready.index),
                        Self::handle_cbz_bytes,
                    );
                }
            }
            Message::EventOccurred(Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key_code: iced::keyboard::KeyCode::Left,
                ..
            })) => {
                if let CbzReaderState::Ready(ref mut ready) = self.state {
                    if ready.index > 0 {
                        ready.index -= 1;
                    }

                    return Command::perform(
                        Self::read_from_cbz(self.cbz.clone(), ready.index),
                        Self::handle_cbz_bytes,
                    );
                }
            }
            Message::SetIndex(new_index) => {
                if let CbzReaderState::Ready(ref mut ready) = self.state {
                    if ready.index < self.cbz_len {
                        ready.index = new_index as usize;
                    }

                    return Command::perform(
                        Self::read_from_cbz(self.cbz.clone(), ready.index),
                        Self::handle_cbz_bytes,
                    );
                }
            }
            Message::SetImage(image) => {
                let image_handle = image::Handle::from_memory(image.to_vec());

                match self.state {
                    CbzReaderState::Ready(ref mut ready) => ready.image_handle = image_handle,
                    CbzReaderState::Init => {
                        self.state = CbzReaderState::Ready(CbzReaderReady {
                            image_handle,
                            image_viewer: image::viewer::State::default(),
                            index: 0,
                            slider: slider::State::default(),
                        });
                    }
                }
            }

            Message::EventOccurred(_) | Message::SetImageError => (),
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::EventOccurred)
    }

    fn view(&mut self) -> Element<Message> {
        match self.state {
            CbzReaderState::Init => Column::new().push(Text::new("Loading").size(50)).into(),
            CbzReaderState::Ready(ref mut ready) => {
                let text = Text::new(format!("{}/{}", ready.index + 1, self.cbz_len));

                #[allow(clippy::cast_possible_truncation)]
                let max_value = if self.cbz_len > 0 {
                    (self.cbz_len - 1) as u32
                } else {
                    0
                };

                #[allow(clippy::cast_possible_truncation)]
                let slider = slider::Slider::new(
                    &mut ready.slider,
                    0..=max_value,
                    ready.index as u32,
                    Message::SetIndex,
                );

                let image = image::Viewer::new(&mut ready.image_viewer, ready.image_handle.clone());

                Column::new().push(text).push(slider).push(image).into()
            }
        }
    }
}

/// Runs the CBZ Reader application.
///
/// # Errors
///
/// IO errors will make this fail.
pub fn run<R>(cbz: CbzReader<'static, R>) -> Result<()>
where
    R: 'static + Read + Seek + Send,
{
    CbzReaderView::run(Settings {
        flags: Flags {
            cbz: Arc::new(Mutex::new(cbz)),
        },
        id: None,
        window: window::Settings::default(),
        default_font: None,
        default_text_size: 20,
        text_multithreading: false,
        antialiasing: false,
        exit_on_close_request: true,
        try_opengles_first: false,
    })?;

    Ok(())
}
