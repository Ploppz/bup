use crate::util::*;
use crate::*;
use std::fmt::*;

#[derive(Debug, Copy, Clone)]
pub enum Icon {
    Folder,
    Delete,
    Edit,
    New,
    Settings,
    Repo,
}
impl Icon {
    pub fn text(&self) -> Text {
        Text::new(&self.to_string())
            .font(ICONS)
            .width(Length::Units(TEXT_SIZE))
            .size(TEXT_SIZE)
    }
    pub fn h3(&self) -> Text {
        Text::new(&self.to_string())
            .size(H3_SIZE)
            .color([0.7, 0.7, 0.7])
            .font(ICONS)
            .width(Length::Units(20))
    }
}
impl Display for Icon {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                Icon::Folder => '\u{f74a}',
                Icon::Delete => '\u{f00d}',
                Icon::Edit => '\u{f044}',
                Icon::New => '\u{f44d}', // TODO
                Icon::Settings => '\u{f992}',
                Icon::Repo => '\u{f401}',
            }
        )
    }
}
