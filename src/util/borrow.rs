use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

/// Implements a thread safe data structure which can be used to temporary borrow an inner value.
pub struct Cell<T> {
    inner: Arc<Mutex<Option<T>>>,
}

/// Used to wrap a borrowed value. Will return the borrowed value then dropped.
pub struct Borrowed<T> {
    ret: Arc<Mutex<Option<T>>>,
    value: Option<T>,
}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(value))),
        }
    }

    /// Returns `true` if the inner value is available to borrow. Returns `false` if value already
    /// borrowed.
    pub fn is_available(&self) -> bool {
        self.inner.lock().unwrap().is_some()
    }

    /// Tries to borrow the inner value. Will return `None` if inner value couldn't be borrowed.
    pub fn try_borrow(&self) -> Option<Borrowed<T>> {
        if let Some(value) = self.inner.lock().unwrap().take() {
            Some(Borrowed {
                ret: self.inner.clone(),
                value: Some(value),
            })
        } else {
            None
        }
    }
}

impl<T: Clone> Cell<T> {
    /// Tries to clone inner cell. Will return `None` if inner value is borrowed.
    pub fn fork(&self) -> Option<Cell<T>> {
        if let Some(inner) = self.try_borrow() {
            return Some(Cell::new(inner.clone()));
        }
        None
    }
}

impl<T> Drop for Borrowed<T> {
    fn drop(&mut self) {
        let value = self.value.take().expect("can only be dropped once");
        self.ret.lock().unwrap().replace(value);
    }
}

impl<T> Deref for Borrowed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().expect("already dropped")
    }
}

impl<T> DerefMut for Borrowed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().expect("already dropped")
    }
}
