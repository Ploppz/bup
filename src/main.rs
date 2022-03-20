#![feature(try_blocks)]
use anyhow::Context;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use iced::alignment::{Horizontal, Vertical};
use iced::{button, pick_list, scrollable, text_input};
use iced::{Application, Color, Command, Font, Length, Settings, Subscription};
use iced::{Button, Column, Container, Element, PickList, Row, Scrollable, Text, TextInput};
use indexmap::IndexMap;
use itertools::izip;
use rdedup_lib::Repo;
use serde::{Deserialize, Serialize};
use slog::{error, info, Logger};
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
use url::Url;
use uuid::Uuid;

mod ext;
mod icon;
mod log;
mod path;
mod rdedup;
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
        pub repos: IndexMap<Uuid, RepoConfig>,
        pub selected_repo: Option<Opt<RepoOption>>,
        pub passphrase_hash: Option<String>,
    }
    impl Config {
        pub fn selected_repo_mut(&mut self) -> Option<&mut RepoConfig> {
            if let Some(ref selected_repo) = self.selected_repo {
                if let Some(id) = selected_repo.value.id() {
                    self.repos.get_mut(&id)
                } else {
                    None
                }
            } else {
                None
            }
        }
        pub fn selected_repo(&self) -> Option<&RepoConfig> {
            self.selected_repo
                .as_ref()
                .and_then(|selected| selected.value.id().and_then(|id| self.repos.get(&id)))
        }
        pub fn find_repo(&self, id: Uuid) -> Option<&RepoConfig> {
            self.repos.get(&id)
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct RepoConfig {
        /// Needs a unique ID, since it's linked to by Targets, and the name (and maybe home) can
        /// be changed.
        pub id: Uuid,
        pub name: String,
        pub home: PathBuf,
        pub targets: Vec<Target>,
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

pub struct PreviousSnapshot {
    /// Superfluous in some cases
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub bytes: usize,
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

fn repo_options<'a, I: Iterator<Item = &'a RepoConfig>>(repos: I) -> Vec<Opt<RepoOption>> {
    std::iter::once(Opt {
        name: "New repo...".to_string(),
        value: RepoOption::New,
    })
    .chain(repos.map(|repo| Opt {
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
    Initial {
        passphrase1: String,
        passphrase2: String,
        error: Option<String>,
        s_pass1: text_input::State,
        s_pass2: text_input::State,
        s_confirm: button::State,
    },
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
        target_index: usize,
    },
    Settings {
        s_back_button: button::State,
    },
}
impl Scene {
    pub fn init() -> Scene {
        Scene::Initial {
            passphrase1: String::new(),
            passphrase2: String::new(),
            error: None,
            s_pass1: Default::default(),
            s_pass2: Default::default(),
            s_confirm: Default::default(),
        }
    }
    pub fn overview(config: &Config) -> Scene {
        let repo = config.selected_repo();
        let n_targets = repo.map(|repo| repo.targets.len()).unwrap_or(0);
        Scene::Overview {
            list: Vec::new(),
            new_button: Default::default(),
            selected_target: None,
            s_open_settings: Default::default(),
            s_repo_pick_list: Default::default(),
        }
    }
    pub fn create_target(repo_id: Uuid) -> Scene {
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
    pub fn edit(target_index: usize, config: &Config) -> Scene {
        let target = config.selected_repo().unwrap().targets[target_index].clone();
        Scene::EditTarget {
            editor: TargetEditor::with_target(target),
            target_index,
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
    log: Logger,
    s_scrollable: scrollable::State,
    /// Will always be set in the initial scene, and thus can be unwrapped in all other scenes
    passphrase: Option<String>,
    /// Current opened repo.
    /// Optional: Error might occur when opening, and it won't be opened until inside Overview
    repo: Option<Repo>,

    argon2: Argon2<'static>,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Only used to check if application should exit
    Tick(Instant),
    ToOverview,
    NewTarget,
    EditTarget(usize),
    ListItem(usize, ListItemMessage),
    TargetEditor(TargetEditorMessage),
    OpenSettings,
    PickRepo(Opt<RepoOption>),

    // Scene::Initial
    SetPassphrase1(String),
    SetPassphrase2(String),
    InitialConfirm,

    // Repo editor (maybe make a new component)
    SetRepoName(String),
    SetRepoHome(PathBuf),
    SaveRepo,
    RepoHome(path::Message),
    RepoSaveResult(Result<Redacted<Repo>, String>),
}

pub fn init_repo(path: &Path, passphrase: String, log: Logger) -> anyhow::Result<Repo> {
    let url = Url::from_directory_path(path)
        .ok()
        .context("RDEDUP_DIR url from path")?;
    if path.read_dir()?.next().is_none() {
        let passphrase = passphrase;
        info!(log, "Initialize repo {:?}", url);
        Repo::init(
            &url,
            &move || Ok(passphrase.clone()),
            RepoSettings::default(),
            log.clone(),
        )
        .context("Initialing Rdedup Repo")
    } else {
        // Is it an already existing repo?
        info!(log, "Open existing repo {:?}", url);
        Repo::open(&url, log.clone()).context("Opening existing Rdedup Repo")
    }
}

impl Application for Ui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = Config::load()
            .context("Could not deserialize config file")
            .unwrap();

        let log = log::logger();
        (
            Ui {
                scene: Scene::init(),
                config,
                s_scrollable: Default::default(),
                log,
                repo: None,
                passphrase: None,
                argon2: Argon2::default(),
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

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(_) => Command::none(),
            Message::ToOverview => {
                self.scene = Scene::overview(&self.config);
                Command::none()
            }
            Message::NewTarget => {
                if let Some(Opt {
                    value: RepoOption::Select(repo_id),
                    ..
                }) = self.config.selected_repo
                {
                    self.scene = Scene::create_target(repo_id);
                }
                Command::none()
            }
            Message::EditTarget(index) => {
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
                        // Easier to do the pattern matching on scene first, due to the need of
                        // capturing `target_index` optionally. (ran into borrowing issues)
                        let (editor, target_index) = match &mut self.scene {
                            Scene::CreateTarget { ref mut editor } => (Some(editor), None),
                            Scene::EditTarget {
                                ref mut editor,
                                target_index,
                            } => (Some(editor), Some(target_index)),
                            _ => panic!(),
                        };
                        if let Some(editor) = editor {
                            match verify_target(&editor.target) {
                                Ok(()) => {
                                    let repo = self.config.selected_repo_mut().unwrap();
                                    if let Some(target_index) = target_index {
                                        repo.targets[*target_index] = editor.target.clone();
                                    } else {
                                        repo.targets.push(editor.target.clone());
                                    }
                                    self.scene = Scene::overview(&self.config);
                                }
                                Err(e) => editor.error = Some(e),
                            }
                        }
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
                    RepoOption::Select(id) => {
                        // Find repo in config

                        let result: anyhow::Result<()> = try {
                            let repo_config =
                                self.config.find_repo(id).context("Cannot find repo")?;

                            let url = &Url::from_directory_path(&repo_config.home)
                                .map_err(|()| anyhow::Error::msg("Url->Path"))?;
                            info!(self.log, "Opening repo at {}", url);

                            let repo = Repo::open(url, self.log.clone())?;
                            self.repo = Some(repo);
                        };

                        match result {
                            Ok(()) => self.config.selected_repo = Some(repo),
                            Err(e) => error!(self.log, "[User error] {:#?}", e),
                        }
                    }
                }
                Command::none()
            }

            Message::SetPassphrase1(pass) => match &mut self.scene {
                Scene::Initial {
                    ref mut passphrase1,
                    ..
                } => {
                    *passphrase1 = pass;
                    Command::none()
                }
                _ => Command::none(),
            },
            Message::SetPassphrase2(pass) => match &mut self.scene {
                Scene::Initial {
                    ref mut passphrase2,
                    ..
                } => {
                    *passphrase2 = pass;
                    Command::none()
                }
                _ => Command::none(),
            },
            Message::InitialConfirm => match &mut self.scene {
                Scene::Initial {
                    ref passphrase1,
                    ref passphrase2,
                    ref mut error,
                    ..
                } => {
                    if let Some(ref passphrase_hash) = self.config.passphrase_hash {
                        let hash = PasswordHash::new(&passphrase_hash).unwrap();
                        if self
                            .argon2
                            .verify_password(&passphrase1.as_bytes(), &hash)
                            .is_ok()
                        {
                            self.passphrase = Some(passphrase1.clone());
                            self.scene = Scene::overview(&self.config);
                        } else {
                            *error = Some("Wrong passphrase".to_string());
                        }
                    } else {
                        if passphrase1 == passphrase2 {
                            self.config.passphrase_hash =
                                Some(hash_passphrase(&self.argon2, &passphrase1));
                            self.passphrase = Some(passphrase1.clone());
                            self.scene = Scene::overview(&self.config);
                        } else {
                            *error = Some("Passphrases don't match".to_string());
                        }
                    }
                    Command::none()
                }
                _ => Command::none(),
            },
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
                    if !name.is_empty() {
                        if let Some(home) = home {
                            match init_repo(
                                home,
                                self.passphrase.clone().unwrap(),
                                self.log.clone(),
                            ) {
                                Ok(repo) => {
                                    self.repo = Some(repo);
                                    let id = Uuid::new_v4();
                                    self.config.repos.insert(
                                        id,
                                        RepoConfig {
                                            id,
                                            name: name.clone(),
                                            home: home.clone(),
                                            targets: Default::default(),
                                        },
                                    );
                                    self.config.selected_repo = Some(Opt {
                                        name: name.clone(),
                                        value: RepoOption::Select(id),
                                    });
                                    self.scene = Scene::overview(&self.config);
                                    Command::none()
                                }
                                Err(e) => {
                                    *error = Some(e.to_string());
                                    Command::none()
                                }
                            }
                        } else {
                            *error = Some("Home path must be set".to_string());
                            Command::none()
                        }
                    } else {
                        *error = Some("Name must be non-empty".to_string());
                        Command::none()
                    }
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
            Message::RepoSaveResult(result) => match &mut self.scene {
                Scene::CreateRepo { ref mut error, .. } => {
                    match result {
                        Ok(repo) => (), // TODO??
                        Err(e) => *error = Some(e),
                    }
                    Command::none()
                }
                _ => Command::none(),
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        let config = &self.config;
        let w: Container<Message> = match &mut self.scene {
            Scene::Initial {
                passphrase1,
                passphrase2,
                s_pass1,
                s_pass2,
                s_confirm,
                error,
            } => Container::new({
                let mut column = Column::new().padding(20).spacing(20).push(
                    TextInput::new(s_pass1, "Passphrase", passphrase1, Message::SetPassphrase1)
                        .password()
                        .style(style::TextInput)
                        .size(H3_SIZE),
                );
                if self.config.passphrase_hash.is_none() {
                    column = column.push(
                        TextInput::new(
                            s_pass2,
                            "Confirm passphrase",
                            passphrase2,
                            Message::SetPassphrase2,
                        )
                        .password()
                        .style(style::TextInput)
                        .size(H3_SIZE),
                    );
                }
                let button = Button::new(s_confirm, Text::new("CONFIRM").size(TEXT_SIZE))
                    .on_press(Message::InitialConfirm);

                column = column.push(button);
                if let Some(error) = error {
                    column = column
                        .push(Text::new(error.as_str()).color(Color::from_rgb(0.5, 0.0, 0.0)));
                }
                column
            }),
            Scene::Overview {
                list,
                new_button,
                selected_target,
                s_open_settings,
                s_repo_pick_list,
            } => {
                let repo_options = repo_options(self.config.repos.values());

                let mut button = Button::new(new_button, Text::new("NEW BUP").size(TEXT_SIZE - 4))
                    .style(style::Button::Primary);
                if self.config.selected_repo.is_some() {
                    button = button.on_press(Message::NewTarget);
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
                    );
                if let Some(ref selected_repo) = self.config.selected_repo {
                    // A bit verbose, getting the path of selected repo
                    //
                    let repo = selected_repo.value.id().and_then(|id| config.find_repo(id));
                    if let Some(repo) = repo {
                        header = header.push(Text::new(repo.home.display().to_string()))
                    }
                }

                header = header.push(button);

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
                    .align_x(Horizontal::Right),
                );

                let mut overview: Column<Message> = Column::new().spacing(20);
                if let Some(repo) = self.config.selected_repo() {
                    for (i, (target, state)) in zip_list(&repo.targets, list).enumerate() {
                        let is_selected = selected_target.map(|s| s == i).unwrap_or(false);
                        overview = overview.push(
                            state
                                .view(&target, is_selected)
                                .map(move |msg| Message::ListItem(i, msg)),
                        );
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
                    .align_x(Horizontal::Center)
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
                            Container::new({
                                let mut row = Row::new()
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
                                    );
                                if let Some(error) = error {
                                    row = row.push(
                                        Text::new(format!("Error: {}", error.as_str()))
                                            .color(Color::from_rgb(0.5, 0.0, 0.0)),
                                    );
                                }
                                row
                            })
                            .width(Length::Fill), // .align_x(Horizontal::End),
                        ),
                )
                .style(style::DialogContainer)
                .width(Length::Fill)
                .max_width(1000)
                .height(Length::Shrink),
            )
            .padding(50)
            .align_x(Horizontal::Center)
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
    pub fn view(&mut self, target: &Target, selected: bool) -> Element<ListItemMessage> {
        let header = Row::new()
            .height(Length::Units(36))
            .width(Length::Fill)
            .push(
                Container::new(Text::new(&target.name).size(TEXT_SIZE))
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Left)
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
                .align_x(Horizontal::Right)
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

fn verify_target(target: &Target) -> Result<(), String> {
    if target.name.is_empty() {
        return Err("Name should not be empty".to_string());
    }
    if target.sources.is_empty() {
        return Err("Should have at least one source".to_string());
    }
    for source in &target.sources {
        if source.is_none() {
            return Err("All sources should have a path".to_string());
        }
    }
    for exclude in &target.excludes {
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
    /// bool: true if config was newly created
    pub fn load() -> anyhow::Result<Self> {
        match std::fs::read_to_string(config_path()) {
            Ok(contents) => Ok(serde_json::from_str(&contents)?),
            Err(_) => Ok(Config::default()),
        }
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
#[derive(Clone)]
pub struct Redacted<T>(pub T);
impl<T> std::fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "<redacted>")
    }
}

fn hash_passphrase(argon2: &Argon2<'static>, passphrase: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    argon2
        .hash_password(passphrase.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

fn zip_list<'a, T, I, S>(data: I, state: &'a mut Vec<S>) -> impl Iterator<Item = (T, &mut S)> + 'a
where
    I: IntoIterator<Item = T> + Clone,
    <I as IntoIterator>::IntoIter: 'a,
    S: Default + Clone,
{
    // Ensure that we have enough state elements
    state.resize(data.clone().into_iter().count(), S::default());

    data.into_iter().zip(state)
}
