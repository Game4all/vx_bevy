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
