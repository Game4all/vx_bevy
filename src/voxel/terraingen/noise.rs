use bevy::math::{Vec2, Vec2Swizzles, Vec3, Vec3Swizzles};

use crate::voxel::ChunkKey;

pub fn rand2to1(p: Vec2, dot: Vec2) -> f32 {
    let sp: Vec2 = p.to_array().map(|x| x.sin()).into();
    let random = sp.dot(dot);
    return (random.sin() * 143758.5453).fract();
}

#[inline(always)]
pub fn rand2to2(p: Vec2) -> Vec2 {
    Vec2::new(
        rand2to1(p, Vec2::new(12.989, 78.233)),
        rand2to1(p, Vec2::new(39.346, 11.135)),
    )
}

pub fn rand2to1i(vec: Vec2) -> f32 {
    let mut p3 = (vec.xyx() * 0.39).fract();
    p3 += p3.dot(p3.yzx());
    return (p3.x + p3.y) * p3.z.fract();
}

#[allow(dead_code)]
#[inline(always)]
pub fn rand2to3(p: Vec2) -> Vec3 {
    Vec3::new(
        rand2to1(p, Vec2::new(12.989, 78.233)),
        rand2to1(p, Vec2::new(39.346, 11.135)),
        rand2to1(p, Vec2::new(73.156, 52.235)),
    )
}

#[allow(dead_code)]
#[inline(always)]
pub fn rand1dto1d(p: f32, mutator: f32) -> f32 {
    let random = (p + mutator).sin();
    return (random * 143758.5453).fract();
}

#[allow(dead_code)]
#[inline(always)]
pub fn rand1to3(p: f32) -> Vec3 {
    Vec3::new(
        rand1dto1d(p, 3.9812),
        rand1dto1d(p, 1.2345),
        rand1dto1d(p, 5.4321),
    )
}

// This was ported from the code at https://www.ronja-tutorials.com/post/028-voronoi-noise/
pub fn voronoi(p: Vec2) -> Vec2 {
    const NEIGHBOUR_RANGE: i32 = 2; // A neighbour range of 1 will generate some weird artifacts lets use 2.

    let base_cell = p.floor();
    let mut closest_point = base_cell;
    let mut min_distance = 1f32;

    for x in -NEIGHBOUR_RANGE..=NEIGHBOUR_RANGE {
        for y in -NEIGHBOUR_RANGE..=NEIGHBOUR_RANGE {
            let cell = base_cell + Vec2::new(x as f32, y as f32);
            let cell_pos = cell + rand2to2(cell);
            let distance = (cell_pos - p).length_squared(); // using non squarred length to increase the throughput (a bit)

            if distance < min_distance {
                min_distance = distance;
                closest_point = cell;
            }
        }
    }

    return closest_point;
}

pub fn generate_heightmap_data(key: ChunkKey, chunk_len: usize) -> Vec<f32> {
    simdnoise::NoiseBuilder::fbm_2d_offset(
        key.location().x as f32,
        chunk_len,
        key.location().z as f32,
        chunk_len,
    )
    .with_octaves(4)
    .generate()
    .0
    .iter()
    .map(|x| 128.0 + x * 5.0)
    .collect()
}

/// A view into a slice of noise values with W x H dimensions.
/// Provides methods for fetching a value at specified coordinates and to map values to a range.
#[derive(Clone, Copy)]
pub struct Heightmap<'a, T: Copy, const W: usize, const H: usize> {
    slice: &'a [T],
}

impl<'a, T: Copy, const W: usize, const H: usize> Heightmap<'a, T, W, H> {
    /// Gets the value at the specified coordinates.
    #[inline]
    pub fn get(&self, pos: [u32; 2]) -> T {
        self.slice[pos[1] as usize * W + pos[0] as usize]
    }

    /// Constructs a view into a slice of noise values with W x H dimensions.
    #[inline]
    pub fn from_slice(slice: &'a [T]) -> Self {
        Self { slice }
    }
}
