use crate::*;

// Fonts
pub const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/agave.ttf"),
};

pub  fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .width(Length::Units(TEXT_SIZE))
        .size(TEXT_SIZE)
}
pub fn icon_h3(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .size(22)
        // .color([0.7,0.7,0.7])
        .font(ICONS)
        .width(Length::Units(20))
}
pub fn text<T: Into<String>>(text: T) -> Text {
    Text::new(text).font(ICONS).size(TEXT_SIZE)
}

pub fn h3<T: Into<String>>(text: T) -> Text {
    Text::new(text)
        .size(22)
        .color([0.7, 0.7, 0.7])
        .horizontal_alignment(HorizontalAlignment::Center)
}
