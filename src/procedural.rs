use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

const TILE_SIZE: u32 = 128;

pub struct SurfaceTextures {
    pub asphalt: Handle<Image>,
    pub pavement: Handle<Image>,
    pub facade: Handle<Image>,
    pub ground: Handle<Image>,
}

pub fn add_surface_textures(images: &mut Assets<Image>) -> SurfaceTextures {
    SurfaceTextures {
        asphalt: images.add(make_texture(asphalt_pixels())),
        pavement: images.add(make_texture(pavement_pixels())),
        facade: images.add(make_texture(facade_pixels())),
        ground: images.add(make_texture(ground_pixels())),
    }
}

fn make_texture(data: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: TILE_SIZE,
            height: TILE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

fn noise(x: u32, y: u32, salt: u32) -> u8 {
    let mut value =
        x.wrapping_mul(1_103_515_245) ^ y.wrapping_mul(12_345) ^ salt.wrapping_mul(2_654_435_761);
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    ((value ^ (value >> 16)) & 31) as u8
}

fn asphalt_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let grain = noise(x, y, 7) as i16 - 15;
        let broad = noise(x / 8, y / 8, 101) as i16 - 15;
        let aggregate = if noise(x / 2, y / 2, 19) > 27 { 18 } else { 0 };
        let repair = if noise(x / 16, y / 16, 127) > 29 {
            12
        } else {
            0
        };
        let shade = grain * 2 + broad * 2 + aggregate + repair;
        [
            (35_i16 + shade).clamp(0, 255) as u8,
            (43_i16 + shade).clamp(0, 255) as u8,
            (53_i16 + shade).clamp(0, 255) as u8,
            255,
        ]
    })
}

fn pavement_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let joint = x % 32 < 2 || y % 24 < 2;
        let grain = noise(x, y, 31) as i16 - 15;
        let slab = ((x / 32 + (y / 24) * 3) % 2) as i16 * 9;
        let grit = if noise(x / 2, y / 2, 137) > 28 { 13 } else { 0 };
        if joint {
            [39, 50, 58, 255]
        } else {
            [
                (96_i16 + grain + slab + grit).clamp(0, 255) as u8,
                (111_i16 + grain + slab + grit).clamp(0, 255) as u8,
                (120_i16 + grain + slab + grit).clamp(0, 255) as u8,
                255,
            ]
        }
    })
}

fn facade_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let local_x = x % 32;
        let local_y = y % 24;
        let window = (5..27).contains(&local_x) && (5..20).contains(&local_y);
        if window {
            let variation = noise(x / 32, y / 24, 53);
            if variation > 18 {
                let glow = variation as i16 / 3;
                [
                    (154_i16 + glow).clamp(0, 255) as u8,
                    (112_i16 + glow / 2).clamp(0, 255) as u8,
                    76,
                    255,
                ]
            } else {
                [20, 37, 53, 255]
            }
        } else if local_x == 4 || local_x == 27 || local_y == 4 || local_y == 20 {
            [43, 62, 74, 255]
        } else {
            let grain = noise(x, y, 71) as i16 - 15;
            let weathering = if noise(x / 4, y / 4, 149) > 26 {
                -13
            } else {
                0
            };
            let groove = if local_x == 0 || local_x == 31 { -9 } else { 0 };
            let detail = grain * 2 + weathering + groove;
            [
                (75_i16 + detail).clamp(0, 255) as u8,
                (90_i16 + detail).clamp(0, 255) as u8,
                (105_i16 + detail).clamp(0, 255) as u8,
                255,
            ]
        }
    })
}

fn ground_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let variation = noise(x, y, 89) as i16 - 15;
        let patch = noise(x / 8, y / 8, 163) as i16 - 15;
        let detail = variation + patch / 2;
        [
            (28_i16 + detail / 2).clamp(0, 255) as u8,
            (70_i16 + detail).clamp(0, 255) as u8,
            (57_i16 + detail / 2).clamp(0, 255) as u8,
            255,
        ]
    })
}

fn pixels(mut color_at: impl FnMut(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut data = Vec::with_capacity((TILE_SIZE * TILE_SIZE * 4) as usize);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            data.extend_from_slice(&color_at(x, y));
        }
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_surface_maps_are_complete_and_repeatable() {
        let asphalt = asphalt_pixels();
        assert_eq!(asphalt.len(), (TILE_SIZE * TILE_SIZE * 4) as usize);
        assert_eq!(asphalt, asphalt_pixels());
        assert_ne!(asphalt, pavement_pixels());
        assert_ne!(facade_pixels(), ground_pixels());
        assert!(asphalt.chunks_exact(4).any(|pixel| pixel[0] > 50));
        assert!(asphalt.chunks_exact(4).any(|pixel| pixel[0] < 25));
    }
}
