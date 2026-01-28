use graphics::{
    commands::CommandBuffer,
    glam::Vec2,
    types::{Bounds, styling::StyleSheet},
    widget::{Anchor, DesiredSize, FrameContext, Widget},
};
use graphics_derive::Queryable;

use crate::{button::Button, draw};

pub struct BarStyle {
    pub background: String,
    pub border: String,
    pub corner: String,
}

impl BarStyle {
    fn new(widget: Option<impl Into<String>>, layer: &str) -> Self {
        let widget = widget.map_or("slider".to_string(), core::convert::Into::into);
        Self {
            background: format!("{widget}:{layer}:background"),
            border: format!("{widget}:{layer}:border"),
            corner: format!("{widget}:{layer}:corners"),
        }
    }
}

pub struct SliderStyles {
    background: BarStyle,
    foreground: BarStyle,
}

impl SliderStyles {
    #[must_use]
    pub fn new(widget: &str) -> Self {
        Self {
            background: BarStyle::new(Some(widget), "background"),
            foreground: BarStyle::new(Some(widget), "foreground"),
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
    pub style: SliderStyles,

    bar_bounds: Bounds,
    foreground_bounds: Bounds,
    handle_bounds: Bounds,

    #[content]
    button: Button,
}

impl Slider {
    fn new_internal(id: Option<String>, min: f32, max: f32) -> Self {
        let mut button = Button::new();
        button.override_name("slider:handle");
        button.size = Vec2::new(8.0, 16.0);
        Self {
            id,
            min,
            max,
            value: 0.0,

            style: SliderStyles::new("slider"),

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

    fn draw<'frame>(&'frame self, stylesheet: &StyleSheet, out: &mut CommandBuffer<'frame>) {
        //Background
        out.push(draw(
            self.bar_bounds,
            stylesheet.get_component(&self.style.background.background),
            stylesheet.get_stroke_component(&self.style.background.border),
            stylesheet.get_corners_component(&self.style.background.corner),
        ));

        //Foreground
        out.push(draw(
            self.foreground_bounds,
            stylesheet.get_component(&self.style.foreground.background),
            stylesheet.get_stroke_component(&self.style.foreground.border),
            stylesheet.get_corners_component(&self.style.foreground.corner),
        ));

        self.button.draw(stylesheet, out);
    }

    fn layout(&mut self, bar_bounds: Bounds) {
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
