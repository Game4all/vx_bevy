use bevy::math::UVec3;
use ndshape::Shape;

/// A buffer of typed voxel data stored as a contiguous array in memory.
#[allow(dead_code)]
pub struct VoxelBuffer<V: Copy + Clone, S: Shape<u32, 3>> {
    data: Box<[V]>,
    shape: S,
}

#[allow(dead_code)]
impl<V: Copy + Clone, S: Shape<u32, 3>> VoxelBuffer<V, S> {
    #[inline]
    pub fn new(shape: S, initial_val: V) -> Self {
        Self {
            data: vec![initial_val.clone(); shape.size() as usize].into_boxed_slice(),
            shape,
        }
    }

    // Returns the voxel at the querried position in local space.
    #[inline]
    pub fn voxel_at(&self, pos: UVec3) -> V {
        self.data[self.shape.linearize(pos.to_array()) as usize]
    }

    // Returns a mutable reference to the the voxel at the querried position in local space.
    #[inline]
    pub fn voxel_at_mut(&mut self, pos: UVec3) -> &mut V {
        &mut self.data[self.shape.linearize(pos.to_array()) as usize]
    }

    #[inline]
    pub fn slice(&self) -> &[V] {
        &self.data
    }

    #[inline]
    pub fn slice_mut(&mut self) -> &mut [V] {
        &mut self.data
    }

    #[inline]
    pub fn shape(&self) -> &S {
        &self.shape
    }
}

// kinda of a helper with voxel types with default implemented.
impl<V: Copy + Clone + Default, S: Shape<u32, 3>> VoxelBuffer<V, S> {
    pub fn new_empty(shape: S) -> Self {
        Self::new(shape, Default::default())
    }
}
