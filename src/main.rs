use iced::{button, pick_list, scrollable, text_input};
use iced::{Align, Button, Column, Container, Element, PickList, Row, Scrollable, Text, TextInput};
use iced::{Application, Color, Command, Font, HorizontalAlignment, Length, Settings, Size};
// use iced_graphics::{Backend, Renderer};
use iced_native::{layout::Node, Overlay, Point, Widget};
use iced_wgpu::{Backend, Renderer};
use itertools::izip;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use app_dirs::*;

mod ext;
mod icon;
mod path;
mod style;
mod util;

pub use ext::*;
pub use icon::Icon;
pub use path::FilePicker;
pub use util::*;

pub const TEXT_SIZE: u16 = 20;
pub const BUTTON_PAD: u16 = 2;

pub use config::*;
mod config {
    use super::*;
    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct Directory {
        pub name: String,
        /// Paths to include in the backup
        pub sources: Vec<Option<PathBuf>>,
        /// Exclude pattern sent to `tar` via `--exclude`
        pub excludes: Vec<String>,
        // TODO duplications
    }
}

const APP_INFO: AppInfo = AppInfo {
    name: "bup",
    author: "Erlend Langseth",
};

pub fn main() -> iced::Result {
    println!("{:?}", get_app_root(AppDataType::UserConfig, &APP_INFO));
    Ui::run(Settings::default())
}

/// Application state for different scenes
pub enum Scene {
    Overview {
        list: Vec<ListItemState>,
        new_button: button::State,
    },
    Create {
        editor: Editor,
    },
    Edit {
        editor: Editor,
        dir_index: usize,
    },
}
impl Ui {
    pub fn enter_overview(&mut self) {
        self.scene = Scene::Overview {
            list: self
                .directories
                .iter()
                .map(|_| ListItemState::default())
                .collect(),
            new_button: Default::default(),
        };
    }
    pub fn enter_create(&mut self) {
        self.scene = Scene::Create {
            editor: Editor::default(),
        };
    }
    pub fn enter_edit(&mut self, dir_index: usize) {
        let dir = self.directories[dir_index].clone();
        self.scene = Scene::Edit {
            editor: Editor::with_directory(dir),
            dir_index,
        };
    }
}
impl Default for Scene {
    fn default() -> Scene {
        Scene::Overview {
            list: vec![],
            new_button: Default::default(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Ui {
    directories: Vec<Directory>,
    #[serde(skip)]
    scene: Scene,

    #[serde(skip)]
    s_scrollable: scrollable::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToOverview,
    NewDir,
    EditDir(usize),
    ListItem(usize, ListItemMessage),
    Editor(EditorMessage),
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
            Message::ToOverview => {
                self.enter_overview();
                Command::none()
            }
            Message::NewDir => {
                self.enter_create();
                Command::none()
            }
            Message::EditDir(index) => {
                self.enter_edit(index);
                Command::none()
            }
            Message::ListItem(i, msg) => match msg {
                ListItemMessage::Click => {
                    self.enter_edit(i);
                    Command::none()
                }
            },
            Message::Editor(msg) => {
                match msg {
                    EditorMessage::Save => {
                        let successful = match &self.scene {
                            Scene::Create { editor } => {
                                if let Ok(()) = verify_directory(&editor.directory) {
                                    self.directories.push(editor.directory.clone());
                                    self.enter_overview();
                                }
                            }
                            Scene::Edit { editor, dir_index } => {
                                if let Ok(()) = verify_directory(&editor.directory) {
                                    self.directories[*dir_index] = editor.directory.clone();
                                    self.enter_overview();
                                }
                            }
                            _ => panic!(),
                        };
                    }
                    EditorMessage::Cancel => {
                        self.enter_overview();
                    }
                    _ => (),
                }
                match &mut self.scene {
                    Scene::Create { editor, .. } | Scene::Edit { editor, .. } => editor.update(msg).map(Message::Editor)
,
                    // Possible because scene might change above
                    _ => Command::none(),
                }
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        match &mut self.scene {
            Scene::Overview { list, new_button } => {
                let directories: Element<_> = self
                    .directories
                    .iter()
                    .zip(list.iter_mut())
                    .enumerate()
                    .fold(
                        Column::new().spacing(20),
                        |column, (i, (directory, state))| {
                            column.push(
                                state
                                    .view(&directory)
                                    .map(move |msg| Message::ListItem(i, msg)),
                            )
                        },
                    )
                    .push(Button::new(new_button, Text::new("New")).on_press(Message::NewDir))
                    .into();
                Scrollable::new(&mut self.s_scrollable)
                    .push(directories)
                    .into()
            }
            Scene::Create { editor } | Scene::Edit { editor, .. } => {
                editor.view().map(Message::Editor)
            }
        }
    }
}

#[derive(Default)]
pub struct Editor {
    directory: Directory,
    error: Option<String>,

    s_name: text_input::State,
    s_new_source: button::State,
    s_new_exclude: button::State,
    s_save_button: button::State,
    s_cancel_button: button::State,

    s_exclude: Vec<text_input::State>,
    s_delete_exclude_button: Vec<button::State>,

    s_source: Vec<FilePicker>,
    s_delete_source_button: Vec<button::State>,
}
impl Editor {
    pub fn with_directory(directory: Directory) -> Self {
        Self {
            // Review; One must manually make sure that the lists of states have the same length as
            // thet lists of values (or other state lists)
            s_exclude: vec![Default::default(); directory.excludes.len()],
            s_delete_exclude_button: vec![Default::default(); directory.excludes.len()],
            s_source: vec![Default::default(); directory.sources.len()],
            s_delete_source_button: vec![Default::default(); directory.sources.len()],
            directory,
            ..Default::default()
        }
    }
    pub fn view<'a>(&'a mut self) -> Element<'a, EditorMessage> {
        let mut x = Column::new()
            .padding(20)
            // .align_items(Align::Center)
            .push(
                Row::new().push(Icon::Folder.h3()).push(
                    TextInput::new(
                        &mut self.s_name,
                        "Name",
                        &self.directory.name,
                        EditorMessage::SetName,
                    )
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
                            for (i, (source, del_button, file_picker)) in izip!(
                                &self.directory.sources,
                                &mut self.s_delete_source_button,
                                &mut self.s_source
                            )
                            .enumerate()
                            {
                                col = col.push(
                                    Row::new()
                                        .push(
                                            file_picker
                                                .view(
                                                    source.as_ref().map(|x| x.as_path()),
                                                    TEXT_SIZE,
                                                    BUTTON_PAD,
                                                )
                                                .map(move |msg| EditorMessage::Source(i, msg)),
                                        )
                                        .push(
                                            Button::new(del_button, Icon::Delete.text())
                                                .on_press(EditorMessage::DelSource(i))
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
                                .on_press(EditorMessage::NewSource),
                            );
                            col
                        })
                        .width(Length::FillPortion(1)),
                    )
                    .push(
                        Container::new(
                            Column::new()
                                .push(h3("Excludes"))
                                .push(
                                    self.directory
                                        .excludes
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
                                                                    EditorMessage::SetExclude(i, s)
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
                                                            .on_press(EditorMessage::DelExclude(i))
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
                                    .on_press(EditorMessage::NewExclude),
                                ),
                        )
                        .width(Length::FillPortion(1)),
                    ),
            )
            .push(
                Row::new()
                    .push(
                        Container::new(
                            Button::new(&mut self.s_save_button, Text::new("Save"))
                                .on_press(EditorMessage::Save),
                        )
                        .width(Length::FillPortion(1)),
                    )
                    .push(
                        Container::new(
                            Button::new(&mut self.s_cancel_button, Text::new("Cancel"))
                                .on_press(EditorMessage::Cancel),
                        )
                        .width(Length::FillPortion(1)),
                    ),
            );
        if let Some(ref error) = self.error {
            x = x .push(Text::new(error).color(Color::from_rgb(0.5, 0.0, 0.0)))
        }
        x.into()
    }
    pub fn update(&mut self, message: EditorMessage) -> Command<EditorMessage> {
        match message {
            EditorMessage::SetName(name) => self.directory.name = name,
            EditorMessage::NewSource => {
                self.directory.sources.push(Default::default());
                self.s_delete_source_button.push(Default::default());
                // Review; I forgot once to put the following line here
                // Makes the UI malfunction due to how I izip! the iterators
                self.s_source.push(Default::default());
            }
            EditorMessage::Source(i, msg) => {
                println!("EditorMessage::Source: {:?}", msg);
                if let path::Message::Path(ref path) = msg {
                    self.directory.sources[i] = Some(path.clone());
                }
                let command = self.s_source[i]
                    .update(msg)
                    .map(move |msg| EditorMessage::Source(i, msg));
                return command;
            }
            EditorMessage::DelSource(i) => {
                self.directory.sources.remove(i);
            }
            EditorMessage::NewExclude => {
                self.directory.excludes.push(Default::default());
                self.s_exclude.push(Default::default());
                self.s_delete_exclude_button.push(Default::default());
            }
            EditorMessage::SetExclude(i, exclude) => self.directory.excludes[i] = exclude,
            EditorMessage::DelExclude(i) => {
                self.directory.excludes.remove(i);
            }
            EditorMessage::Save => {
                // Show eventual error message
                if let Err(error) = verify_directory(&self.directory) {
                    self.error = Some(error);
                }
            }
            EditorMessage::Cancel => (),
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub enum EditorMessage {
    SetName(String),

    NewSource,
    Source(usize, path::Message),
    DelSource(usize),

    NewExclude,
    SetExclude(usize, String),
    DelExclude(usize),

    // Meant for outside
    /// Save button pressed
    Save,
    /// Cancel button pressed
    Cancel,
}

#[derive(Default, Debug, Clone)]
pub struct ListItemState {
    s_button: button::State,
}
impl ListItemState {
    pub fn view(&mut self, dir: &Directory) -> Element<ListItemMessage> {
        Button::new(&mut self.s_button, Text::new(&dir.name).size(TEXT_SIZE))
            .on_press(ListItemMessage::Click)
            .into()
    }
}
#[derive(Clone, Debug)]
pub enum ListItemMessage {
    Click,
}

fn verify_directory(dir: &Directory) -> Result<(), String> {
    if dir.name.is_empty() {
        return Err("Name should not be empty".to_string());
    }
    if dir.sources.is_empty() {
        return Err("Should have at least one source".to_string());
    }
    for source in &dir.sources {
        if source.is_none() {
            return Err("All sources should have a path".to_string());
        }
    }
    for exclude in &dir.excludes {
        if exclude.is_empty() {
            return Err("No exclude should be empty".to_string());
        }
    }
    Ok(())
}
