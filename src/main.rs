use iced::{button, Align, Button, Column, Element, Sandbox, Settings, Text};
use iced_native::{Widget};
use iced_graphics::{Renderer, Backend};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};


use app_dirs::*;

const APP_INFO: AppInfo = AppInfo{name: "Backup", author: "Erlend Langseth"};

pub fn main() -> iced::Result {

    println!("{:?}", get_app_root(AppDataType::UserConfig, &APP_INFO));
    Ui::run(Settings::default())
}

#[derive(Default)]
pub struct Ui {
    directories: Vec<Directory>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
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
        //
    }

    fn view(&mut self) -> Element<Message> {
        let mut el =Column::new()
            .padding(20)
            .align_items(Align::Center);
        for dir in &mut self.directories {
            el = el.push(dir.view());
        }
        el.into()
    }
}




#[derive(Default, Serialize, Deserialize)]
pub struct Directory {
    pub path: Option<PathBuf>,
    pub duplication: Vec<Duplication>,
}
impl Directory {
    pub fn view(&mut self) -> Element<Message> {
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize)]
pub enum Duplication {
    Disk (PathBuf),
}
