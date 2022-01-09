use std::sync::Arc;
use thread_local::ThreadLocal;

// A resource with a thread-local storage of its instances.
pub struct ThreadLocalRes<T: Send + Default> {
    tls: Arc<ThreadLocal<T>>,
}

impl<T: Send + Default> Default for ThreadLocalRes<T> {
    fn default() -> Self {
        Self {
            tls: Default::default(),
        }
    }
}

impl<T: Send + Default> ThreadLocalRes<T> {
    pub fn get_handle(&self) -> ThreadLocalResHandle<T> {
        ThreadLocalResHandle {
            handle: self.tls.clone(),
        }
    }
}

pub struct ThreadLocalResHandle<T: Send + Default> {
    handle: Arc<ThreadLocal<T>>,
}

impl<T: Send + Default> ThreadLocalResHandle<T> {
    pub fn get(&self) -> &T {
        &self.handle.get_or_default()
    }
}
