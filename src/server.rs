use axum::{ Router, routing::{ get, put }, Json, extract::{ Path, State }, http::StatusCode };
use serde::Serialize;
use std::sync::{ Arc, Mutex};
use crate::database::Database;
use emm_shared::request::{ CreateRecordRequest, UpdateRecordRequest };
use emm_shared::response::{ GetRecordsResponse, CreateRecordResponse,  BootstrapResponse };
use tower_http::cors::{ Any, CorsLayer };

#[derive(Serialize)]
struct TestResponse {
    message: String,
}

#[derive(Clone)]
pub struct AppState {
    database: Arc<Mutex<Database>>,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        Self {
            database: Arc::new(Mutex::new(database)),
        }
    }
}


pub struct Server {
    state: AppState,
}

impl Server {
    pub fn new(database: Database) -> Self {
        Self {
            state: AppState::new(database),
        }
    }
    async fn test() -> Json<TestResponse> {
        Json(TestResponse {
            message: "Server works".to_string(),
        })
    }

    fn log_error(message: &str) {
        eprintln!("[SERVER ERROR]: {}", message);
    }
//    fn log(message: &str) {
//        eprintln!("[SERVER LOG]: {}", message);
//    }

    async fn get_records(Path(sheet_id): Path<i64>, State(state): State<AppState>) -> Result<Json<GetRecordsResponse>, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        match database.get_records(sheet_id) {
            Ok(records) => Ok(Json(GetRecordsResponse { records } )),
            Err(error)  => {
                Self::log_error(&format!("Failed to get records: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    async fn create_record(Path(sheet_id): Path<i64>, State(state): State<AppState>, Json(input): Json<CreateRecordRequest>) -> Result<Json<CreateRecordResponse>, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let id = match database.create_record(sheet_id, &input.description, input.date, input.value) {
            Ok(id)     => id,
            Err(error) => {
                Self::log_error(&format!("Failed to create record: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        Ok(Json(CreateRecordResponse { id } ))
    }

    async fn update_record(Path(id): Path<i64>, State(state): State<AppState>, Json(input): Json<UpdateRecordRequest>) -> Result<StatusCode, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        match database.update_record(id, &input.description, input.date, input.value) {
            Ok(())     => Ok(StatusCode::NO_CONTENT),
            Err(error) => {
                Self::log_error(&format!("Failed to edit record: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    async fn remove_record(Path(id): Path<i64>, State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        match database.remove_record(id) {
            Ok(())     => Ok(StatusCode::NO_CONTENT),
            Err(error) => {
                Self::log_error(&format!("Failed to remove record: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    async fn get_bootstrap(State(state): State<AppState>) -> Result<Json<BootstrapResponse>, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let mut collections = match database.get_collections() {
            Ok(collections) => collections,
            Err(error)      => {
                Self::log_error(&format!("Failed to get bootstrap collections from database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        for collection in &mut collections {
            collection.sheets = match database.get_sheets(collection.id()) {
                Ok(sheet)  => sheet,
                Err(error) => {
                    Self::log_error(&format!("Failed to get bootstrap sheets from database: {}", error));
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            for sheet in &mut collection.sheets {
                sheet.records = match database.get_records(sheet.id()) {
                    Ok(records) => records,
                    Err(error)  => {
                        Self::log_error(&format!("Failed to get bootstrap records from database: {}", error));
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                };
            }
        }

        Ok(Json(BootstrapResponse { collections } ))
    }

    fn records_router() -> Router<AppState> {
        Router::new().route(
            "/records/sheet/{sheet_id}",
            get(Self::get_records)
            .post(Self::create_record)
        ).route(
        "/records/{id}",
        put(Self::update_record)
        .delete(Self::remove_record)
        )
    }

    fn bootstrap_router() -> Router<AppState> {
        Router::new().route("/bootstrap", get(Self::get_bootstrap))
    }
    fn router(&self) -> Router {
        let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

        Router::new()
            .route("/", get(Self::test))
            .merge(Self::records_router())
            .merge(Self::bootstrap_router())
            .with_state(self.state.clone())
            .layer(cors)
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
    let app = self.router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
}

