use iced::{button, pick_list, text_input};
use iced::{Align, Button, Column, Element, PickList, Row, Sandbox, Settings, Text, TextInput};
use iced_graphics::{Backend, Renderer};
use iced_native::Widget;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use app_dirs::*;

const APP_INFO: AppInfo = AppInfo {
    name: "Backup",
    author: "Erlend Langseth",
};

pub fn main() -> iced::Result {
    println!("{:?}", get_app_root(AppDataType::UserConfig, &APP_INFO));
    Ui::run(Settings::default())
}

#[derive(Default, Serialize, Deserialize)]
pub struct Ui {
    directories: Vec<Directory>,
    #[serde(skip)]
    new_button: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewDirectory,
    Directory(usize, DirectoryMessage),
}

impl Sandbox for Ui {
    type Message = Message;
    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Ui - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NewDirectory => {
                self.directories.push(Directory::default());
            }
            Message::Directory(i, message) => {
                self.directories[i].update(message);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let directories: Element<_> = self
            .directories
            .iter_mut()
            .enumerate()
            .fold(Column::new().spacing(20), |column, (i, directory)| {
                column.push(directory.view().map(move |msg| Message::Directory(i, msg)))
            })
            .into();
        Column::new()
            .push(directories)
            .push(
                Button::new(&mut self.new_button, Text::new("New directory")).on_press(Message::NewDirectory),
            )
            .padding(20)
            .align_items(Align::Center)
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum DirectoryMessage {
    NameChange(String),
    Dup (usize, DuplicationMessage)
}
#[derive(Default, Serialize, Deserialize)]
pub struct Directory {
    pub name: String,
    pub path: Option<PathBuf>,
    pub duplication: Vec<Duplication>,

    #[serde(skip)]
    input_state: text_input::State,
}
impl Directory {
    pub fn update(&mut self, message: DirectoryMessage) {
        match message {
            DirectoryMessage::NameChange(name) => self.name = name,
            DirectoryMessage::Dup(i, msg) => self.duplication[i].update(msg),
        }
    }
    pub fn view(&mut self) -> Element<DirectoryMessage> {
        Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(TextInput::new(
                &mut self.input_state,
                "Name",
                &self.name,
                DirectoryMessage::NameChange,
            ))
            .push(
                self.duplication
                    .iter_mut()
                    .enumerate()
                    .fold(Column::new(), |column, (i, dup)| column.push(dup.view().map(move |msg| DirectoryMessage::Dup (i, msg)))),
            )
            .into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Duplication {
    inner: Duplication2,

    #[serde(skip)]
    pick_list: pick_list::State<DuplicationKind>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Duplication2 {
    Disk(Option<PathBuf>),
}
impl Duplication2 {
    pub fn new(kind: DuplicationKind) -> Self {
        match kind {
            DuplicationKind::Disk => Self::Disk(None),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum DuplicationKind {
    Disk,
}
impl std::fmt::Display for DuplicationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DuplicationMessage {
    SelectKind(DuplicationKind),
}

impl Duplication {
    pub fn update(&mut self, message: DuplicationMessage) {
        match message {
            DuplicationMessage::SelectKind(kind) => self.inner = Duplication2::new(kind),
        }
    }
    pub fn view(&mut self) -> Element<DuplicationMessage> {
        let options = &[DuplicationKind::Disk][..];
        Row::new().push(PickList::<_,DuplicationMessage>::new(
            &mut self.pick_list,
            options,
            None,
            DuplicationMessage::SelectKind,
        ))
        .into()
    }
}
