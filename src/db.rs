use tokio_postgres::Client;

pub struct DbClient {
    pub client: Client,
}
