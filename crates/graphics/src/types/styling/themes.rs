#![allow(clippy::non_std_lazy_statics)]

use derive_more::From;
use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::types::{Argb8888, Color, Corners, Stroke, Texture};

pub struct Themes {
    themes: HashMap<String, StyleSheet>,
}

#[derive(Debug, Clone, From)]
pub enum ThemeComponent {
    Color(Color),
    Stroke(Stroke),
    Corners(Corners),
    Texture(Texture),
}

impl From<Argb8888> for ThemeComponent {
    fn from(color: Argb8888) -> Self {
        ThemeComponent::Color(Color::Simple(color))
    }
}

#[derive(Default, Debug)]
pub struct StyleSheet {
    inner: HashMap<&'static str, ThemeComponent>,
}

impl StyleSheet {
    #[must_use]
    pub fn get_component(&self, key: &str) -> ThemeComponent {
        self.inner
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("Theme component not found: {key}"))
    }

    #[must_use]
    pub fn get_corners_component(&self, key: &str) -> Corners {
        match self.get_component(key) {
            ThemeComponent::Corners(corners) => corners,
            _ => panic!("Theme component is not a Corners"),
        }
    }

    #[must_use]
    pub fn get_stroke_component(&self, key: &str) -> Stroke {
        match self.get_component(key) {
            ThemeComponent::Stroke(stroke) => stroke,
            _ => panic!("Theme component is not a Stroke"),
        }
    }
}

macro_rules! map {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key, $value.into());
        )*
        map
    }};
}

lazy_static! {
    pub static ref AERO_THEME: StyleSheet = {
        StyleSheet {
            inner: map![
                "slider:background:background" => Argb8888::new(240, 240, 240, 255),
                "slider:background:border" => Stroke {
                    color: [
                        Argb8888::new(194, 194, 194, 255),
                        Argb8888::WHITE,
                        Argb8888::new(194, 194, 194, 255),
                        Argb8888::WHITE,
                    ],
                    width: 1.0,
                },
                "slider:background:corners" => Corners::NONE,

                "slider:foreground:background" => Argb8888::TRANSPARENT,
                "slider:foreground:border" => Stroke::NONE,
                "slider:foreground:corners" => Corners::NONE,

                "slider:handle:normal:background" => Argb8888::new(240, 240, 240, 255),
                "slider:handle:normal:border" => Stroke {
                    color: [
                        Argb8888::new(150, 150, 150, 255),
                        Argb8888::new(150, 150, 150, 255),
                        Argb8888::new(150, 150, 150, 255),
                        Argb8888::new(150, 150, 150, 255),
                    ],
                    width: 1.0,
                },
                "slider:handle:normal:corners" => Corners::NONE,

                "button:normal:background" => Argb8888::LIGHT_GRAY,
                "button:normal:border" => Stroke {
                    color: [Argb8888::DARK_GRAY; 4],
                    width: 1.0,
                },
                "button:normal:corners" => Corners::DEFAULT,

                "button:hover:background" => Argb8888::new(230, 230, 230, 255),
                "button:hover:border" => Stroke {
                    color: [Argb8888::BLUE; 4],
                    width: 1.0,
                },
                "button:hover:corners" => Corners::DEFAULT,

                "button:pressed:background" => Argb8888::GRAY,
                "button:pressed:border" => Stroke {
                    color: [Argb8888::DARK_GRAY; 4],
                    width: 1.0,
                },
                "button:pressed:corners" => Corners::DEFAULT,
            ],
        }
    };
}
