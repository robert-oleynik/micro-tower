use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait GetConnection {
    type Output;

    /// Try to receive an active connection.
    ///
    /// Will return `None` if no active connection is available.
    fn try_get(&mut self) -> Option<Self::Output>;
}

/// Used to manage external connections.
#[derive(Clone)]
pub struct Connection<C: GetConnection> {
    inner: C,
}

/// Single use future to fetch new connection from connection manger.
pub struct GetConn<'a, C: GetConnection> {
    inner: Option<&'a mut C>,
}

impl<C: GetConnection> Connection<C> {
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

impl<'a, C: GetConnection> Future for GetConn<'a, C> {
    type Output = C::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let c = match self
            .inner
            .as_mut()
            .expect("Unexpected poll after Poll::Ready")
            .try_get()
        {
            Some(conn) => conn,
            None => {
                // TODO: Find a better solution
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };

        self.inner.take();

        Poll::Ready(c)
    }
}

#[cfg(test)]
mod tests {
    use super::{Connection, GetConnection};

    pub struct ConnectionStub(usize);
    pub struct ConnectionManager(usize);

    impl GetConnection for ConnectionManager {
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
