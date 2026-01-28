use graphics::{
    commands::{CommandBuffer, DrawRectCommand},
    glam::Vec2,
    types::{Argb8888, Bounds, Color, Corners, Stroke},
    widget::{Anchor, DesiredSize, FrameContext, Widget},
};
use graphics_derive::Queryable;

use crate::button::Button;

#[derive(Queryable)]
pub struct Slider {
    id: Option<String>,
    min: f32,
    max: f32,
    value: f32,

    pub background: Color,
    pub foreground: Color,
    pub anchor: Anchor,

    bar_bounds: Bounds,
    foreground_bounds: Bounds,
    #[content]
    button: Button,
}

impl Slider {
    fn new_internal(id: Option<String>, min: f32, max: f32) -> Self {
        let mut button = Button::new();
        button.size = Vec2::new(16.0, 16.0);
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

            background: Argb8888::GRAY.into(),
            foreground: Argb8888::BLUE.into(),
            bar_bounds: Bounds::from_size((100.0, 5.0)),
            anchor: Anchor::Left,
            button,
            foreground_bounds: Bounds::ZERO,
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

    pub fn set_foreground(&mut self, color: impl Into<Color>) {
        self.foreground = color.into();
    }

    pub fn set_background(&mut self, color: impl Into<Color>) {
        self.background = color.into();
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
        out.push(
            DrawRectCommand::from_bounds(self.bar_bounds)
                .with_color(self.background.clone())
                .with_corners(Corners::DEFAULT),
        );

        out.push(DrawRectCommand::new(
            self.bar_bounds,
            self.background.clone(),
            Stroke::NONE,
            Corners::NONE,
        ));

        // Draw foreground (сдвинутый)
        out.push(DrawRectCommand::new(
            self.foreground_bounds,
            self.foreground.clone(),
            Stroke::NONE,
            Corners::NONE,
        ));

        self.button.draw(out);
    }

    fn layout(&mut self, bar_bounds: Bounds) {
        // 1. Определяем размер handle и смещение (offset)
        let handle_size = self.button.size;
        let vertical_offset = handle_size.y / 2.0;

        // 2. Сдвигаем базовые границы полосы
        let mut shifted_bar_bounds = bar_bounds.clone();
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

        self.button.layout(handle_bounds);
    }

    fn update(&mut self, ctx: &FrameContext) {
        self.button.update(ctx);
        if self.value.eq(&self.max) {
            self.value = self.min;
            return;
        }
        self.add_value(1.0);
    }
}
