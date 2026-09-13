//#[cfg(feature = "server")]
mod server;
//#[cfg(feature = "server")]
mod database;
mod record;
mod sheet;
mod sheetcollection;
mod requests;
mod clienttask;


#[tokio::main]
async fn main() {
    if let Err(error) = server::run().await {
        eprintln!("Server failed to start: {}", error);
    }
}
