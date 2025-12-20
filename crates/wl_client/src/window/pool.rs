use crate::{window::WindowId, WlClient};
use memmap2::MmapMut;
use rayon::{iter::ParallelIterator, slice::ParallelSliceMut};
use std::fs::File;
use std::os::fd::AsFd;
use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer,
        wl_shm::{Format, WlShm},
        wl_shm_pool::WlShmPool,
    },
    QueueHandle,
};

pub const DEFAULT_FILE_SIZE: u64 = (1920 * 4) * 1080;

#[derive(Debug)]
struct Mmap {
    file_size: u64,
    file: File,
    inner: MmapMut,
}

impl Mmap {
    pub fn new(size: u64) -> Self {
        let tmp = tempfile::tempfile().unwrap();
        tmp.set_len(size).unwrap();
        Self {
            file_size: size,
            inner: unsafe { memmap2::MmapOptions::new().map_mut(&tmp).unwrap() },
            file: tmp,
        }
    }
}

impl Default for Mmap {
    fn default() -> Self {
        Self::new(DEFAULT_FILE_SIZE)
    }
}

#[derive(Debug)]
pub struct ShmPool {
    mmap: Mmap,
    inner: WlShmPool,
    pool_size: i32,
}

impl ShmPool {
    pub fn new(size: u64, id: &WindowId, shm: &WlShm, qh: &QueueHandle<WlClient>) -> Self {
        let mut mmap = Mmap::new(size.max(DEFAULT_FILE_SIZE));
        let b = 0x2E;
        let g = 0x1E;
        let r = 0x1E;
        let a = 0xFF;
        for chunk in mmap.inner.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[b, g, r, a]);
        }
        let pool = shm.create_pool(mmap.file.as_fd(), size as i32, qh, id.clone());
        Self {
            mmap,
            pool_size: size as i32,
            inner: pool,
        }
    }

    pub fn destroy(self) {
        self.inner.destroy();
    }

    pub fn need_resize(&self, new_size: u64) -> bool {
        self.mmap.file_size < new_size || self.pool_size < new_size as i32
    }

    pub fn resize(&mut self, new_size: u64) {
        let mut file_modified = false;
        if self.mmap.file_size < new_size {
            let new_size = new_size + new_size / 2;
            self.mmap.file.set_len(new_size).unwrap();
            self.mmap.file_size = new_size;
            file_modified = true;
        }

        if self.pool_size < new_size as i32 || file_modified {
            self.inner.resize(new_size as i32);
            self.pool_size = new_size as i32;
        }
    }

    pub fn create_buffer(
        &self,
        offset: i32,
        width: i32,
        height: i32,
        qh: &QueueHandle<WlClient>,
        id: &WindowId,
    ) -> WlBuffer {
        self.inner.create_buffer(
            offset,
            width,
            height,
            width * 4,
            Format::Argb8888,
            qh,
            id.clone(),
        )
    }

    pub fn draw_text_at(&mut self, x: usize, y: usize, width: usize, height: usize, coverage: f32) {
        let buffer = &mut self.mmap.inner;

        if x >= width || y >= height {
            return;
        }

        let offset = (y * width + x) * 4;

        let src_r = 255;
        let src_g = 255;
        let src_b = 255;
        let src_a = (coverage * 255.0) as u8;

        let dst_r = buffer[offset];
        let dst_g = buffer[offset + 1];
        let dst_b = buffer[offset + 2];
        let dst_a = buffer[offset + 3];

        let alpha = src_a as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;

        buffer[offset] = (src_r as f32 * alpha + dst_r as f32 * inv_alpha) as u8;
        buffer[offset + 1] = (src_g as f32 * alpha + dst_g as f32 * inv_alpha) as u8;
        buffer[offset + 2] = (src_b as f32 * alpha + dst_b as f32 * inv_alpha) as u8;
        buffer[offset + 3] = (src_a as f32 + dst_a as f32 * inv_alpha) as u8;
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, width: usize, pixel: (u8, u8, u8, u8)) {
        let buffer = &mut self.mmap.inner;
        let offset = (y * width + x) * 4;

        buffer[offset] = pixel.0;
        buffer[offset + 1] = pixel.1;
        buffer[offset + 2] = pixel.2;
        buffer[offset + 3] = pixel.3;
    }

    pub fn clear(&mut self) {
        let buffer = &mut self.mmap.inner;
        let b = 0;
        let g = 0;
        let r = 0;
        let a = 0;
        buffer.par_chunks_exact_mut(4).for_each(|chunk| {
            chunk.copy_from_slice(&[b, g, r, a]);
        });
    }
}
