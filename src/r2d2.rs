use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use r2d2::{ManageConnection, PooledConnection};

/// Used to manage a pool of connections of type `C`. Uses an `r2d2` connection pool.
///
/// # Usage
///
/// ```rust
/// let conn = Connection::new(/* your connection */);
///
/// // ..
///
/// async {
///     let c = conn.get().await;
/// }
/// ````
pub struct Connection<C: ManageConnection> {
    pool: r2d2::Pool<C>,
}

/// Single use future to await an available connection.
pub struct GetConnection<'a, C: ManageConnection> {
    inner: Option<&'a mut Connection<C>>,
}

impl<C: ManageConnection> Connection<C> {
    /// Creates a new connection pool.
    ///
    /// # Parameters
    /// - `conn` Connection configuration
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to create connection pool.
    pub fn new(conn: C) -> Result<Self, r2d2::Error> {
        Ok(Self {
            pool: r2d2::Pool::new(conn)?,
        })
    }

    /// Returns future to await new connection.
    ///
    /// # Usage
    ///
    /// ```rust
    /// async {
    ///     let conn = connection.get().await;
    /// }
    /// ```
    pub fn get(&mut self) -> GetConnection<'_, C> {
        GetConnection { inner: Some(self) }
    }
}

impl<'a, C: ManageConnection> Future for GetConnection<'a, C> {
    type Output = PooledConnection<C>;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let c = match self
            .inner
            .as_mut()
            .expect("Unexpected poll after Poll::Ready")
            .pool
            .try_get()
        {
            Some(c) => c,
            None => return Poll::Pending,
        };

        self.inner.take();

        Poll::Ready(c)
    }
}
