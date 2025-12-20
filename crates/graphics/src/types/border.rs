use crate::types::{Argb8888, Color};

pub trait AsColor: Default + Clone + PartialEq {
    fn as_color(&self) -> Color;
}

#[derive(Default, Clone, PartialEq)]
pub struct None;
impl AsColor for None {
    fn as_color(&self) -> Color {
        Color::Simple(Argb8888::TRANSPARENT)
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct Border<L, R, T, B> 
where
    L: AsColor,
    R: AsColor,
    T: AsColor,
    B: AsColor,
{
    left: L,
    right: R,
    top: T,
    bottom: B,
}

impl<L, R, T, B> Border<L, R, T, B>
where
    L: AsColor,
    R: AsColor,
    T: AsColor,
    B: AsColor,
{
    #[must_use]
    pub fn as_color_array(&self) -> [Color; 4] {
        [
            self.left.as_color(),
            self.right.as_color(),
            self.top.as_color(),
            self.bottom.as_color(),
        ]
    }
}

impl<R, T, B> Border<Argb8888, R, T, B>
where
    R: AsColor,
    T: AsColor,
    B: AsColor,
{
    pub const fn set_left(&mut self, color: Argb8888) {
        self.left = color;
    }

    #[must_use]
    pub const fn get_left(&self) -> Argb8888 {
        self.left
    }
}

impl<L, T, B> Border<L, Argb8888, T, B>
where
    L: AsColor,
    T: AsColor,
    B: AsColor,
{
    pub const fn set_right(&mut self, color: Argb8888) {
        self.right = color;
    }

    #[must_use]
    pub const fn get_right(&self) -> Argb8888 {
        self.right
    }
}

impl<L, R, B> Border<L, R, Argb8888, B>
where
    L: AsColor,
    R: AsColor,
    B: AsColor,
{
    pub const fn set_top(&mut self, color: Argb8888) {
        self.top = color;
    }

    #[must_use]
    pub const fn get_top(&self) -> Argb8888 {
        self.top
    }
}

impl<L, R, T> Border<L, R, T, Argb8888>
where
    L: AsColor,
    R: AsColor,
    T: AsColor,
{
    pub const fn set_bottom(&mut self, color: Argb8888) {
        self.bottom = color;
    }

    #[must_use]
    pub const fn get_bottom(&self) -> Argb8888 {
        self.bottom
    }
}

impl AsColor for Argb8888 {
    fn as_color(&self) -> Color {
        Color::Simple(*self)
    }
}
