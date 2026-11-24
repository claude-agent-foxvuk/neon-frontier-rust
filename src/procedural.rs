use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

const TILE_SIZE: u32 = 256;

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
        let aggregate = noise(x / 2, y / 2, 19) as i16;
        let broad = noise(x / 8, y / 8, 101) as i16 - 15;
        let patch = noise(x / 24, y / 24, 127);
        let repair = if patch > 28 { -18 } else if patch < 4 { 8 } else { 0 };
        let tar_fleck = if aggregate > 27 { -22 } else { 0 };
        let crack = if noise(x / 12, y / 12, 223) > 29
            && noise(x / 3, y / 3, 229) < 12
        {
            -16
        } else {
            0
        };
        let shade = grain * 2 + broad + repair + tar_fleck + crack;
        [
            (39_i16 + shade).clamp(0, 255) as u8,
            (47_i16 + shade).clamp(0, 255) as u8,
            (58_i16 + shade).clamp(0, 255) as u8,
            255,
        ]
    })
}

fn pavement_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let local_x = x % 32;
        let local_y = y % 32;
        let joint = local_x < 2 || local_y < 2;
        let slab_x = x / 32;
        let slab_y = y / 32;
        let grain = noise(x, y, 31) as i16 - 15;
        let slab = noise(slab_x, slab_y, 83) as i16 - 15;
        let stain = if noise(x / 8, y / 8, 137) > 27 { -14 } else { 0 };
        let grit = if noise(x / 2, y / 2, 151) > 28 { 12 } else { 0 };
        if joint {
            [34, 44, 53, 255]
        } else {
            let detail = grain + slab / 2 + stain + grit;
            [
                (102_i16 + detail).clamp(0, 255) as u8,
                (117_i16 + detail).clamp(0, 255) as u8,
                (126_i16 + detail).clamp(0, 255) as u8,
                255,
            ]
        }
    })
}

fn facade_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let bay_x = x / 32;
        let bay_y = y / 32;
        let local_x = x % 32;
        let local_y = y % 32;
        let inset_x = 4 + (noise(bay_x, bay_y, 211) % 3) as u32;
        let inset_y = 4 + (noise(bay_x, bay_y, 223) % 4) as u32;
        let window_width = 19 + (noise(bay_x, bay_y, 227) % 7) as u32;
        let window_height = 19 + (noise(bay_x, bay_y, 229) % 7) as u32;
        let in_window = (inset_x..inset_x + window_width).contains(&local_x)
            && (inset_y..inset_y + window_height).contains(&local_y);
        let frame = in_window
            || local_x + 1 == inset_x
            || local_x == inset_x + window_width
            || local_y + 1 == inset_y
            || local_y == inset_y + window_height;
        if frame {
            [31, 48, 59, 255]
        } else if in_window {
            let variation = noise(bay_x, bay_y, 53);
            if variation > 15 {
                let glow = variation as i16 / 3;
                [
                    (151_i16 + glow).clamp(0, 255) as u8,
                    (108_i16 + glow / 2).clamp(0, 255) as u8,
                    (72_i16 + glow / 3).clamp(0, 255) as u8,
                    255,
                ]
            } else {
                [18, 33, 48, 255]
            }
        } else {
            let grain = noise(x, y, 71) as i16 - 15;
            let weathering = if noise(x / 5, y / 5, 149) > 27 {
                -15
            } else {
                0
            };
            let streak = if noise(x / 16, y / 32, 241) > 29 { -12 } else { 0 };
            let groove = if local_x == 0 || local_x == 31 { -10 } else { 0 };
            let detail = grain + weathering + streak + groove;
            [
                (78_i16 + detail).clamp(0, 255) as u8,
                (94_i16 + detail).clamp(0, 255) as u8,
                (108_i16 + detail).clamp(0, 255) as u8,
                255,
            ]
        }
    })
}

fn ground_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let fine = noise(x, y, 89) as i16 - 15;
        let patch = noise(x / 8, y / 8, 163) as i16 - 15;
        let broad = noise(x / 32, y / 32, 181) as i16 - 15;
        let detail = fine / 2 + patch + broad / 2;
        let fleck = if noise(x / 2, y / 2, 193) > 29 { 10 } else { 0 };
        [
            (30_i16 + detail / 2 + fleck).clamp(0, 255) as u8,
            (72_i16 + detail + fleck).clamp(0, 255) as u8,
            (56_i16 + detail / 2 + fleck / 2).clamp(0, 255) as u8,
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

    #[test]
    fn larger_surface_tiles_add_more_spatial_variation() {
        assert_eq!(TILE_SIZE, 256);
        assert!(facade_pixels().chunks_exact(4).any(|pixel| pixel[0] > 150));
        assert!(pavement_pixels().chunks_exact(4).any(|pixel| pixel[0] < 50));
    }
}
