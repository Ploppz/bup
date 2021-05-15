use iced::{button, pick_list, scrollable, text_input};
use iced::{Align, Button, Column, Container, Element, PickList, Row, Scrollable, Text, TextInput};
use iced::{
    Application, Color, Command, Font, HorizontalAlignment, Length, Settings, Subscription,
};
use itertools::izip;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};
use uuid::Uuid;

mod backup;
mod ext;
mod icon;
mod path;
mod style;
mod target_editor;
mod util;

pub use ext::*;
pub use icon::Icon;
pub use path::FilePicker;
pub use target_editor::*;
pub use util::*;

pub const TEXT_SIZE: u16 = 20;
pub const H3_SIZE: u16 = 24;
pub const BUTTON_PAD: u16 = 2;

pub type RepoSettings = rdedup_lib::settings::Repo;

lazy_static::lazy_static! {
    pub static ref SHOULD_EXIT: AtomicBool = AtomicBool::new(false);
}

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
        pub repo: Uuid,
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
impl RepoOption {
    fn id(&self) -> Option<Uuid> {
        match self {
            RepoOption::New => None,
            RepoOption::Select(id) => Some(*id),
        }
    }
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
    ctrlc::set_handler(move || {
        SHOULD_EXIT.store(true, std::sync::atomic::Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");
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
        editor: TargetEditor,
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
    EditTarget {
        editor: TargetEditor,
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
    pub fn create_directory(repo_id: Uuid) -> Scene {
        Scene::CreateTarget {
            editor: TargetEditor::new_target(repo_id),
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
        Scene::EditTarget {
            editor: TargetEditor::with_target(dir),
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
    /// Only used to check if application should exit
    Tick(Instant),
    ToOverview,
    NewDir,
    EditDir(usize),
    ListItem(usize, ListItemMessage),
    TargetEditor(TargetEditorMessage),
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

    fn should_exit(&self) -> bool {
        SHOULD_EXIT.load(std::sync::atomic::Ordering::Relaxed)
    }
    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(1)).map(Message::Tick)
    }

    fn title(&self) -> String {
        String::from("Ui - Iced")
    }

    fn update(&mut self, message: Message, _clip: &mut iced::Clipboard) -> Command<Message> {
        match message {
            Message::Tick(_) => Command::none(),
            Message::ToOverview => {
                self.scene = Scene::overview(&self.config);
                Command::none()
            }
            Message::NewDir => {
                if let Some(Opt {
                    value: RepoOption::Select(repo_id),
                    ..
                }) = self.config.selected_repo
                {
                    self.scene = Scene::create_directory(repo_id);
                }
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
            Message::TargetEditor(msg) => {
                match msg {
                    TargetEditorMessage::Save => {
                        match &self.scene {
                            Scene::CreateTarget { editor } => {
                                if let Ok(()) = verify_target(&editor.target) {
                                    self.config.targets.push(editor.target.clone());
                                    self.scene = Scene::overview(&self.config);
                                }
                            }
                            Scene::EditTarget { editor, dir_index } => {
                                if let Ok(()) = verify_target(&editor.target) {
                                    self.config.targets[*dir_index] = editor.target.clone();
                                    self.scene = Scene::overview(&self.config);
                                }
                            }
                            _ => panic!(),
                        };
                    }
                    TargetEditorMessage::Cancel => {
                        self.scene = Scene::overview(&self.config);
                    }
                    _ => (),
                }
                match &mut self.scene {
                    Scene::CreateTarget { editor, .. } | Scene::EditTarget { editor, .. } => {
                        editor.update(msg).map(Message::TargetEditor)
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
                let repo_options = repo_options(&self.config.repos);

                let mut button = Button::new(new_button, Text::new("NEW BUP").size(TEXT_SIZE - 4))
                    .style(style::Button::Primary);
                if self.config.selected_repo.is_some() {
                    button = button.on_press(Message::NewDir);
                }
                let mut header = Row::new()
                    .spacing(20)
                    .push(Text::new("BUP").size(H3_SIZE))
                    .push(
                        PickList::new(
                            s_repo_pick_list,
                            repo_options,
                            self.config.selected_repo.clone(),
                            Message::PickRepo,
                        )
                        .font(ICONS)
                        .width(Length::Units(150))
                        .style(style::Dropdown),
                    )
                    .push(button);

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

                let selected_repo_id = self
                    .config
                    .selected_repo
                    .clone()
                    .and_then(|opt| opt.value.id());
                let mut overview: Column<Message> = Column::new().spacing(20);
                for (i, (target, state)) in
                    self.config.targets.iter().zip(list.iter_mut()).enumerate()
                {
                    if let Some(selected_repo_id) = selected_repo_id {
                        if selected_repo_id == target.repo {
                            let is_selected = selected_target.map(|s| s == i).unwrap_or(false);
                            overview = overview.push(
                                state
                                    .view(&target, is_selected)
                                    .map(move |msg| Message::ListItem(i, msg)),
                            );
                        }
                    }
                }

                Container::new(
                    Column::new()
                        .push(header)
                        .push(Scrollable::new(&mut self.s_scrollable).push(overview)),
                )
            }
            Scene::CreateTarget { editor } | Scene::EditTarget { editor, .. } => {
                // Center the editor
                Container::new(editor.view().map(Message::TargetEditor))
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
                Container::new(
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
                            Row::new().spacing(8).push(Text::new("RDEDUP_HOME:")).push(
                                s_home
                                    .view(home.as_ref().map(|x| x.as_path()), TEXT_SIZE)
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
                .style(style::DialogContainer)
                .width(Length::Fill)
                .max_width(1000)
                .height(Length::Shrink),
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
