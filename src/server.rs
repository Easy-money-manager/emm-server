use emm_shared::request::{ RegisterRequest, LoginRequest, CreateRecordRequest, UpdateRecordRequest };
use emm_shared::response::{ LoginResponse, GetRecordsResponse, CreateRecordResponse,  BootstrapResponse };
use axum::{ Router, routing::{ get, put, post, delete }, Json, extract::{ Path, State }, http::{ StatusCode, HeaderMap, header::AUTHORIZATION } };
use serde::Serialize;
use std::sync::{ Arc, Mutex };
use crate::database::{ Database, CreateUserError };
use tower_http::cors::{ Any, CorsLayer };
use sha2::{ Digest, Sha256 };
use argon2::{ Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString };
use rand_core::{ OsRng, RngCore };
use chrono::{ Utc, Duration };
use unicode_normalization::UnicodeNormalization;

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
    fn log(message: &str) {
        eprintln!("[SERVER LOG]: {}", message);
    }

    fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt: SaltString = SaltString::generate(&mut OsRng);
        let password_normalized: String = password.to_string().nfc().collect::<String>();
        let password_hash: String = Argon2::default().hash_password(password_normalized.as_bytes(), &salt)?.to_string();
        Ok(password_hash)
    }
    fn verify_password(password: &str, password_hash: &str) -> bool {
        let parsed_hash: PasswordHash = match PasswordHash::new(password_hash) {
            Ok(hash) => hash,
            Err(_) => return false,
        };
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }
    fn generate_session_token() -> String {
        let mut bytes: [u8; 32] = [0; 32];
        OsRng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
    fn hash_session_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let hash = hasher.finalize();
        hex::encode(hash)
    }
    fn get_bearer_token(headers: &HeaderMap) -> Result<&str, StatusCode> {
        let authorization = headers.get(AUTHORIZATION).ok_or(StatusCode::UNAUTHORIZED)?;
        let authorization = authorization.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
        let token = authorization.strip_prefix("Bearer ").ok_or(StatusCode::UNAUTHORIZED)?;
        Ok(token)
    }
    fn authenticate(headers: &HeaderMap, database: &Database) -> Result<i64, StatusCode> {
        let token: &str = Self::get_bearer_token(headers)?;
        let token_hash: String = Self::hash_session_token(token);
        match database.get_session_user(&token_hash) {
            Ok(Some(user_id)) => Ok(user_id),
            Ok(None) => Err(StatusCode::UNAUTHORIZED),
            Err(error) => {
                Self::log_error(&format!("Failed to authenticate session: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    async fn register(State(state): State<AppState>, Json(input): Json<RegisterRequest>) -> Result<StatusCode, StatusCode> {
        let password_hash: String = match Self::hash_password(&input.password) {
            Ok(password_hash) => password_hash,
            Err(error)        => {
                Self::log_error(&format!("Failed to hash password: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let user_id: i64 = match database.create_user(&input.username, &password_hash) {
            Ok(user_id) => user_id,
            Err(CreateUserError::UsernameTaken) => {
                return Err(StatusCode::CONFLICT);
            }
            Err(CreateUserError::Database(error)) => {
                Self::log_error(&format!("Failed to create user: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        if let Err(error) = database.create_defaults(user_id) {
            Self::log_error(&format!("Failed to create defaults for user {}: {}", user_id, error));
        }
        Self::log(!format!("Created user:\t{}\t{}", user_id, input.username));
        Ok(StatusCode::CREATED)
    }
    async fn login(State(state): State<AppState>, Json(input): Json<LoginRequest>) -> Result<Json<LoginResponse>, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let (user_id, password_hash): (i64, String) = match database.get_user_for_login(&input.username) {
            Ok(Some(user)) => user,
            Ok(None)       => return Err(StatusCode::UNAUTHORIZED),
            Err(error)     => {
                Self::log_error(&format!("Failed to get user for login: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        if !Self::verify_password(&input.password, &password_hash) {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let session_token: String = Self::generate_session_token();
        let token_hash: String = Self::hash_session_token(&session_token);
        let expires_at: String = (Utc::now() + Duration::days(10)).to_rfc3339();
        if let Err(error) = database.create_session(user_id, &token_hash, &expires_at) {
            Self::log_error(&format!("Failed to create session: {}", error));
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        let bootstrap = Self::build_bootstrap(&database, user_id)?;
        Ok(Json(LoginResponse {
            user_id,
            username: input.username,
            session_token,
            bootstrap
        }))
    }
    async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Result<StatusCode, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let _: i64 = Self::authenticate(&headers, &database)?;
        let token: &str = Self::get_bearer_token(&headers)?;
        let token_hash: String = Self::hash_session_token(token);
        match database.remove_session(&token_hash) {
            Ok(())     => Ok(StatusCode::NO_CONTENT),
            Err(error) => {
                Self::log_error(&format!("Failed to remove session: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
    async fn remove_account(State(state): State<AppState>, headers: HeaderMap) -> Result<StatusCode, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let user_id: i64 = Self::authenticate(&headers, &database)?;
        match database.remove_user(user_id) {
            Ok(())     => {
                Self::log(&format!("Successfully removed user:\t{}", user_id));
                Ok(StatusCode::NO_CONTENT)
            }
            Err(error) => {
                Self::log_error(&format!("Failed to remove account: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    async fn get_records(Path(sheet_id): Path<i64>, State(state): State<AppState>, headers: HeaderMap) -> Result<Json<GetRecordsResponse>, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let user_id: i64 = Self::authenticate(&headers, &database)?;

        match database.get_records(user_id, sheet_id) {
            Ok(records) => Ok(Json(GetRecordsResponse { records } )),
            Err(error)  => {
                Self::log_error(&format!("Failed to get records: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    async fn create_record(Path(sheet_id): Path<i64>, State(state): State<AppState>, headers: HeaderMap, Json(input): Json<CreateRecordRequest>) -> Result<Json<CreateRecordResponse>, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let user_id: i64 = Self::authenticate(&headers, &database)?;

        match database.create_record(user_id, sheet_id, &input.description, input.date, input.value) {
            Ok(id)     => Ok(Json(CreateRecordResponse { id } )),
            Err(error) => {
                Self::log_error(&format!("Failed to create record: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    async fn update_record(Path(id): Path<i64>, State(state): State<AppState>, headers: HeaderMap, Json(input): Json<UpdateRecordRequest>) -> Result<StatusCode, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let user_id: i64 = Self::authenticate(&headers, &database)?;

        match database.update_record(user_id, id, &input.description, input.date, input.value) {
            Ok(())     => Ok(StatusCode::NO_CONTENT),
            Err(error) => {
                Self::log_error(&format!("Failed to edit record: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    async fn remove_record(Path(id): Path<i64>, State(state): State<AppState>, headers: HeaderMap) -> Result<StatusCode, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let user_id: i64 = Self::authenticate(&headers, &database)?;

        match database.remove_record(user_id, id) {
            Ok(())     => Ok(StatusCode::NO_CONTENT),
            Err(error) => {
                Self::log_error(&format!("Failed to remove record: {}", error));
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    fn build_bootstrap(database: &Database, user_id: i64) -> Result<BootstrapResponse, StatusCode> {
        let mut collections = match database.get_collections(user_id) {
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
                sheet.records = match database.get_records(user_id, sheet.id()) {
                    Ok(records) => records,
                    Err(error)  => {
                        Self::log_error(&format!("Failed to get bootstrap records from database: {}", error));
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                };
            }
        }
        Ok(BootstrapResponse { collections } )
    }
    async fn get_bootstrap(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<BootstrapResponse>, StatusCode> {
        let database = match state.database.lock() {
            Ok(database) => database,
            Err(error)   => {
                Self::log_error(&format!("Failed to lock database: {}", error));
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let user_id: i64 = Self::authenticate(&headers, &database)?;
        let bootstrap = Self::build_bootstrap(&database, user_id)?;
        Ok(Json(bootstrap))
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
    fn auth_router() -> Router<AppState> {
        Router::new()
            .route("/auth/register", post(Self::register))
            .route("/auth/login", post(Self::login))
            .route("/auth/logout", post(Self::logout))
            .route("/adios", delete(Self::remove_account))
    }
    fn router(&self) -> Router {
        let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

        Router::new()
            .route("/", get(Self::test))
            .merge(Self::records_router())
            .merge(Self::bootstrap_router())
            .merge(Self::auth_router())
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

