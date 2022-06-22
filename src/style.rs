use iced::{
    pure::widget::{button, container, scrollable, text_input},
    Background, Color, Vector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Theme {
        // Falls back to light if it cannot detect the theme
        match dark_light::detect() {
            dark_light::Mode::Dark => Theme::Dark,
            dark_light::Mode::Light => Theme::Light,
        }
    }
}
pub struct LightButton;

impl button::StyleSheet for LightButton {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Color::from([0.282, 0.424, 0.788]))),
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            ..active
        }
    }

    fn pressed(&self) -> button::Style {
        let active = self.active();

        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            ..active
        }
    }
}

pub struct DarkButton;

impl button::StyleSheet for DarkButton {
    fn active(&self) -> button::Style {
        button::Style {
            background: Color::from([0.466, 0.569, 0.820]).into(),
            text_color: Color::from([0.2, 0.2, 0.2]).into(),
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Color::from([0.466, 0.569, 0.820]).into(),
            text_color: Color::from([0.2, 0.2, 0.2]).into(),
            border_width: 0.0,
            ..active
        }
    }

    fn pressed(&self) -> button::Style {
        let active = self.active();

        button::Style {
            shadow_offset: Vector::new(0.5, 1.0),
            ..active
        }
    }
}
impl<'a> From<Theme> for Box<dyn button::StyleSheet + 'a> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => LightButton.into(),
            Theme::Dark => DarkButton.into(),
        }
    }
}

pub struct LightTextInput;

impl text_input::StyleSheet for LightTextInput {
    fn hovered(&self) -> text_input::Style {
        self.focused()
    }

    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Color::TRANSPARENT.into(),
            ..Default::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            background: Color::TRANSPARENT.into(),
            ..Default::default()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from([0.4, 0.4, 0.4])
    }

    fn value_color(&self) -> Color {
        Color::BLACK
    }

    fn selection_color(&self) -> Color {
        Color::from([0.8, 0.8, 0.8])
    }
}
pub struct DarkTextInput;

impl text_input::StyleSheet for DarkTextInput {
    fn hovered(&self) -> text_input::Style {
        self.focused()
    }

    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Color::TRANSPARENT.into(),
            ..Default::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            background: Color::TRANSPARENT.into(),
            ..Default::default()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from([0.4, 0.4, 0.4])
    }

    fn value_color(&self) -> Color {
        Color::from([0.466, 0.569, 0.820])
    }

    fn selection_color(&self) -> Color {
        Color::WHITE
    }
}

impl<'a> From<Theme> for Box<dyn text_input::StyleSheet + 'a> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => LightTextInput.into(),
            Theme::Dark => DarkTextInput.into(),
        }
    }
}

pub struct InterestsTextInput;

impl text_input::StyleSheet for InterestsTextInput {
    fn hovered(&self) -> text_input::Style {
        self.focused()
    }

    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Color::TRANSPARENT.into(),
            ..Default::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            background: Color::TRANSPARENT.into(),
            ..Default::default()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from([0.4, 0.4, 0.4])
    }

    fn value_color(&self) -> Color {
        Color::from([0.466, 0.569, 0.820])
    }

    fn selection_color(&self) -> Color {
        Color::WHITE
    }
}

pub struct DarkContainer;

impl container::StyleSheet for DarkContainer {
    fn style(&self) -> container::Style {
        container::Style {
            background: Color::from([0.2, 0.2, 0.2]).into(),
            text_color: Color::from([0.466, 0.569, 0.820]).into(),
            ..Default::default()
        }
    }
}
impl<'a> From<Theme> for Box<dyn container::StyleSheet + 'a> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Default::default(),
            Theme::Dark => DarkContainer.into(),
        }
    }
}

pub struct YouContainer;

impl container::StyleSheet for YouContainer {
    fn style(&self) -> container::Style {
        container::Style {
            background: None,
            text_color: Color::from([0.003, 0.003, 1.0]).into(),
            ..Default::default()
        }
    }
}

pub struct StrangerContainer;

impl container::StyleSheet for StrangerContainer {
    fn style(&self) -> container::Style {
        container::Style {
            background: None,
            text_color: Color::from([1.0, 0.003, 0.003]).into(),
            ..Default::default()
        }
    }
}
