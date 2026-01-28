use graphics::{
    commands::{CommandBuffer, DrawRectCommand, DrawTextureCommand},
    glam::Vec2,
    types::{Argb8888, Bounds, Corners, Paint, PainterContext, Stroke},
    widget::{Anchor, DesiredSize, FrameContext, Widget},
};
use graphics_derive::Queryable;

use crate::button::{Button, ButtonStyle};

pub struct BarStyle {
    pub paint: Paint,
    pub border: Stroke,
    pub corners: Corners,
}

impl BarStyle {
    fn background_default() -> Self {
        Self {
            paint: Argb8888::new(239, 239, 239, 255).into(),
            border: Stroke {
                color: [
                    Argb8888::new(194, 194, 194, 255),
                    Argb8888::WHITE,
                    Argb8888::new(194, 194, 194, 255),
                    Argb8888::WHITE,
                ],
                width: 1.0,
            },
            corners: Corners::NONE,
        }
    }

    fn foreground_default() -> Self {
        Self {
            paint: Argb8888::TRANSPARENT.into(),
            border: Stroke::NONE,
            corners: Corners::NONE,
        }
    }
}

pub struct HandleStyle {
    pub normal: ButtonStyle,
    pub hover: ButtonStyle,
    pub pressed: ButtonStyle,
}

impl HandleStyle {
    fn default() -> Self {
        Self {
            normal: ButtonStyle::normal(),
            hover: ButtonStyle::hover(),
            pressed: ButtonStyle::pressed(),
        }
    }
}

pub struct SliderStyle {
    pub background: BarStyle,
    pub foreground: BarStyle,
    pub handle: HandleStyle,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            background: BarStyle::background_default(),
            foreground: BarStyle::foreground_default(),
            handle: HandleStyle::default(),
        }
    }
}

#[derive(Queryable)]
pub struct Slider {
    id: Option<String>,
    min: f32,
    max: f32,
    value: f32,

    pub anchor: Anchor,
    pub style: SliderStyle,

    bar_bounds: Bounds,
    foreground_bounds: Bounds,
    handle_bounds: Bounds,

    #[content]
    button: Button,
}

impl Slider {
    fn new_internal(id: Option<String>, min: f32, max: f32) -> Self {
        let mut button = Button::new();
        button.size = Vec2::new(8.0, 16.0);
        button.normal.background = Argb8888 {
            r: 255,
            g: 0,
            b: 0,
            a: 128,
        }
        .into();
        Self {
            id,
            min,
            max,
            value: 0.0,

            style: SliderStyle::default(),

            bar_bounds: Bounds::from_size((100.0, 5.0)),
            foreground_bounds: Bounds::ZERO,
            handle_bounds: Bounds::ZERO,

            anchor: Anchor::Left,
            button,
        }
    }

    pub fn new_with_id(id: impl Into<String>, min: f32, max: f32) -> Self {
        Self::new_internal(Some(id.into()), min, max)
    }

    #[must_use]
    pub fn new(min: f32, max: f32) -> Self {
        Self::new_internal(None, min, max)
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    pub fn add_value(&mut self, value: f32) {
        self.set_value(self.value + value);
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new(0.0, 100.0)
    }
}

impl Widget for Slider {
    fn desired_size(&self) -> DesiredSize {
        DesiredSize::Exact(self.bar_bounds.size)
    }

    fn anchor(&self) -> Anchor {
        self.anchor
    }

    fn draw<'frame>(&'frame self, out: &mut CommandBuffer<'frame>) {
        // Draw background (сдвинутый)
        match &self.style.background.paint {
            Paint::Color(color) => out.push(
                DrawRectCommand::from_bounds(self.bar_bounds)
                    .with_color(color.clone())
                    .with_stroke(self.style.background.border)
                    .with_corners(self.style.background.corners),
            ),
            Paint::Texture(texture) => out.push(
                DrawTextureCommand::from_bounds(texture.clone(), self.bar_bounds)
                    .with_stroke(self.style.background.border)
                    .with_corners(self.style.background.corners),
            ),
            Paint::Custom(custom) => custom.draw(
                &PainterContext {
                    bounds: self.bar_bounds,
                    border: self.style.background.border,
                    corners: self.style.background.corners,
                },
                out,
            ),
        }

        //out.push(DrawRectCommand::new(
        //    self.bar_bounds,
        //    self.background.clone(),
        //    self.background_border.clone(),
        //    Corners::default(),
        //));

        // Draw foreground (сдвинутый)
        match &self.style.foreground.paint {
            Paint::Color(color) => out.push(
                DrawRectCommand::from_bounds(self.foreground_bounds)
                    .with_color(color.clone())
                    .with_stroke(self.style.foreground.border)
                    .with_corners(self.style.foreground.corners),
            ),
            Paint::Texture(texture) => out.push(
                DrawTextureCommand::from_bounds(texture.clone(), self.foreground_bounds)
                    .with_stroke(self.style.foreground.border)
                    .with_corners(self.style.foreground.corners),
            ),
            Paint::Custom(custom) => custom.draw(
                &PainterContext {
                    bounds: self.foreground_bounds,
                    border: self.style.foreground.border,
                    corners: self.style.foreground.corners,
                },
                out,
            ),
        }

        self.button.draw(out);
    }

    fn layout(&mut self, bar_bounds: Bounds) {
        //TODO FIX
        self.button.normal = self.style.handle.normal.clone();
        self.button.hover = self.style.handle.hover.clone();
        self.button.pressed = self.style.handle.pressed.clone();

        // 1. Определяем размер handle и смещение (offset)
        let handle_size = self.button.size;
        let vertical_offset = handle_size.y / 2.0;

        // 2. Сдвигаем базовые границы полосы
        let mut shifted_bar_bounds = bar_bounds;
        shifted_bar_bounds.position.y += vertical_offset;

        let progress = (self.value - self.min) / (self.max - self.min);
        let new_width = shifted_bar_bounds.size.x * progress;

        // 3. Вычисляем вертикальный центр сдвинутой полосы
        let bar_center_y = shifted_bar_bounds.position.y + (shifted_bar_bounds.size.y / 2.0);

        let foreground_bounds = Bounds {
            position: shifted_bar_bounds.position,
            size: Vec2::new(new_width, shifted_bar_bounds.size.y),
        };

        // 4. Рассчитываем позицию handle относительно СДВИНУТОЙ полосы
        let handle_pos_y = bar_center_y - (handle_size.y / 2.0);
        let handle_pos_x = shifted_bar_bounds.position.x + new_width - (handle_size.x / 2.0);

        let handle_bounds = Bounds {
            position: Vec2::new(handle_pos_x, handle_pos_y),
            size: handle_size,
        };

        self.bar_bounds = shifted_bar_bounds;
        self.foreground_bounds = foreground_bounds;
        self.handle_bounds = handle_bounds;

        self.button.layout(handle_bounds);
    }

    fn update(&mut self, ctx: &FrameContext) {
        self.button.update(ctx);
        if self.value.eq(&self.max) {
            self.value = self.min;
            //return;
        }
        //self.add_value(1.0);
    }
}
