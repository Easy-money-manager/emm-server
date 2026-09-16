mod server;
mod database;
use crate::database::Database;
use crate::server::Server;


#[tokio::main]
async fn main() {
    let database = Database::new("/var/lib/emm/easy_money_manager.db").expect("Failed to initialize database");
    database.initialize().expect("Failed to initialize database");
    let server = Server::new(database);
    if let Err(error) = server.run().await {
        eprintln!("Server failed: {}", error);
    }
}
