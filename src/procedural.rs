use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

const TILE_SIZE: u32 = 512;

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

fn fine_grain(x: u32, y: u32, salt: u32) -> i16 {
    let mut value =
        x.wrapping_mul(0x9E37_79B9) ^ y.wrapping_mul(0x85EB_CA6B) ^ salt.wrapping_mul(0xC2B2_AE35);
    value ^= value >> 15;
    value = value.wrapping_mul(0x2C1B_3C6D);
    value ^= value >> 12;
    ((value >> 24) & 0xff) as i16 - 128
}

fn asphalt_pixels() -> Vec<u8> {
    pixels(|x, y| {
        let grain = noise(x, y, 7) as i16 - 15;
        let micro = fine_grain(x, y, 257) / 10;
        let aggregate = noise(x / 3, y / 3, 19) as i16;
        let coarse = noise(x / 8, y / 8, 337) as i16 - 15;
        let broad = noise(x / 16, y / 16, 101) as i16 - 15;
        let patch = noise(x / 48, y / 48, 127);
        let repair = if patch > 28 {
            -18
        } else if patch < 4 {
            8
        } else {
            0
        };
        let tar_fleck = if aggregate > 29 { -24 } else { 0 };
        let crack_vein = noise(x / 24, y / 24, 223) > 29 && noise(x / 6, y / 6, 229) < 10;
        let crack = if crack_vein { -20 } else { 0 };
        let pothole = if noise(x / 32, y / 32, 349) > 30 && noise(x / 4, y / 4, 351) > 22 {
            -26
        } else {
            0
        };
        let grit = if noise(x / 2, y / 2, 353) > 30 { 9 } else { 0 };
        let shade =
            grain * 2 + micro + coarse + broad + repair + tar_fleck + crack + pothole + grit;
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
        let slab_x = x / 64;
        let slab_y = y / 64;
        let local_x = x % 64;
        let local_y = y % 64;
        let joint = local_x < 3 || local_y < 3;
        if joint {
            let damp = noise(x, y, 471) as i16 - 15;
            let moss = if noise(x / 2, y / 2, 467) > 26 { 9 } else { 0 };
            [
                (34_i16 + damp / 3).clamp(0, 255) as u8,
                (44_i16 + damp / 3 + moss).clamp(0, 255) as u8,
                (53_i16 + damp / 3).clamp(0, 255) as u8,
                255,
            ]
        } else {
            let grain = noise(x, y, 31) as i16 - 15;
            let tint = noise(slab_x, slab_y, 83) as i16 - 15;
            let grit = if noise(x / 2, y / 2, 151) > 28 { 10 } else { 0 };
            let stain = if noise(x / 16, y / 16, 137) > 27 {
                -14
            } else {
                0
            };
            let gum = if noise(x / 4, y / 4, 457) > 30 && noise(x, y, 461) > 20 {
                -22
            } else {
                0
            };
            let chalk = if noise(x / 8, y / 8, 463) > 30 { 8 } else { 0 };
            let micro = fine_grain(x, y, 263) / 14;
            let detail = grain + tint / 2 + grit + stain + gum + chalk + micro;
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
        let bay_x = x / 64;
        let bay_y = y / 64;
        let local_x = x % 64;
        let local_y = y % 64;
        let inset_x = 8 + (noise(bay_x, bay_y, 211) % 5) as u32;
        let inset_y = 8 + (noise(bay_x, bay_y, 223) % 6) as u32;
        let window_width = 38 + (noise(bay_x, bay_y, 227) % 7) as u32;
        let window_height = 40 + (noise(bay_x, bay_y, 229) % 9) as u32;
        let window_right = (inset_x + window_width).min(62);
        let window_bottom = (inset_y + window_height).min(60);
        let in_window_x = inset_x <= local_x && local_x < window_right;
        let in_window_y = inset_y <= local_y && local_y < window_bottom;
        let in_window = in_window_x && in_window_y;
        let frame = in_window
            && (local_x == inset_x
                || local_x + 1 == window_right
                || local_y == inset_y
                || local_y + 1 == window_bottom);
        let grain = noise(x, y, 71) as i16 - 15;
        let ac_unit = !in_window
            && (44..56).contains(&local_x)
            && (52..60).contains(&local_y)
            && noise(bay_x, bay_y, 421) > 24;
        if frame {
            [26, 40, 50, 255]
        } else if in_window {
            let lit = noise(bay_x, bay_y, 53);
            let depth = (local_y as i16 - inset_y as i16).max(0);
            let height = (window_height as i16).max(1);
            if lit > 13 {
                let glow = (74 + lit as i16 * 2 - depth * 28 / height).clamp(0, 255);
                [
                    (146_i16 + glow / 3).clamp(0, 255) as u8,
                    (116_i16 + glow / 4).clamp(0, 255) as u8,
                    (74_i16 + glow / 6).clamp(0, 255) as u8,
                    255,
                ]
            } else {
                let sheen = (26 - depth * 22 / height).max(-6);
                [
                    (18_i16 + sheen).clamp(0, 255) as u8,
                    (33_i16 + sheen + lit as i16 / 4).clamp(0, 255) as u8,
                    (48_i16 + sheen + lit as i16 / 3).clamp(0, 255) as u8,
                    255,
                ]
            }
        } else if ac_unit {
            [
                (96_i16 + grain / 3).clamp(0, 255) as u8,
                (104_i16 + grain / 3).clamp(0, 255) as u8,
                (108_i16 + grain / 3).clamp(0, 255) as u8,
                255,
            ]
        } else {
            let weathering = if noise(x / 5, y / 5, 149) > 27 { -15 } else { 0 };
            let streak = if noise(x / 32, y / 32, 241) > 29 { -12 } else { 0 };
            let groove = if local_x == 0 || local_x == 63 { -12 } else { 0 };
            let sill = if local_y == window_bottom && in_window_x {
                -18
            } else {
                0
            };
            let detail = grain + weathering + streak + groove + sill + fine_grain(x, y, 269) / 24;
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
        let fine = fine_grain(x, y, 89) / 16;
        let patch = noise(x / 16, y / 16, 163) as i16 - 15;
        let broad = noise(x / 64, y / 64, 181) as i16 - 15;
        let fleck = if noise(x / 2, y / 2, 193) > 29 { 10 } else { 0 };
        let tuft = if noise(x / 4, y / 4, 433) > 27 { 12 } else { 0 };
        let dry = if noise(x / 64, y / 64, 443) > 28 { 14 } else { 0 };
        let stone = if noise(x / 8, y / 8, 447) > 30 { 16 } else { 0 };
        let detail = fine + patch + broad / 2 + fleck + tuft + stone;
        [
            (30_i16 + detail / 2 + dry).clamp(0, 255) as u8,
            (72_i16 + detail).clamp(0, 255) as u8,
            (56_i16 + detail / 3 + dry / 2).clamp(0, 255) as u8,
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

    for y in 0..TILE_SIZE {
        let first = (y * TILE_SIZE * 4) as usize;
        let last = (y * TILE_SIZE + TILE_SIZE - 1) as usize * 4;
        let edge = [
            data[first],
            data[first + 1],
            data[first + 2],
            data[first + 3],
        ];
        data[last..last + 4].copy_from_slice(&edge);
    }
    let row_bytes = (TILE_SIZE * 4) as usize;
    let first_row = data[..row_bytes].to_vec();
    let last_row = ((TILE_SIZE - 1) * row_bytes as u32) as usize;
    data[last_row..last_row + row_bytes].copy_from_slice(&first_row);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tile_edges_match(texture: &[u8]) {
        let row_bytes = (TILE_SIZE * 4) as usize;
        for y in 0..TILE_SIZE as usize {
            let first = y * row_bytes;
            let last = first + row_bytes - 4;
            assert_eq!(&texture[first..first + 4], &texture[last..last + 4]);
        }
        for x in 0..TILE_SIZE as usize {
            let first = x * 4;
            let last = (TILE_SIZE as usize - 1) * row_bytes + first;
            assert_eq!(&texture[first..first + 4], &texture[last..last + 4]);
        }
    }

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
    fn surface_maps_have_broad_color_ranges() {
        let facade = facade_pixels();
        assert!(facade.chunks_exact(4).any(|pixel| pixel[0] > 150));
        assert!(facade.chunks_exact(4).any(|pixel| pixel[0] < 35));
        assert!(pavement_pixels().chunks_exact(4).any(|pixel| pixel[0] < 50));
    }

    #[test]
    fn facade_mixes_lit_windows_with_sheened_glass() {
        let facade = facade_pixels();
        let bright = facade.chunks_exact(4).filter(|pixel| pixel[0] > 150).count();
        let dark = facade.chunks_exact(4).filter(|pixel| pixel[2] > pixel[0]).count();
        assert!(bright > 2_000, "lit windows should cover a visible share");
        assert!(dark > 2_000, "unlit glass should reflect the sky");
    }

    #[test]
    fn repeatable_surface_maps_have_matching_tile_edges() {
        for texture in [
            asphalt_pixels(),
            pavement_pixels(),
            facade_pixels(),
            ground_pixels(),
        ] {
            assert_tile_edges_match(&texture);
        }
    }

    #[test]
    fn fine_grain_is_deterministic_and_varies_between_pixels() {
        assert_eq!(fine_grain(20, 40, 7), fine_grain(20, 40, 7));
        assert_ne!(fine_grain(20, 40, 7), fine_grain(21, 40, 7));
        assert_ne!(fine_grain(20, 40, 7), fine_grain(20, 40, 8));
    }
}
