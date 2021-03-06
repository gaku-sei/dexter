#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    convert::TryFrom,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Result;
use dexter_core::read_by_index;
use iced::{
    executor, image, slider, Application, Column, Command, Element, Settings, Subscription, Text,
};
use iced_native::Event;

#[derive(Debug)]
struct CbzReaderReady {
    image_handle: image::Handle,
    image_viewer: image::viewer::State,
    index: i32,
    slider: slider::State,
}

#[derive(Debug)]
enum CbzReaderState {
    Init,
    Ready(CbzReaderReady),
}

#[derive(Debug)]
pub struct CbzReader {
    archive_size: i32,
    archive_path: PathBuf,
    state: CbzReaderState,
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),
    SetImage(Vec<u8>),
    SetImageError,
    SetIndex(i32),
}

#[derive(Debug, Default)]
pub struct Flags {
    archive_path: PathBuf,
    archive_size: i32,
}

impl CbzReader {
    async fn read_from_cbz(archive_path: impl AsRef<Path>, index: i32) -> Result<Vec<u8>> {
        let file = File::open(archive_path)?;

        let index = usize::try_from(index)?;

        read_by_index(file, index)
    }

    fn handle_cbz_bytes(result: Result<Vec<u8>>) -> Message {
        match result {
            Ok(bytes) => Message::SetImage(bytes),
            Err(_) => Message::SetImageError,
        }
    }
}

impl Application for CbzReader {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let cbz_reader = Self {
            archive_path: flags.archive_path.clone(),
            archive_size: flags.archive_size,
            state: CbzReaderState::Init,
        };

        (
            cbz_reader,
            Command::perform(
                Self::read_from_cbz(flags.archive_path, 0),
                Self::handle_cbz_bytes,
            ),
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
                    if ready.index < self.archive_size - 1 {
                        ready.index += 1;
                    }

                    return Command::perform(
                        Self::read_from_cbz(self.archive_path.clone(), ready.index),
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
                        Self::read_from_cbz(self.archive_path.clone(), ready.index),
                        Self::handle_cbz_bytes,
                    );
                }
            }
            Message::SetIndex(new_index) => {
                if let CbzReaderState::Ready(ref mut ready) = self.state {
                    if ready.index < self.archive_size {
                        ready.index = new_index;
                    }

                    return Command::perform(
                        Self::read_from_cbz(self.archive_path.clone(), ready.index),
                        Self::handle_cbz_bytes,
                    );
                }
            }
            Message::SetImage(image) => {
                let image_handle = image::Handle::from_memory(image);

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
                let text = Text::new(format!("{}/{}", ready.index + 1, self.archive_size));

                #[allow(clippy::range_minus_one)]
                let slider = slider::Slider::new(
                    &mut ready.slider,
                    0..=self.archive_size - 1,
                    ready.index,
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
pub fn run(archive_path: PathBuf, archive_size: i32) -> Result<()> {
    CbzReader::run(Settings {
        flags: Flags {
            archive_path,
            archive_size,
        },
        ..Default::default()
    })?;

    Ok(())
}
