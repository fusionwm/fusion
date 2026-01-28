#![allow(clippy::non_std_lazy_statics)]

use derive_more::From;
use lazy_static::lazy_static;
use std::collections::HashMap;

use lasso::Rodeo;

use crate::types::{Argb8888, Color, Corners, Stroke};

pub struct Themes {
    themes: HashMap<String, StyleSheet>,
}

#[derive(Debug, From)]
pub enum ThemeComponent {
    Color(Color),
    Stroke(Stroke),
    Corners(Corners),
}

impl From<Argb8888> for ThemeComponent {
    fn from(color: Argb8888) -> Self {
        ThemeComponent::Color(Color::Simple(color))
    }
}

#[derive(Debug)]
pub struct StyleSheet {
    inner: HashMap<&'static str, ThemeComponent>,
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
    static ref INTERNER: Rodeo = Rodeo::new();
    static ref AERO_THEME: StyleSheet = {
        StyleSheet {
            inner: map![
                "slider:background:color" => Argb8888::new(240, 240, 240, 255),
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

                "slider:foreground:color" => Argb8888::TRANSPARENT,
                "slider:foreground:border" => Stroke::NONE,
                "slider:foreground:corners" => Corners::NONE,
            ],
        }
    };
}
