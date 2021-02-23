use iced::{button, pick_list, scrollable, text_input};
use iced::{Align, Button, Column, Container, Element, PickList, Row, Scrollable, Text, TextInput};
use iced::{Application, Color, Command, Font, HorizontalAlignment, Length, Settings};
use iced_graphics::{Backend, Renderer};
use iced_native::Widget;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use app_dirs::*;

mod icon;
mod path;
mod style;

pub use icon::Icon;
pub use path::FilePicker;

pub const TEXT_SIZE: u16 = 20;
pub const BUTTON_PAD: u16 = 2;

// Fonts
const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/agave.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .width(Length::Units(TEXT_SIZE))
        .size(TEXT_SIZE)
}
fn icon_h3(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .size(22)
        // .color([0.7,0.7,0.7])
        .font(ICONS)
        .width(Length::Units(20))
}
fn text<T: Into<String>>(text: T) -> Text {
    Text::new(text).font(ICONS).size(TEXT_SIZE)
}

fn h3<T: Into<String>>(text: T) -> Text {
    Text::new(text)
        .size(22)
        .color([0.7, 0.7, 0.7])
        .horizontal_alignment(HorizontalAlignment::Center)
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
    #[serde(skip)]
    s_scrollable: scrollable::State,
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
        Scrollable::new(&mut self.s_scrollable)
            .push(
                Column::new()
                    .push(directories)
                    .push(
                        Button::new(
                            &mut self.new_button,
                            Text::new("New directory").size(TEXT_SIZE),
                        )
                        .style(style::Button::Creation)
                        .padding(BUTTON_PAD)
                        .on_press(Message::NewBackup),
                    )
                    .padding(20)
                    .align_items(Align::Center),
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
    #[serde(skip)]
    s_delete_source_button: Vec<button::State>,
    #[serde(skip)]
    s_delete_exclude_button: Vec<button::State>,
}

impl Backup {
    pub fn update(&mut self, message: BackupMessage) -> Command<BackupMessage> {
        match message {
            BackupMessage::SetName(name) => self.name = name,
            BackupMessage::NewSource => {
                self.source.push(Default::default());
                self.s_delete_source_button.push(Default::default());
            }
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
                self.s_delete_exclude_button.push(Default::default());
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
                Row::new().push(Icon::Folder.h3()).push(
                    TextInput::new(&mut self.s_name, "Name", &self.name, BackupMessage::SetName)
                        .style(style::TextInput)
                        .size(20),
                ),
            )
            .push(
                Row::new()
                    // Sources
                    .push(
                        Container::new({
                            let mut col = Column::new().push(h3("Sources"));
                            for (i, (source, del_button)) in self
                                .source
                                .iter_mut()
                                .zip(self.s_delete_source_button.iter_mut())
                                .enumerate()
                            {
                                col = col.push(
                                    Row::new()
                                        .push(
                                            source
                                                .view(TEXT_SIZE, BUTTON_PAD)
                                                .map(move |msg| BackupMessage::Source(i, msg)),
                                        )
                                        .push(
                                            Button::new(del_button, Icon::Delete.text())
                                                .on_press(BackupMessage::DelSource(i))
                                                .padding(0)
                                                .style(style::Button::Icon {
                                                    hover_color: Color::from_rgb(0.7, 0.2, 0.2),
                                                }),
                                        ),
                                );
                            }
                            col = col.push(
                                Button::new(
                                    &mut self.s_new_source,
                                    Text::new("New source").size(TEXT_SIZE),
                                )
                                .padding(BUTTON_PAD)
                                .style(style::Button::Creation)
                                .on_press(BackupMessage::NewSource),
                            );
                            col
                        })
                        .width(Length::FillPortion(1)),
                    )
                    // Excludes (TODO)
                    .push(
                        Container::new(
                            Column::new()
                                .push(h3("Excludes"))
                                .push(
                                    self.exclude
                                        .iter_mut()
                                        .zip(self.s_exclude.iter_mut())
                                        .zip(self.s_delete_exclude_button.iter_mut())
                                        .enumerate()
                                        .fold(
                                            Column::new(),
                                            |column, (i, ((exclude, state), del_button))| {
                                                column.push(
                                                    Row::new()
                                                        .push(
                                                            TextInput::new(
                                                                state,
                                                                "Exclude string",
                                                                exclude,
                                                                move |s| {
                                                                    BackupMessage::SetExclude(i, s)
                                                                },
                                                            )
                                                            .style(style::TextInput)
                                                            .size(TEXT_SIZE),
                                                        )
                                                        .push(
                                                            Button::new(
                                                                del_button,
                                                                Icon::Delete.text(),
                                                            )
                                                            .on_press(BackupMessage::DelExclude(i))
                                                            .padding(0)
                                                            .style(style::Button::Icon {
                                                                hover_color: Color::from_rgb(
                                                                    0.7, 0.2, 0.2,
                                                                ),
                                                            }),
                                                        ),
                                                )
                                            },
                                        ),
                                )
                                .push(
                                    Button::new(
                                        &mut self.s_new_exclude,
                                        Text::new("New exclude").size(TEXT_SIZE),
                                    )
                                    .style(style::Button::Creation)
                                    .padding(BUTTON_PAD)
                                    .on_press(BackupMessage::NewExclude),
                                ),
                        )
                        .width(Length::FillPortion(1)),
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
