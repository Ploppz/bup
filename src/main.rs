use iced::{button, pick_list, text_input};
use iced::{Align, Button, Column, Element, PickList, Row, Text, TextInput};
use iced::{Application, Command, Settings, Font, HorizontalAlignment, Length};
use iced_graphics::{Backend, Renderer};
use iced_native::Widget;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use app_dirs::*;

mod path;
use path::FilePicker;

pub const TEXT_SIZE: u16 = 12;
pub const BUTTON_PAD: u16 = 2;

// Fonts
const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/icons.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .width(Length::Units(20))
        .horizontal_alignment(HorizontalAlignment::Center)
        .size(20)
}

fn edit_icon() -> Text {
    icon('\u{F303}')
}

fn delete_icon() -> Text {
    icon('\u{F1F8}')
}

const APP_INFO: AppInfo = AppInfo {
    name: "bup",
    author: "Erlend Langseth",
};

pub fn main() -> iced::Result {
    println!("{:?}", get_app_root(AppDataType::UserConfig, &APP_INFO));
    Ui::run(Settings::default())
}

#[derive(Default, Serialize, Deserialize)]
pub struct Ui {
    directories: Vec<Backup>,
    #[serde(skip)]
    new_button: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewBackup,
    Backup(usize, BackupMessage),
}

impl Application for Ui {
    type Executor = iced_native::executor::Tokio;
    type Message = Message;
    type Flags = ();
    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Ui - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NewBackup => {
                self.directories.push(Backup::default());
                Command::none()
            }
            Message::Backup(i, message) => self.directories[i]
                .update(message)
                .map(move |msg| Message::Backup(i, msg)),
        }
    }

    fn view(&mut self) -> Element<Message> {
        let directories: Element<_> = self
            .directories
            .iter_mut()
            .enumerate()
            .fold(Column::new().spacing(20), |column, (i, directory)| {
                column.push(directory.view().map(move |msg| Message::Backup(i, msg)))
            })
            .into();
        Column::new()
            .push(directories)
            .push(
                Button::new(
                    &mut self.new_button,
                    Text::new("New directory").size(TEXT_SIZE),
                )
                .padding(BUTTON_PAD)
                .on_press(Message::NewBackup),
            )
            .padding(20)
            .align_items(Align::Center)
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
        Row::new()
            .push(PickList::<_, DuplicationMessage>::new(
                &mut self.pick_list,
                options,
                None,
                DuplicationMessage::SelectKind,
            ))
            .into()
    }
}

//
//
// New attempt
//
//

// Vec<Backup>

#[derive(Debug, Clone)]
pub enum BackupMessage {
    SetName(String),

    NewSource,
    Source(usize, path::Message),
    DelSource(usize),

    NewExclude,
    SetExclude(usize, String),
    DelExclude(usize),
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Backup {
    pub name: String,
    /// Paths to include in the backup
    pub source: Vec<FilePicker>,
    /// Exclude pattern sent to `tar` via `--exclude`
    pub exclude: Vec<String>,
    pub dup: Vec<Duplication>,

    // State
    #[serde(skip)]
    s_name: text_input::State,
    #[serde(skip)]
    s_new_source: button::State,
    #[serde(skip)]
    s_exclude: Vec<text_input::State>,
    #[serde(skip)]
    s_new_exclude: button::State,
}

impl Backup {
    pub fn update(&mut self, message: BackupMessage) -> Command<BackupMessage> {
        match message {
            BackupMessage::SetName(name) => self.name = name,
            BackupMessage::NewSource => self.source.push(Default::default()),
            BackupMessage::Source(i, source) => {
                let command = self.source[i]
                    .update(source)
                    .map(move |msg| BackupMessage::Source(i, msg));
                return command;
            }
            BackupMessage::DelSource(i) => {
                self.source.remove(i);
            }
            BackupMessage::NewExclude => {
                self.exclude.push(Default::default());
                self.s_exclude.push(Default::default());
            }
            BackupMessage::SetExclude(i, exclude) => self.exclude[i] = exclude,
            BackupMessage::DelExclude(i) => {
                self.exclude.remove(i);
            }
        }
        Command::none()
    }
    pub fn view(&mut self) -> Element<BackupMessage> {
        Column::new()
            .padding(20)
            // .align_items(Align::Center)
            .push(
                TextInput::new(&mut self.s_name, "Name", &self.name, BackupMessage::SetName)
                    .size(TEXT_SIZE),
            )
            .push(
                Row::new()
                    // Sources
                    //
                    // Button::new(
                    // delete_button,
                    // Row::new()
                    // .spacing(10)
                    // .push(delete_icon())
                    // .push(Text::new("Delete")),
                    // )
                    // .on_press(TaskMessage::Delete)
                    // .padding(10)
                    // .style(style::Button::Destructive)
                    .push(
                        Column::new()
                            .push(Text::new("Sources").size(TEXT_SIZE))
                            .push(self.source.iter_mut().enumerate().fold(
                                Column::new(),
                                |column, (i, source)| {
                                    column.push(
                                        source
                                            .view(TEXT_SIZE, BUTTON_PAD)
                                            .map(move |msg| BackupMessage::Source(i, msg)),
                                    )
                                },
                            ))
                            .push(
                                Button::new(
                                    &mut self.s_new_source,
                                    Text::new("New source").size(TEXT_SIZE),
                                )
                                .padding(BUTTON_PAD)
                                .on_press(BackupMessage::NewSource),
                            ),
                    )
                    // Excludes (TODO)
                    .push(
                        Column::new()
                            .push(Text::new("Excludes").size(TEXT_SIZE))
                            .push(
                                self.exclude
                                    .iter_mut()
                                    .zip(self.s_exclude.iter_mut())
                                    .enumerate()
                                    .fold(Column::new(), |column, (i, (exclude, state))| {
                                        column.push(
                                            TextInput::new(
                                                state,
                                                "Exclude string",
                                                exclude,
                                                move |s| BackupMessage::SetExclude(i, s),
                                            )
                                            .size(TEXT_SIZE),
                                        )
                                    }),
                            )
                            .push(
                                Button::new(
                                    &mut self.s_new_exclude,
                                    Text::new("New exclude").size(TEXT_SIZE),
                                )
                                .padding(BUTTON_PAD)
                                .on_press(BackupMessage::NewExclude),
                            ),
                    ),
            )
            .into()
    }
}

trait ColumnExt<'a, M> {
    fn push_iter<I, E>(self, iter: I) -> Self
    where
        I: Iterator<Item = E>,
        E: Into<Element<'a, M>>;
}
impl<'a, M> ColumnExt<'a, M> for Column<'a, M> {
    fn push_iter<I, E>(mut self, iter: I) -> Self
    where
        I: Iterator<Item = E>,
        E: Into<Element<'a, M>>,
    {
        for item in iter {
            self = self.push(item);
        }
        self
    }
}

trait RowExt<'a, M> {
    fn push_iter<I, E>(self, iter: I) -> Self
    where
        I: Iterator<Item = E>,
        E: Into<Element<'a, M>>;
}
impl<'a, M> RowExt<'a, M> for Row<'a, M> {
    fn push_iter<I, E>(mut self, iter: I) -> Self
    where
        I: Iterator<Item = E>,
        E: Into<Element<'a, M>>,
    {
        for item in iter {
            self = self.push(item);
        }
        self
    }
}
