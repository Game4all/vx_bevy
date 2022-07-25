use std::{
    cmp::Ordering,
    collections::BTreeMap,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use bevy::math::IVec3;
use ndshape::Shape;

use super::buffer::VoxelBuffer;

/// A strongly typed key representing the minimum of a buffer stored in a [`VoxelMap<V,S>`]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VoxelMapKey(IVec3);

impl VoxelMapKey {
    #[inline]
    pub fn new(position: IVec3) -> Self {
        Self(position)
    }

    #[inline]
    /// Computes the distance between two chunk keys.
    pub fn distance(&self, other: &Self) -> u32 {
        let delta = self.0 - other.0;
        (delta.x.pow(2) + delta.y.pow(2) + delta.z.pow(2)) as u32
    }
}

impl From<IVec3> for VoxelMapKey {
    #[inline]
    fn from(p: IVec3) -> Self {
        Self::new(p)
    }
}

impl Deref for VoxelMapKey {
    type Target = IVec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoxelMapKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialOrd for VoxelMapKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.0.x, self.0.y, self.0.z).partial_cmp(&(other.0.x, other.0.y, other.0.z))
    }
}

impl Ord for VoxelMapKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Provides an interface to query or modify voxel data for worlds or scenes split into multiple voxel data buffers of a same shape with no level of detail.
pub struct VoxelMap<V, S>
where
    V: Clone + Copy + Default + PartialEq + Eq + Hash,
    S: Shape<3, Coord = u32> + Clone,
{
    chunks: BTreeMap<VoxelMapKey, VoxelBuffer<V, S>>,
    shape: S,
}

#[allow(dead_code)]
impl<V, S> VoxelMap<V, S>
where
    V: Clone + Copy + Default + PartialEq + Eq + Hash,
    S: Shape<3, Coord = u32> + Clone,
{
    pub fn new(chunk_shape: S) -> Self {
        Self {
            chunks: Default::default(),
            shape: chunk_shape,
        }
    }

    /// Checks whether there's a buffer at the specified minimum.
    #[inline]
    pub fn exists(&self, minimum: VoxelMapKey) -> bool {
        self.chunks.contains_key(&minimum)
    }

    /// Returns a reference to the [`VoxelBuffer<V, S>`] at the specified minimum if there's one.
    #[inline]
    pub fn buffer_at(&self, minimum: VoxelMapKey) -> Option<&VoxelBuffer<V, S>> {
        self.chunks.get(&minimum)
    }

    /// Returns a mutable reference to the [`VoxelBuffer<V, S>`] at the specified minimum if there's one.
    #[inline]
    pub fn buffer_at_mut(&mut self, minimum: VoxelMapKey) -> Option<&mut VoxelBuffer<V, S>> {
        self.chunks.get_mut(&minimum)
    }

    /// Inserts a new buffer at the specified minimum.
    pub fn insert(&mut self, minimum: VoxelMapKey, buffer: VoxelBuffer<V, S>) {
        assert!(buffer.shape().as_array() == self.shape.as_array());
        self.chunks.insert(minimum, buffer);
    }

    /// Inserts a new buffer inititalized with the default value of [`V`] at the specified minimum.
    pub fn insert_empty(&mut self, minimum: VoxelMapKey) {
        self.chunks
            .insert(minimum, VoxelBuffer::<V, S>::new_empty(self.shape.clone()));
    }

    /// Inserts buffers from an iterator passed as a parameter
    pub fn insert_batch<T: IntoIterator<Item = (VoxelMapKey, VoxelBuffer<V, S>)>>(
        &mut self,
        iter: T,
    ) {
        self.chunks.extend(iter);
    }

    /// Removes the buffer at the specified minimum and returns it if it exists.
    pub fn remove(&mut self, pos: &VoxelMapKey) -> Option<VoxelBuffer<V, S>> {
        self.chunks.remove(&pos)
    }
}
