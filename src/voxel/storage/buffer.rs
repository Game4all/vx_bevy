use bevy::math::UVec3;
use ndshape::Shape;

/// A buffer of typed voxel data stored as a contiguous array in memory.
#[allow(dead_code)]
pub struct VoxelBuffer<V, S: Shape<u32, 3>>
where
    V: Copy + Clone + Default,
{
    data: Box<[V]>,
    shape: S,
}

#[allow(dead_code)]
impl<V, S: Shape<u32, 3>> VoxelBuffer<V, S>
where
    V: Copy + Clone + Default,
{
    #[inline]
    pub fn new(shape: S, initial_val: V) -> Self {
        Self {
            data: vec![initial_val.clone(); shape.size() as usize].into_boxed_slice(),
            shape,
        }
    }

    #[inline]
    pub fn new_empty(shape: S) -> Self {
        Self {
            data: vec![Default::default(); shape.size() as usize].into_boxed_slice(),
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
