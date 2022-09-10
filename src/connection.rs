use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Get {
    type Output;

    /// Try to receive an active connection.
    ///
    /// Will return `None` if no active connection is available.
    fn try_get(&mut self) -> Option<Self::Output>;
}

/// Used to manage external connections.
#[derive(Clone)]
pub struct Connection<C: Get> {
    inner: C,
}

/// Single use future to fetch new connection from connection manger.
pub struct GetConn<'a, C: Get> {
    inner: Option<&'a mut C>,
}

impl<C: Get> Connection<C> {
    /// Wrap inner connection manager.
    ///
    /// # Parameters
    /// - `inner` Connection manager
    pub fn new(inner: C) -> Self {
        Self { inner }
    }

    /// Returns a future which awaits an active connection.
    pub fn get(&mut self) -> GetConn<'_, C> {
        GetConn {
            inner: Some(&mut self.inner),
        }
    }
}

impl<'a, C: Get> Future for GetConn<'a, C> {
    type Output = C::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let c = if let Some(conn) = self
            .inner
            .as_mut()
            .expect("Unexpected poll after Poll::Ready")
            .try_get()
        {
            conn
        } else {
            // TODO: Find a better solution
            cx.waker().wake_by_ref();
            return Poll::Pending;
        };

        self.inner.take();

        Poll::Ready(c)
    }
}

#[cfg(test)]
mod tests {
    use super::{Connection, Get};

    pub struct ConnectionStub(usize);
    pub struct ConnectionManager(usize);

    impl Get for ConnectionManager {
        type Output = ConnectionStub;

        fn try_get(&mut self) -> Option<Self::Output> {
            if self.0 >= 1 {
                Some(ConnectionStub(self.0))
            } else {
                self.0 += 1;
                None
            }
        }
    }

    #[test]
    fn new() {
        let _ = Connection::new(ConnectionManager(42));
    }

    #[tokio::test]
    async fn get() {
        let mut conn = Connection::new(ConnectionManager(42));
        let c = conn.get().await;
        assert_eq!(c.0, 42);
    }

    #[tokio::test]
    async fn get_suspend() {
        let mut conn = Connection::new(ConnectionManager(0));
        let c = conn.get().await;
        assert_eq!(c.0, 1);
        let c = conn.get().await;
        assert_eq!(c.0, 1);
    }
}
