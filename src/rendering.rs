use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::engine::{EngineFrame, PixelFormat};

pub fn normalize_pixels(frame: &EngineFrame, bypass_swizzle: bool) -> Vec<u8> {
    let width = frame.width as usize;
    let height = frame.height as usize;
    if width == 0 || height == 0 {
        return Vec::new();
    }
    let row_bytes = width.saturating_mul(4);
    let stride = frame.stride_bytes.max(row_bytes);
    let needs_swizzle = !bypass_swizzle && frame.pixel_format == PixelFormat::Bgra8;

    if stride == row_bytes {
        let mut out = frame.pixels.clone();
        if needs_swizzle {
            swizzle_bgra_to_rgba_in_place(&mut out);
        }
        return out;
    }

    let mut out = vec![0; row_bytes.saturating_mul(height)];
    for row in 0..height {
        let src_start = row.saturating_mul(stride);
        let dst_start = row.saturating_mul(row_bytes);
        if src_start + row_bytes > frame.pixels.len() || dst_start + row_bytes > out.len() {
            break;
        }
        out[dst_start..dst_start + row_bytes]
            .copy_from_slice(&frame.pixels[src_start..src_start + row_bytes]);
        if needs_swizzle {
            swizzle_bgra_to_rgba_in_place(&mut out[dst_start..dst_start + row_bytes]);
        }
    }
    out
}

pub fn frame_checksum(pixels: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    pixels.hash(&mut hasher);
    hasher.finish()
}

fn swizzle_bgra_to_rgba_in_place(pixels: &mut [u8]) {
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{AlphaMode, ColorSpace, EngineFrame};

    fn frame_with_pixels(
        width: u32,
        height: u32,
        stride_bytes: usize,
        pixel_format: PixelFormat,
        pixels: Vec<u8>,
    ) -> EngineFrame {
        EngineFrame {
            width,
            height,
            frame_number: 1,
            stride_bytes,
            pixel_format,
            alpha_mode: AlphaMode::Straight,
            color_space: ColorSpace::Srgb,
            pixels,
        }
    }

    #[test]
    fn swizzles_bgra_to_rgba() {
        let pixels = vec![10, 20, 30, 255, 40, 50, 60, 255];
        let frame = frame_with_pixels(2, 1, 8, PixelFormat::Bgra8, pixels);
        let normalized = normalize_pixels(&frame, false);
        assert_eq!(normalized, vec![30, 20, 10, 255, 60, 50, 40, 255]);
    }

    #[test]
    fn repacks_stride_and_preserves_gradient() {
        let width = 2;
        let height = 2;
        let row_bytes = width * 4;
        let stride = row_bytes + 4;
        let mut pixels = vec![0; stride * height];
        for y in 0..height {
            for x in 0..width {
                let base = y * stride + x * 4;
                pixels[base] = (x * 20 + y * 40) as u8;
                pixels[base + 1] = 10;
                pixels[base + 2] = 200;
                pixels[base + 3] = 255;
            }
        }
        let frame = frame_with_pixels(
            width as u32,
            height as u32,
            stride,
            PixelFormat::Rgba8,
            pixels,
        );
        let normalized = normalize_pixels(&frame, false);
        let sample = &normalized[row_bytes..row_bytes + 4];
        assert_eq!(sample, &[40, 10, 200, 255]);
    }

    #[test]
    fn checksum_changes_on_pixel_update() {
        let first = vec![1, 2, 3, 4];
        let second = vec![1, 2, 3, 5];
        assert_ne!(frame_checksum(&first), frame_checksum(&second));
    }
}
