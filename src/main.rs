use iced::{button, pick_list, scrollable, text_input};
use iced::{Align, Button, Column, Container, Element, PickList, Row, Scrollable, Text, TextInput};
use iced::{
    Application, Background, Color, Command, Font, HorizontalAlignment, Length, Settings, Size,
};
use uuid::Uuid;
// use iced_graphics::{Backend, Renderer};
use iced_native::{layout::Node, Overlay, Point, Widget};
use iced_wgpu::{Backend, Renderer};
use itertools::izip;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

mod backup;
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
pub const H3_SIZE: u16 = 24;
pub const BUTTON_PAD: u16 = 2;

pub type RepoSettings = rdedup_lib::settings::Repo;

pub use config::*;
mod config {
    use super::*;
    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct Config {
        pub repos: Vec<Repo>,
        pub targets: Vec<Target>,

        pub selected_repo: Option<Opt<RepoOption>>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct Repo {
        /// Needs a unique ID, since it's linked to by Targets, and the name (and maybe home) can
        /// be changed.
        pub id: Uuid,
        pub name: String,
        pub home: PathBuf,
        // pub settings: RepoSettings,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct Target {
        pub name: String,
        /// Paths to include in the backup
        pub sources: Vec<Option<PathBuf>>,
        /// Exclude pattern sent to `tar` via `--exclude`
        pub excludes: Vec<String>,
        pub duplication: Vec<Duplication>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Duplication {
        interval: Duration,
        kind: DuplicationKind,
    }
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum DuplicationKind {
        Disk { path: PathBuf },
        // TODO S3
        // TODO Syncthing?
    }
}

#[derive(Clone, Debug, Eq, Serialize, Deserialize)]
pub struct Opt<T> {
    name: String,
    value: T,
}
impl<T: PartialEq> PartialEq<Self> for Opt<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T> std::fmt::Display for Opt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RepoOption {
    New,
    Select(Uuid),
}

fn repo_options(repos: &[config::Repo]) -> Vec<Opt<RepoOption>> {
    std::iter::once(Opt {
        name: "New repo...".to_string(),
        value: RepoOption::New,
    })
    .chain(repos.iter().map(|repo| Opt {
        name: format!("{} {}", Icon::Repo, repo.name),
        value: RepoOption::Select(repo.id),
    }))
    .collect()
}

pub fn main() -> iced::Result {
    Ui::run(Settings::default())
}

/// Application state for different scenes
pub enum Scene {
    Overview {
        list: Vec<ListItemState>,
        new_button: button::State,
        selected_target: Option<usize>,
        s_open_settings: button::State,
        // The `None` means "New"
        s_repo_pick_list: pick_list::State<Opt<RepoOption>>,
    },
    CreateTarget {
        editor: Editor,
    },
    CreateRepo {
        name: String,
        home: Option<PathBuf>,

        error: Option<String>,
        s_cancel_button: button::State,
        s_save_button: button::State,
        s_name: text_input::State,
        s_home: FilePicker,
    },
    Edit {
        editor: Editor,
        dir_index: usize,
    },
    Settings {
        s_back_button: button::State,
    },
}
impl Scene {
    pub fn overview(config: &Config) -> Scene {
        Scene::Overview {
            list: config
                .targets
                .iter()
                .map(|_| ListItemState::default())
                .collect(),
            new_button: Default::default(),
            selected_target: None,
            s_open_settings: Default::default(),
            s_repo_pick_list: Default::default(),
        }
    }
    pub fn create_directory() -> Scene {
        Scene::CreateTarget {
            editor: Editor::default(),
        }
    }
    pub fn create_repo() -> Scene {
        Scene::CreateRepo {
            name: String::new(),
            home: None,
            error: None,

            s_cancel_button: Default::default(),
            s_save_button: Default::default(),
            s_name: Default::default(),
            s_home: Default::default(),
        }
    }
    pub fn edit(dir_index: usize, config: &Config) -> Scene {
        let dir = config.targets[dir_index].clone();
        Scene::Edit {
            editor: Editor::with_target(dir),
            dir_index,
        }
    }
    pub fn settings() -> Scene {
        Scene::Settings {
            s_back_button: Default::default(),
        }
    }
}

pub struct Ui {
    config: Config,
    scene: Scene,
    s_scrollable: scrollable::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToOverview,
    NewDir,
    EditDir(usize),
    ListItem(usize, ListItemMessage),
    Editor(EditorMessage),
    OpenSettings,
    PickRepo(Opt<RepoOption>),

    // Repo editor (maybe make a new component)
    SetRepoName(String),
    SetRepoHome(PathBuf),
    SaveRepo,
    RepoHome(path::Message),
}

impl Application for Ui {
    type Executor = iced_native::executor::Tokio;
    type Message = Message;
    type Flags = ();
    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = if let Ok(config) = Config::load() {
            config
        } else {
            Config::default()
        };
        (
            Ui {
                scene: Scene::overview(&config),
                config,
                s_scrollable: Default::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Ui - Iced")
    }

    fn update(&mut self, message: Message, _clip: &mut iced::Clipboard) -> Command<Message> {
        match message {
            Message::ToOverview => {
                self.scene = Scene::overview(&self.config);
                Command::none()
            }
            Message::NewDir => {
                self.scene = Scene::create_directory();
                Command::none()
            }
            Message::EditDir(index) => {
                self.scene = Scene::edit(index, &self.config);
                Command::none()
            }
            Message::ListItem(i, msg) => match msg {
                ListItemMessage::Edit => {
                    self.scene = Scene::edit(i, &self.config);
                    Command::none()
                }
                ListItemMessage::Expand => {
                    match self.scene {
                        Scene::Overview {
                            ref mut selected_target,
                            ..
                        } => {
                            if selected_target.is_some() {
                                *selected_target = None
                            } else {
                                *selected_target = Some(i)
                            }
                        }
                        // Scene::Overview {selected_target: None} =>
                        _ => unreachable!(),
                    }
                    // TODO: expand
                    Command::none()
                }
            },
            Message::Editor(msg) => {
                match msg {
                    EditorMessage::Save => {
                        match &self.scene {
                            Scene::CreateTarget { editor } => {
                                if let Ok(()) = verify_target(&editor.target) {
                                    self.config.targets.push(editor.target.clone());
                                    self.scene = Scene::overview(&self.config);
                                }
                            }
                            Scene::Edit { editor, dir_index } => {
                                if let Ok(()) = verify_target(&editor.target) {
                                    self.config.targets[*dir_index] = editor.target.clone();
                                    self.scene = Scene::overview(&self.config);
                                }
                            }
                            _ => panic!(),
                        };
                    }
                    EditorMessage::Cancel => {
                        self.scene = Scene::overview(&self.config);
                    }
                    _ => (),
                }
                match &mut self.scene {
                    Scene::CreateTarget { editor, .. } | Scene::Edit { editor, .. } => {
                        editor.update(msg).map(Message::Editor)
                    }
                    // Possible because scene might change above
                    _ => Command::none(),
                }
            }
            Message::OpenSettings => {
                self.scene = Scene::settings();
                Command::none()
            }
            Message::PickRepo(repo) => {
                match repo.value {
                    RepoOption::New => self.scene = Scene::create_repo(),
                    RepoOption::Select(_) => self.config.selected_repo = Some(repo),
                }
                Command::none()
            }
            Message::SetRepoName(new_name) => match self.scene {
                Scene::CreateRepo { ref mut name, .. } => {
                    *name = new_name;
                    Command::none()
                }
                _ => Command::none(),
            },
            Message::SetRepoHome(new_home) => match self.scene {
                Scene::CreateRepo { ref mut home, .. } => {
                    *home = Some(new_home);
                    Command::none()
                }
                _ => Command::none(),
            },
            Message::SaveRepo => match &mut self.scene {
                Scene::CreateRepo {
                    name,
                    home,
                    ref mut error,
                    ..
                } => {
                    if let Some(home) = home {
                        self.config.repos.push(Repo {
                            id: Uuid::new_v4(),
                            name: name.clone(),
                            home: home.clone(),
                        });
                        self.scene = Scene::overview(&self.config);
                    } else {
                        *error = Some("Home path must be set".to_string());
                    }
                    Command::none()
                }
                _ => Command::none(),
            },
            Message::RepoHome(msg) => match &mut self.scene {
                Scene::CreateRepo {
                    ref mut home,
                    ref mut s_home,
                    ..
                } => {
                    if let path::Message::Path(ref path) = msg {
                        *home = Some(path.clone());
                    }
                    s_home.update(msg).map(Message::RepoHome)
                }
                _ => Command::none(),
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        let w: Container<Message> = match &mut self.scene {
            Scene::Overview {
                list,
                new_button,
                selected_target,
                s_open_settings,
                s_repo_pick_list,
            } => {
                let options = repo_options(&self.config.repos);
                let selected = self
                    .config
                    .selected_repo
                    .clone()
                    .and_then(|selected| options.iter().find(|opt| opt.value == selected.value))
                    .cloned();
                let mut header = Row::new()
                    .spacing(20)
                    .push(Text::new("BUP").size(H3_SIZE))
                    .push(
                        PickList::new(s_repo_pick_list, options, selected, Message::PickRepo)
                            .font(ICONS)
                            .width(Length::Units(150))
                            .style(style::Dropdown),
                    );

                if self.config.selected_repo.is_some() {
                    header = header.push(
                        Button::new(new_button, Text::new("NEW BUP").size(TEXT_SIZE - 4))
                            .style(style::Button::Primary)
                            .on_press(Message::NewDir),
                    );
                }
                header = header.push(
                    Container::new(
                        Row::new().push(
                            Button::new(s_open_settings, Icon::Settings.text())
                                .padding(4)
                                .style(style::Button::Icon {
                                    hover_color: Color::WHITE,
                                })
                                .on_press(Message::OpenSettings),
                        ),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                );

                let mut overview: Column<Message> = Column::new().spacing(20);
                for (i, (target, state)) in
                    self.config.targets.iter().zip(list.iter_mut()).enumerate()
                {
                    let is_selected = selected_target.map(|s| s == i).unwrap_or(false);
                    overview = overview.push(
                        state
                            .view(&target, is_selected)
                            .map(move |msg| Message::ListItem(i, msg)),
                    );
                }

                Container::new(
                    Column::new()
                        .push(header)
                        .push(Scrollable::new(&mut self.s_scrollable).push(overview)),
                )
            }
            Scene::CreateTarget { editor } | Scene::Edit { editor, .. } => {
                // Center the editor
                Container::new(editor.view().map(Message::Editor))
                    .padding(50)
                    .align_x(Align::Center)
                    .width(Length::Fill)
                    .height(Length::Fill)
            }
            Scene::CreateRepo {
                name,
                home,
                error,
                ref mut s_cancel_button,
                ref mut s_save_button,
                ref mut s_name,
                ref mut s_home,
            } => Container::new(
                Column::new()
                    .padding(20)
                    .spacing(20)
                    .push(
                        Row::new().spacing(8).push(Icon::Repo.h3()).push(
                            TextInput::new(s_name, "Repo name", &name, Message::SetRepoName)
                                .style(style::TextInput)
                                .size(H3_SIZE),
                        ),
                    )
                    .push(
                        Row::new()
                            .spacing(8)
                            .push(Text::new("Home directory"))
                            .push(
                                s_home
                                    .view(home.as_ref().map(|x| x.as_path()), TEXT_SIZE, BUTTON_PAD)
                                    .map(Message::RepoHome),
                            ),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .spacing(10)
                                .push(
                                    Button::new(
                                        s_cancel_button,
                                        Text::new("CANCEL").size(TEXT_SIZE - 4),
                                    )
                                    .padding(8)
                                    .style(style::Button::Text)
                                    .on_press(Message::ToOverview),
                                )
                                .push(
                                    Button::new(
                                        s_save_button,
                                        Text::new("SAVE").size(TEXT_SIZE - 4),
                                    )
                                    .padding(8)
                                    .style(style::Button::Primary)
                                    .on_press(Message::SaveRepo),
                                ),
                        )
                        .width(Length::Fill)
                        .align_x(Align::End),
                    ),
            )
            .padding(50)
            .align_x(Align::Center)
            .width(Length::Fill)
            .height(Length::Fill),
            Scene::Settings { s_back_button } => Container::new(
                Column::new().push(
                    Button::new(s_back_button, Text::new("BACK").size(TEXT_SIZE - 4))
                        .style(style::Button::Text)
                        .on_press(Message::ToOverview),
                ),
            ),
        };
        // To apply a global style
        Container::new(w)
            .style(style::MenuContainer)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(15)
            .into()
    }
}

#[derive(Default)]
pub struct Editor {
    target: Target,
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

    s_scrollable: scrollable::State,
}
impl Editor {
    pub fn with_target(target: Target) -> Self {
        Self {
            // Review; One must manually make sure that the lists of states have the same length as
            // thet lists of values (or other state lists)
            s_exclude: vec![Default::default(); target.excludes.len()],
            s_delete_exclude_button: vec![Default::default(); target.excludes.len()],
            s_source: vec![Default::default(); target.sources.len()],
            s_delete_source_button: vec![Default::default(); target.sources.len()],
            target,
            ..Default::default()
        }
    }
    pub fn view<'a>(&'a mut self) -> Element<'a, EditorMessage> {
        let mut x = Column::new()
            .padding(20)
            .spacing(20)
            // .align_items(Align::Center)
            .push(
                Row::new().spacing(8).push(Icon::Folder.h3()).push(
                    TextInput::new(
                        &mut self.s_name,
                        "Name",
                        &self.target.name,
                        EditorMessage::SetName,
                    )
                    .style(style::TextInput)
                    .size(H3_SIZE),
                ),
            )
            // Sources
            .push(
                Container::new({
                    let mut col = Column::new().push(
                        Row::new().spacing(20).push(h3("Sources")).push(
                            // TODO: icon button
                            Button::new(&mut self.s_new_source, Icon::New.text())
                                .padding(4)
                                .style(style::Button::Icon {
                                    hover_color: Color::WHITE,
                                })
                                .on_press(EditorMessage::NewSource),
                        ),
                    );
                    for (i, (source, del_button, file_picker)) in izip!(
                        &self.target.sources,
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
                    col
                })
                .width(Length::FillPortion(1)),
            )
            // Excludes
            .push(
                Container::new(
                    Column::new()
                        .push(
                            Row::new().spacing(20).push(h3("Excludes")).push(
                                Button::new(&mut self.s_new_exclude, Icon::New.text())
                                    .style(style::Button::Icon {
                                        hover_color: Color::WHITE,
                                    })
                                    .padding(BUTTON_PAD)
                                    .on_press(EditorMessage::NewExclude),
                            ),
                        )
                        .push(
                            self.target
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
                                                        move |s| EditorMessage::SetExclude(i, s),
                                                    )
                                                    .style(style::TextInput)
                                                    .size(TEXT_SIZE),
                                                )
                                                .push(
                                                    Button::new(del_button, Icon::Delete.text())
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
                        ),
                )
                .width(Length::FillPortion(1)),
            )
            .push(
                Container::new(
                    Row::new()
                        .spacing(10)
                        .push(
                            Button::new(
                                &mut self.s_cancel_button,
                                Text::new("CANCEL").size(TEXT_SIZE - 4),
                            )
                            .padding(8)
                            .style(style::Button::Text)
                            .on_press(EditorMessage::Cancel),
                        )
                        .push(
                            Button::new(
                                &mut self.s_save_button,
                                Text::new("SAVE").size(TEXT_SIZE - 4),
                            )
                            .padding(8)
                            .style(style::Button::Primary)
                            .on_press(EditorMessage::Save),
                        ),
                )
                .width(Length::Fill)
                .align_x(Align::End),
            );
        if let Some(ref error) = self.error {
            x = x.push(Text::new(error).color(Color::from_rgb(0.5, 0.0, 0.0)))
        }
        let x = Container::new(x)
            .style(style::EditorContainer)
            .width(Length::Fill)
            .max_width(1000)
            .height(Length::Shrink);
        let x = Scrollable::new(&mut self.s_scrollable).push(x);
        x.into()
    }
    pub fn update(&mut self, message: EditorMessage) -> Command<EditorMessage> {
        match message {
            EditorMessage::SetName(name) => self.target.name = name,
            EditorMessage::NewSource => {
                self.target.sources.push(Default::default());
                self.s_delete_source_button.push(Default::default());
                // Review; I forgot once to put the following line here
                // Makes the UI malfunction due to how I izip! the iterators
                self.s_source.push(Default::default());
            }
            EditorMessage::Source(i, msg) => {
                if let path::Message::Path(ref path) = msg {
                    self.target.sources[i] = Some(path.clone());
                }
                return self.s_source[i]
                    .update(msg)
                    .map(move |msg| EditorMessage::Source(i, msg));
            }
            EditorMessage::DelSource(i) => {
                self.target.sources.remove(i);
            }
            EditorMessage::NewExclude => {
                self.target.excludes.push(Default::default());
                self.s_exclude.push(Default::default());
                self.s_delete_exclude_button.push(Default::default());
            }
            EditorMessage::SetExclude(i, exclude) => self.target.excludes[i] = exclude,
            EditorMessage::DelExclude(i) => {
                self.target.excludes.remove(i);
            }
            EditorMessage::Save => {
                // Show eventual error message
                if let Err(error) = verify_target(&self.target) {
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
    s_button2: button::State,
}
impl ListItemState {
    pub fn view(&mut self, dir: &Target, selected: bool) -> Element<ListItemMessage> {
        let header = Row::new()
            .height(Length::Units(36))
            .width(Length::Fill)
            .push(
                Container::new(Text::new(&dir.name).size(TEXT_SIZE))
                    .align_y(Align::Center)
                    .align_x(Align::Start)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .push(
                Container::new(
                    Button::new(&mut self.s_button2, Icon::Edit.text())
                        .padding(6)
                        .style(style::Button::Icon {
                            hover_color: Color::WHITE,
                        })
                        .on_press(ListItemMessage::Edit),
                )
                .align_x(Align::End)
                .width(Length::Fill),
            );
        let mut column = Column::new();
        column = column.push(
            Button::new(&mut self.s_button, header)
                .on_press(ListItemMessage::Expand)
                .style(style::ListItemHeader { selected }),
        );
        if selected {
            column = column.push(
                Container::new(Text::new("Details goes here"))
                    .style(style::ListItemExpanded)
                    .width(Length::Fill)
                    .padding(10),
            );
        }

        column.into()
    }
}
#[derive(Clone, Debug)]
pub enum ListItemMessage {
    Expand,
    Edit,
}

fn verify_target(dir: &Target) -> Result<(), String> {
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

// Persistent state

fn config_path() -> std::path::PathBuf {
    let mut path = if let Some(project_dirs) = directories_next::ProjectDirs::from("", "", "Bup") {
        project_dirs.data_dir().into()
    } else {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::new())
    };

    path.push("config.json");

    path
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(config_path())?;

        Ok(serde_json::from_str(&contents)?)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        use std::io::Write;
        let json = serde_json::to_string_pretty(&self)?;

        let path = config_path();
        println!("Saving to path: {}", path.display());

        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }

        {
            let mut file = std::fs::File::create(path)?;

            file.write_all(json.as_bytes())?;
        }

        Ok(())
    }
}
impl Drop for Ui {
    fn drop(&mut self) {
        let result = self.config.save();
        if let Err(e) = result {
            eprintln!("Error saving state: {}", e);
        }
    }
}
