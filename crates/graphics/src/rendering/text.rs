use crate::rendering::{
    material::{Material, MaterialDescriptor},
    Gpu,
};
use fontdue::{Font, Metrics};
use glam::Vec4;
use guillotiere::AtlasAllocator;
use std::collections::HashMap;
use wgpu::{FilterMode, TextureFormat};

#[derive(Default)]
pub struct FontAtlasSet {
    inner: HashMap<u32, FontAtlas>,
}

impl FontAtlasSet {
    pub fn get_atlas(&mut self, size: u32) -> &mut FontAtlas {
        self.inner.entry(size).or_insert(FontAtlas::new())
    }
}

#[derive(Clone)]
pub struct GlyphData {
    pub uv: Vec4,
    pub metrics: Metrics,
}

pub struct FontAtlas {
    inner: HashMap<char, GlyphData>,
    allocator: AtlasAllocator,
    texture: Vec<u8>,
    size: u32,
    material: Option<Material>,
    recreate_material: bool,
}

impl FontAtlas {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            allocator: AtlasAllocator::new((512, 512).into()),
            texture: vec![0; 512 * 512 * 4],
            size: 512,
            material: None,
            recreate_material: true,
        }
    }

    pub fn get_or_add_glyph(&mut self, char: char, size: u32, font: &Font) -> GlyphData {
        if let Some(glyph) = self.inner.get(&char) {
            return glyph.clone();
        }

        let (metrics, bitmap) = font.rasterize(char, size as f32);
        if metrics.width == 0 || metrics.height == 0 {
            let glyph = GlyphData {
                uv: Vec4::ZERO,
                metrics,
            };

            self.inner.insert(char, glyph.clone());

            return glyph;
        }

        let rectangle = self
            .allocator
            .allocate((metrics.width as i32, metrics.height as i32).into())
            .unwrap()
            .rectangle;

        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let alpha = bitmap[y * metrics.width + x];
                let dst_index = (((rectangle.min.x + x as i32) as u32)
                    + ((rectangle.min.y + y as i32) as u32) * self.size)
                    as usize
                    * 4;

                self.texture[dst_index] = 255; // R
                self.texture[dst_index + 1] = 255; // G
                self.texture[dst_index + 2] = 255; // B
                self.texture[dst_index + 3] = alpha; // A
            }
        }

        let u0 = rectangle.min.x as f32 / self.size as f32;
        let v0 = rectangle.min.y as f32 / self.size as f32;
        let u1 = rectangle.max.x as f32 / self.size as f32;
        let v1 = rectangle.max.y as f32 / self.size as f32;

        let data = GlyphData {
            uv: Vec4::new(u0, v0, u1, v1),
            metrics,
        };

        self.recreate_material = true;
        self.inner.insert(char, data.clone());

        data
    }

    pub fn get_or_add_material(&mut self, gpu: &Gpu) -> &Material {
        if self.material.is_none() || self.recreate_material {
            self.create_material(gpu);
            self.recreate_material = false;
        }
        self.material.as_ref().unwrap()
    }

    fn create_material(&mut self, gpu: &Gpu) {
        self.material = Some(Material::from_pixels(
            &MaterialDescriptor {
                label: "Glyph atlas",
                pixels: &self.texture,
                size: (self.size, self.size),
                format: TextureFormat::Rgba8Unorm,
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
            },
            &gpu.device,
            &gpu.queue,
        ));
    }
}
