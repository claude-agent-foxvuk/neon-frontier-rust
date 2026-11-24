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
    let mut value = x.wrapping_mul(1_103_515_245)
        ^ y.wrapping_mul(12_345)
        ^ salt.wrapping_mul(2_654_435_761);
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    ((value ^ (value >> 16)) & 31) as u8
}

fn asphalt_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let grain = noise(x, y, 7) as i16 - 15;
        let fleck = if noise(x / 3, y / 3, 19) > 26 { 10 } else { 0 };
        [
            (37_i16 + grain + fleck).clamp(0, 255) as u8,
            (46_i16 + grain + fleck).clamp(0, 255) as u8,
            (59_i16 + grain + fleck).clamp(0, 255) as u8,
            255,
        ]
    })
}

fn pavement_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let joint = x % 32 < 2 || y % 24 < 2;
        let grain = noise(x, y, 31) as i16 - 15;
        if joint {
            [46, 59, 68, 255]
        } else {
            [
                (103_i16 + grain).clamp(0, 255) as u8,
                (118_i16 + grain).clamp(0, 255) as u8,
                (126_i16 + grain).clamp(0, 255) as u8,
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
            let lit = noise(x / 32, y / 24, 53) > 18;
            if lit {
                [162, 127, 83, 255]
            } else {
                [23, 43, 61, 255]
            }
        } else if local_x == 4 || local_x == 27 || local_y == 4 || local_y == 20 {
            [51, 72, 87, 255]
        } else {
            let grain = noise(x, y, 71) as i16 - 15;
            [
                (75_i16 + grain).clamp(0, 255) as u8,
                (91_i16 + grain).clamp(0, 255) as u8,
                (107_i16 + grain).clamp(0, 255) as u8,
                255,
            ]
        }
    })
}

fn ground_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let variation = noise(x, y, 89) as i16 - 15;
        [
            (28_i16 + variation / 2).clamp(0, 255) as u8,
            (70_i16 + variation).clamp(0, 255) as u8,
            (57_i16 + variation / 2).clamp(0, 255) as u8,
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
    }
}
