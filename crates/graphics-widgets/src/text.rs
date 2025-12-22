use graphics::{
    FontHandle,
    commands::{CommandBuffer, DrawCommand, DrawTextCommand},
    fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    glam::Vec2,
    types::{Argb8888, Bounds, Spacing},
    widget::{Anchor, DesiredSize, FrameContext, Widget},
};
use graphics_derive::Queryable;

#[derive(Queryable)]
pub struct Text {
    pub id: Option<String>,

    pub size: u32,
    pub color: Argb8888,
    pub anchor: Anchor,
    pub margin: Spacing,
    font: FontHandle,

    value: String,
    layout: Layout,
    bounds: Bounds,
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Text {
    fn new_internal(id: Option<String>) -> Self {
        let mut instance = Self {
            id,
            value: String::new(),
            font: FontHandle::default(),
            size: 12,
            color: Argb8888::WHITE,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            anchor: Anchor::Left,
            bounds: Bounds::ZERO,
            margin: Spacing::ZERO,
        };

        instance.refresh_layout();
        instance
    }

    pub fn new_with_id(id: impl Into<String>) -> Self {
        Self::new_internal(Some(id.into()))
    }

    #[must_use]
    pub fn new() -> Self {
        Self::new_internal(None)
    }

    pub fn set_font(&mut self, font: FontHandle) {
        self.font = font;
        self.refresh_layout();
    }

    pub fn set_text(&mut self, value: &str) {
        self.value.clear();
        self.value.insert_str(0, value);
        self.refresh_layout();
    }

    fn refresh_layout(&mut self) {
        self.layout.clear();
        self.layout.append(
            &[self.font.as_ref()],
            &TextStyle {
                text: &self.value,
                px: self.size as f32,
                font_index: 0,
                user_data: (),
            },
        );
    }
}

impl Widget for Text {
    fn desired_size(&self) -> DesiredSize {
        let font = self.font.as_ref();
        let mut x = 0.0;

        let glyphs = self.layout.glyphs();
        let text_width = match self.layout.lines() {
            Some(lines) => lines
                .iter()
                .map(|ln| {
                    let glyph = &glyphs[ln.glyph_end];
                    glyph.x + glyph.width as f32
                })
                .fold(f32::NAN, |m, v| v.max(m)),
            None => 0.0,
        };

        self.layout.glyphs().iter().for_each(|c| {
            let metrics = font.metrics(c.parent, self.size as f32);
            x += metrics.advance_width;
            x += metrics.bounds.xmin;
        });

        let y = self.layout.height();
        DesiredSize::Exact(Vec2::new(
            text_width + self.margin.right,
            y + self.margin.bottom,
        ))
    }

    fn anchor(&self) -> Anchor {
        self.anchor
    }

    fn draw<'frame>(&'frame self, out: &mut CommandBuffer<'frame>) {
        out.push(DrawCommand::Text(DrawTextCommand::new(
            self.size,
            self.color,
            self.bounds.position + Vec2::new(self.margin.left, self.margin.top),
            &self.font,
            &self.layout,
        )));
    }

    fn layout(&mut self, bounds: Bounds) {
        self.bounds = bounds;
        self.layout.reset(&LayoutSettings {
            max_width: Some(self.bounds.size.x + self.bounds.position.x),
            max_height: Some(self.bounds.size.y),
            ..LayoutSettings::default()
        });

        self.refresh_layout();
    }

    fn update(&mut self, _: &FrameContext) {}
}
