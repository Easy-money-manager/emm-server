mod server;
mod database;
use crate::database::Database;
use crate::server::Server;


#[tokio::main]
async fn main() {
    let database = Database::new("easy_money_manager.db").expect("Failed to initialize database");
    match database.initialize() {
        Ok(())     => eprintln!("Initialized database"),
        Err(error) => eprintln!("Failed to initialize database: {}", error),
    };
    let server = Server::new(database);
    if let Err(error) = server.run().await {
        eprintln!("Server failed: {}", error);
    }
}
