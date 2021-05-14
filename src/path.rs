use crate::style;
use iced::{button, pick_list, text_input};
use iced::{
    Align, Button, Column, Command, Element, Length, PickList, Row, Sandbox, Settings, Text,
    TextInput,
};
use nfd::Response;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

pub async fn open() -> anyhow::Result<PathBuf> {
    let result = tokio::task::spawn_blocking(|| {
        let result: nfd::Response = match nfd::open_pick_folder(None) {
            Ok(result) => result,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unable to unwrap data from new file dialog",
                ))
            }
        };

        let file_string: String = match result {
            Response::Okay(file_path) => file_path,
            Response::OkayMultiple(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Multiple files returned when one was expected",
                ))
            }
            Response::Cancel => {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "User cancelled file open",
                ))
            }
        };

        let mut result: PathBuf = PathBuf::new();
        result.push(Path::new(&file_string));

        if result.exists() {
            Ok(result)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "File does not exist",
            ))
        }
    })
    .await;
    Ok(result??)
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct FilePicker {
    #[serde(skip)]
    s_button: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Error(String),
    Path(PathBuf),
    SelectPath,
}
impl FilePicker {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::SelectPath => Command::perform(open(), |result| match result {
                Ok(path) => Message::Path(path),
                Err(e) => Message::Error(e.to_string()),
            }),
            Message::Path(path) => Command::none(),
            _ => Command::none(),
        }
    }
    pub fn view(&mut self, path: Option<&Path>, text_size: u16) -> Element<Message> {
        let text = match path {
            Some(path) => path.display().to_string(),
            None => format!("No folder selected"),
        };
        Row::new()
            .width(Length::Fill)
            .push(
                Button::new(&mut self.s_button, Text::new(text).size(text_size))
                    .padding(0)
                    .style(style::Button::Path)
                    .on_press(Message::SelectPath),
            )
            .into()
    }
}
