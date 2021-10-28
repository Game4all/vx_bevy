pub struct ValueMap2D<T: Copy> {
    values: Vec<T>,
    width: i32,
    height: i32,
}

impl<T: Copy> ValueMap2D<T> {
    pub fn new(width: i32, height: i32, values: Vec<T>) -> Self {
        assert!(width * height == values.len() as i32);

        Self {
            values,
            width,
            height,
        }
    }

    #[inline]
    pub fn value_at(&self, x: i32, y: i32) -> T {
        assert!(self.width > x && self.height > y);
        unsafe { *self.values.get_unchecked((y * self.width + x) as usize) }
    }
}

pub struct ValueMap3D<T: Copy> {
    values: Vec<T>,
    width: i32,
    height: i32,
    depth: i32,
}

impl<T: Copy> ValueMap3D<T> {
    pub fn new(width: i32, height: i32, depth: i32, values: Vec<T>) -> Self {
        assert!(width * height * depth == values.len() as i32);

        Self {
            values,
            width,
            height,
            depth,
        }
    }

    #[inline]
    pub fn value_at(&self, x: i32, y: i32, z: i32) -> T {
        assert!(self.width > x && self.height > y && self.depth > z);
        unsafe {
            *self
                .values
                .get_unchecked(((self.width * self.height * z) + (self.width * y) + x) as usize)
        }
    }
}
