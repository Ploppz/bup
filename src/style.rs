//! Copied from the `todos` example
use iced::{button, text_input, Background, Color, Vector};

pub enum Button {
    Icon { hover_color: Color },
    Creation,
    Path,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        match self {
            Button::Icon { .. } => button::Style {
                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..button::Style::default()
            },
            Button::Creation => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.7, 0.2))),
                border_radius: 5.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            Button::Path => button::Style {
                background: None,
                ..button::Style::default()
            },
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();
        match self {
            Button::Icon { hover_color } => button::Style {
                text_color: *hover_color,
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                ..active
            },
            Button::Creation => button::Style {
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
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
            background: Background::Color(Color::WHITE),
            ..Default::default()
            // border_radius: 0.0,
            // border_width: 0.0,
            // border_color: Color::default(),
        }
    }
    fn focused(&self) -> text_input::Style {
        self.active()
        
    }
    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.7,0.7,0.7)
    }
    fn value_color(&self) -> Color {
        Color::from_rgb(0.0,0.0,0.0)
    }
    fn selection_color(&self) -> Color {
        Color::from_rgb(0.8, 0.85, 0.7)
    }
}
