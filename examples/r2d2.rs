use micro_tower::connection::Connection;
use micro_tower::runtime::Runtime;
use micro_tower::service::Service;
use micro_tower::util::Buildable;
use r2d2::Pool;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

struct ConnectionManagerStub;
struct ConnectionStub(usize);

impl ::r2d2::ManageConnection for ConnectionManagerStub {
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

#[micro_tower::codegen::service]
async fn hello_connection(_: (), mut connection: Connection<Pool<ConnectionManagerStub>>) -> usize {
    let c = connection.get().await;
    c.0
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let conn = Connection::new(Pool::new(ConnectionManagerStub).unwrap());

    let service = Service::<hello_connection>::builder()
        .connection(conn)
        .build()
        .unwrap();

    Runtime::default().bind_service(8000, service).run().await;
}
