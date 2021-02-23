use std::fmt::*;
use crate::*;

#[derive(Debug, Copy, Clone)]
pub enum Icon {
    Folder,
    Delete,
    Edit,

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
            .size(20)
            .color([0.7,0.7,0.7])
            .font(ICONS)
            .width(Length::Units(20))
    }
}
impl Display for Icon {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}",
        match self {
            Icon::Folder => '\u{f74a}',
            Icon::Delete => '\u{fae7}',
            Icon::Edit => '\u{F303}',
        }
            )
    }
}
