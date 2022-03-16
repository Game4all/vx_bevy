use std::{cmp::Ordering, collections::BTreeMap, hash::Hash, marker::PhantomData};

use bevy::math::IVec3;
use ndshape::Shape;

use super::buffer::VoxelBuffer;

/// A strongly typed key pointing to the origin of a voxel buffer in a [`VoxelMap<V, S>`]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VoxelMapKey<V>(IVec3, PhantomData<V>)
where
    V: Clone + Copy + Default + Eq + Hash;

impl<V> VoxelMapKey<V>
where
    V: Clone + Copy + Default + Eq + Hash,
{
    /// Constructs a key from the given coordinates
    #[inline]
    pub fn from_ivec3(pos: IVec3) -> Self {
        Self(pos, Default::default())
    }

    /// Returns an [`IVec3`] pointing to the origin point of a voxel buffer.
    #[inline]
    pub fn location(&self) -> IVec3 {
        self.0
    }

    #[inline]
    /// Computes the distance between two chunk keys.
    pub fn distance(&self, other: &Self) -> u32 {
        let delta = self.location() - other.location();
        (delta.x.pow(2) + delta.y.pow(2) + delta.z.pow(2)) as u32
    }
}

impl<V> PartialOrd for VoxelMapKey<V>
where
    V: Clone + Copy + Default + Eq + Hash,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.0.x, self.0.y, self.0.z).partial_cmp(&(other.0.x, other.0.y, other.0.z))
    }
}

impl<V> Ord for VoxelMapKey<V>
where
    V: Clone + Copy + Default + Eq + Hash,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Provides an interface to query or modify voxel data for worlds or scenes split into multiple voxel data buffers of a same shape with no level of detail.
pub struct VoxelMap<V, S>
where
    V: Clone + Copy + Default + PartialEq + Eq + Hash,
    S: Shape<u32, 3> + Clone,
{
    pub chunks: BTreeMap<VoxelMapKey<V>, VoxelBuffer<V, S>>,
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

    /// Inserts buffers from an iterator passed as a parameter
    pub fn insert_batch<T: IntoIterator<Item = (VoxelMapKey<V>, VoxelBuffer<V, S>)>>(
        &mut self,
        iter: T,
    ) {
        self.chunks.extend(iter);
    }

    /// Removes the buffer at the specified origin and returns it if it exists.
    pub fn remove(&mut self, pos: &VoxelMapKey<V>) -> Option<VoxelBuffer<V, S>> {
        self.chunks.remove(&pos)
    }
}
