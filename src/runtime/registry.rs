use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, thiserror::Error)]
#[error("Failed to convert entry")]
pub struct Error;

/// Registry to store any type by name/identifier respectively a hash map of any object.
#[derive(Default)]
pub struct Type {
    data: HashMap<String, Box<dyn Any>>,
}

impl Type {
    /// Insert object into registry. Will return the old value of this `key` (if exist)
    pub fn insert<T: 'static>(
        &mut self,
        key: impl Into<String>,
        value: Box<T>,
    ) -> Option<Box<dyn Any>> {
        self.data.insert(key.into(), value)
    }

    /// Get value of entry with `key`. Will return `Ok(None)` if key not exists.
    ///
    /// # Errors
    ///
    /// Will return `Err` if value exists but cannot converted into `&T`.
    pub fn get<Q: ?Sized, T: 'static>(&self, key: &Q) -> Result<Option<&T>, Error>
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.data
            .get(key)
            .map(|v| v.downcast_ref::<T>())
            .map(|v| v.ok_or(Error))
            .transpose()
    }

    /// Same as [`Self::get`] but returns a mutable reference to `T`. Will return `Ok(None)` if key
    /// not exists.
    ///
    /// # Errors
    ///
    /// Will return `Err` if value exists but cannot converted into `&mut T`
    pub fn get_mut<Q: ?Sized, T: 'static>(&mut self, key: &Q) -> Result<Option<&mut T>, Error>
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.data
            .get_mut(key)
            .map(|v| v.downcast_mut::<T>())
            .map(|v| v.ok_or(Error))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::Type;

    #[test]
    pub fn default() {
        let _reg = Type::default();
    }

    #[test]
    pub fn insert() {
        let mut registry = Type::default();

        registry.insert("key", Box::new(42_i32));
    }

    #[test]
    pub fn get() {
        let mut registry = Type::default();

        registry.insert("key", Box::new(42_i32));
        assert_eq!(*registry.get::<_, i32>("key").unwrap().unwrap(), 42)
    }

    #[test]
    pub fn get_mut() {
        let mut registry = Type::default();

        registry.insert("key", Box::new(42_i32));
        assert_eq!(*registry.get_mut::<_, i32>("key").unwrap().unwrap(), 42)
    }

    #[test]
    pub fn get_missing() {
        let mut registry = Type::default();

        registry.insert("key", Box::new(42_i32));
        assert!(registry.get_mut::<_, i32>("key2").unwrap().is_none())
    }

    #[test]
    pub fn get_error() {
        let mut registry = Type::default();

        registry.insert("key", Box::new(42_i32));
        assert!(registry.get_mut::<_, usize>("key").is_err())
    }
}
