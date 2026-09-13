use axum::{ Router, routing::{ get, put }, Json, extract::{ Path, State }, http::StatusCode };
use serde::Serialize;
use std::sync::{ Arc, Mutex};
use crate::database::Database;
use crate::requests::{ GetRecordsResponse, CreateRecordRequest, CreateRecordResponse, UpdateRecordRequest, BootstrapResponse };
use tower_http::cors::{ Any, CorsLayer };

#[derive(Serialize)]
struct TestResponse {
    message: String,
}

#[derive(Clone)]
pub struct AppState {
    database: Arc<Mutex<Database>>,
}


async fn test() -> Json<TestResponse> {
    Json(TestResponse {
        message: "Server works".to_string(),
    })
}

async fn get_records(Path(sheet_id): Path<i64>, State(state): State<AppState>) -> Result<Json<GetRecordsResponse>, StatusCode> {
    let database = match state.database.lock() {
        Ok(database) => database,
        Err(error)   => {
            eprintln!("Failed to lock database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match database.get_records(sheet_id) {
        Ok(records) => Ok(Json(GetRecordsResponse{ records: records })),
        Err(error)  => {
            eprintln!("Failed to get records: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_record(Path(sheet_id): Path<i64>, State(state): State<AppState>, Json(input): Json<CreateRecordRequest>) -> Result<Json<CreateRecordResponse>, StatusCode> {
    let database = match state.database.lock() {
        Ok(database) => database,
        Err(error)   => {
            eprintln!("Failed to lock database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let id = match database.create_record(sheet_id, &input.description, input.date, input.value) {
        Ok(id)     => id,
        Err(error) => {
            eprintln!("Failed to create record: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(CreateRecordResponse { id: id } ))
}

async fn update_record(Path(id): Path<i64>, State(state): State<AppState>, Json(input): Json<UpdateRecordRequest>) -> Result<StatusCode, StatusCode> {
    let database = match state.database.lock() {
        Ok(database) => database,
        Err(error)   => {
            eprintln!("Failed to lock database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match database.update_record(id, &input.description, input.date, input.value) {
        Ok(())     => Ok(StatusCode::NO_CONTENT),
        Err(error) => {
            eprintln!("Failed to edit record: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn remove_record(Path(id): Path<i64>, State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    let database = match state.database.lock() {
        Ok(database) => database,
        Err(error)   => {
            eprintln!("Failed to lock database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match database.remove_record(id) {
        Ok(())     => Ok(StatusCode::NO_CONTENT),
        Err(error) => {
            eprintln!("Failed to remove record: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_bootstrap(State(state): State<AppState>) -> Result<Json<BootstrapResponse>, StatusCode> {
    let database = match state.database.lock() {
        Ok(database) => database,
        Err(error)   => {
            eprintln!("Failed to lock database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut collections = match database.get_collections() {
        Ok(collections) => collections,
        Err(error)      => {
            eprintln!("Failed to get bootstrap collections from database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    for collection in &mut collections {
        collection.sheets = match database.get_sheets(collection.id()) {
            Ok(sheet)  => sheet,
            Err(error) => {
                eprintln!("Failed to get bootstrap sheets from database: {}", error);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        for sheet in &mut collection.sheets {
            sheet.records = match database.get_records(sheet.id()) {
                Ok(records) => records,
                Err(error)  => {
                    eprintln!("Failed to get bootstrap records from database: {}", error);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };
        }
    }

    Ok(Json(BootstrapResponse { collections: collections }))
}

fn records_router() -> Router<AppState> {
    Router::new().route(
        "/records/sheet/{sheet_id}",
        get(get_records)
        .post(create_record)
    ).route(
        "/records/{id}",
        put(update_record)
        .delete(remove_record)
    )
}

fn bootstrap_router() -> Router<AppState> {
    Router::new().route("/bootstrap", get(get_bootstrap))
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new("easy_money_manager.db").expect("Failed to initialize database");
    match database.initialize() {
        Ok(())     => eprintln!("Initialized database"),
        Err(error) => eprintln!("Failed to initialize database: {}", error),
    };

    let state = AppState {
        database: Arc::new(Mutex::new(database)),
    };

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let app = Router::new()
        .route("/", get(test))
        .merge(records_router())
        .merge(bootstrap_router())
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
