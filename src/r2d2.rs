use r2d2::{ManageConnection, PooledConnection};

use crate::connection::GetConnection;

impl<C: ManageConnection> GetConnection for r2d2::Pool<C> {
    type Output = PooledConnection<C>;

    fn try_get(&mut self) -> Option<Self::Output> {
        r2d2::Pool::try_get(self)
    }
}

#[cfg(test)]
mod tests {
    use r2d2::Pool;

    use crate::connection::Connection;

    struct ConnectionManagerStub;
    struct ConnectionStub(usize);

    impl r2d2::ManageConnection for ConnectionManagerStub {
        type Connection = ConnectionStub;
        type Error = std::convert::Infallible;

        fn connect(&self) -> Result<Self::Connection, Self::Error> {
            Ok(ConnectionStub(42))
        }

        fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
            assert_eq!(conn.0, 42);
            Ok(())
        }

        fn has_broken(&self, _: &mut Self::Connection) -> bool {
            false
        }
    }

    #[test]
    fn new() {
        let _ = Connection::new(Pool::new(ConnectionManagerStub).unwrap());
    }

    #[tokio::test]
    async fn get() {
        let mut conn = Connection::new(Pool::new(ConnectionManagerStub).unwrap());
        let c = conn.get().await;
        assert_eq!(c.0, 42);
    }
}
