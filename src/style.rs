//! Copied from the `todos` example
use iced::{button, container, text_input, Background, Color, Vector};

pub const PRIMARY_COLOR: Color = Color::from_rgb(0.2, 0.6, 0.2);

pub const GREY: Color = Color::from_rgb(0.3, 0.3, 0.3);

pub fn shadow(mut col: Color) -> Color {
    col.r *= 0.82;
    col.g *= 0.82;
    col.b *= 0.82;
    col
}

pub enum Button {
    Primary,
    Text,
    Icon { hover_color: Color },
    Path,
    Item,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        match self {
            Button::Primary => button::Style {
                background: Some(Background::Color(PRIMARY_COLOR)),
                border_radius: 5.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            Button::Text => button::Style {
                background: None,
                border_radius: 5.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            Button::Icon { hover_color } => button::Style {
                text_color: *hover_color,
                // text_color: Color::WHITE,
                background: None,
                border_radius: 20.0,
                ..button::Style::default()
            },
            Button::Path => button::Style {
                background: None,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            Button::Item => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.8, 0.8))),
                ..button::Style::default()
            },
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();
        match self {
            Button::Primary => button::Style {
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                background: Some(Background::Color(shadow(PRIMARY_COLOR))),
                ..active
            },
            Button::Text => button::Style {
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                background: Some(Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.1))),
                ..active
            },
            Button::Item => button::Style {
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                ..active
            },
            Button::Icon { hover_color } => button::Style {
                text_color: *hover_color,
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
                ..active
            },
            Button::Path => active,
        }
    }
}

pub struct TextInput;
impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Style {
        text_input::Style  {
            background: Background::Color(Color::TRANSPARENT),
            border_radius: 10.0,
            ..Default::default()
            // border_radius: 0.0,
            // border_width: 0.0,
            // border_color: Color::default(),
        }
    }
    fn focused(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(Color::from_rgb(0.2, 0.2, 0.2)),
            ..self.active()
        }
    }
    fn hovered(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(Color::from_rgb(0.1, 0.1, 0.1)),
            ..self.active()
        }
    }
    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.5, 0.5, 0.5)
    }
    fn value_color(&self) -> Color {
        Color::WHITE
    }
    fn selection_color(&self) -> Color {
        Color::from_rgb(0.1, 0.5, 0.1)
    }
}

pub struct EditorContainer;
impl container::StyleSheet for EditorContainer {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: Some(Color::from_rgb(1.0, 1.0, 1.0)),
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.12))),
            border_radius: 18.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}
pub struct AppContainer;

impl container::StyleSheet for AppContainer {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: Some(Color::WHITE),
            background: Some(Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

pub struct MenuContainer;

impl container::StyleSheet for MenuContainer {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: Some(Color::WHITE),
            background: Some(Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

pub struct ListItemHeader {
    pub selected: bool,
}

impl ListItemHeader {
    fn base_color(&self) -> Color {
        if self.selected {
            Color::from_rgb(0.2, 0.2, 0.2)
        } else {
            Color::from_rgb(0.14, 0.14, 0.14)
        }
    }
    fn highlight_color(&self) -> Color {
        let b = self.base_color();
        Color {
            a: 1.0,
            r: b.r * 0.9,
            g: b.g * 0.9,
            b: b.b * 0.9,
        }
    }
}
impl button::StyleSheet for ListItemHeader {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(self.base_color())),
            border_radius: 5.0,
            text_color: Color::WHITE,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();
        button::Style {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            background: Some(Background::Color(self.highlight_color())),
            ..active
        }
    }
}
