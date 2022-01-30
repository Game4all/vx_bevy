use std::{hash::Hash, marker::PhantomData};

use bevy::{math::IVec3, utils::HashMap};
use ndshape::Shape;

use super::buffer::VoxelBuffer;

/// A strongly typed key pointing to the origin of a voxel buffer in a [`VoxelMap<V, S>`]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VoxelMapKey<V: Clone + Copy + Default + Eq + Hash>(IVec3, PhantomData<V>);

impl<V: Clone + Copy + Default + Eq + Hash> VoxelMapKey<V> {
    /// Constructs a key from the given coordinates
    pub fn from_ivec3(pos: IVec3) -> Self {
        Self(pos, Default::default())
    }

    /// Returns an [`IVec3`] pointing to the origin point of a voxel buffer.
    #[inline]
    pub fn location(&self) -> IVec3 {
        self.0
    }
}

/// Provides an interface to query or modify voxel data for worlds or scenes split into multiple voxel data buffers of a same shape with no level of detail.
pub struct VoxelMap<V, S>
where
    V: Clone + Copy + Default + PartialEq + Eq + Hash,
    S: Shape<u32, 3> + Clone,
{
    pub chunks: HashMap<VoxelMapKey<V>, VoxelBuffer<V, S>>,
    shape: S,
}

#[allow(dead_code)]
impl<V, S> VoxelMap<V, S>
where
    V: Clone + Copy + Default + PartialEq + Eq + Hash,
    S: Shape<u32, 3> + Clone,
{
    pub fn new(chunk_shape: S) -> Self {
        Self {
            chunks: Default::default(),
            shape: chunk_shape,
        }
    }

    /// Checks whether there's a buffer at the specified origin.
    #[inline]
    pub fn exists(&self, origin: VoxelMapKey<V>) -> bool {
        self.chunks.contains_key(&origin)
    }

    /// Returns a reference to the [`VoxelBuffer<V, S>`] at the specified origin if there's one.
    #[inline]
    pub fn buffer_at(&self, origin: VoxelMapKey<V>) -> Option<&VoxelBuffer<V, S>> {
        self.chunks.get(&origin)
    }

    /// Returns a mutable reference to the [`VoxelBuffer<V, S>`] at the specified origin if there's one.
    #[inline]
    pub fn buffer_at_mut(&mut self, origin: VoxelMapKey<V>) -> Option<&mut VoxelBuffer<V, S>> {
        self.chunks.get_mut(&origin)
    }

    /// Inserts a new buffer at the specified origin.
    pub fn insert(&mut self, origin: VoxelMapKey<V>, buffer: VoxelBuffer<V, S>) {
        assert!(buffer.shape().as_array() == self.shape.as_array());
        self.chunks.insert(origin, buffer);
    }

    /// Inserts a new buffer inititalized with the default value of [`V`] at the specified origin.
    pub fn insert_empty(&mut self, origin: VoxelMapKey<V>) {
        self.chunks
            .insert(origin, VoxelBuffer::<V, S>::new_empty(self.shape.clone()));
    }

    /// Removes the buffer at the specified origin and returns it if it exists.
    pub fn remove(&mut self, pos: VoxelMapKey<V>) -> Option<VoxelBuffer<V, S>> {
        self.chunks.remove(&pos)
    }
}
