use rusqlite::{ Connection, OptionalExtension };
use chrono::{ NaiveDate, Utc };
use emm_shared::record::{ Record, ParsedImportRecord };
use emm_shared::sheet::Sheet;
use emm_shared::sheetcollection::SheetCollection;

#[allow(unused)]
pub enum CreateUserError {
    UsernameTaken,
    Database(rusqlite::Error),
}
impl From<rusqlite::Error> for CreateUserError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

pub struct Database {
    connection: Connection,
}

#[allow(dead_code, unused_assignments)]
impl Database {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;

        Ok(Self { connection } )
    }
    pub fn initialize(&self) -> rusqlite::Result<()> {
        self.connection.execute_batch(
            "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
            token_hash TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            expires_at TEXT NOT NULL,

            FOREIGN KEY (user_id)
                REFERENCES users(id)
                ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS collections (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,

            UNIQUE(user_id, name),

            FOREIGN KEY (user_id)
                REFERENCES users(id)
                ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS sheets (
            id INTEGER PRIMARY KEY,
            collection_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            fraction INTEGER NOT NULL,

            UNIQUE(collection_id, name),

            FOREIGN KEY (collection_id)
                REFERENCES collections(id)
                ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS records (
            id INTEGER PRIMARY KEY,
            sheet_id INTEGER NOT NULL,
            description TEXT NOT NULL,
            date TEXT NOT NULL,
            value INTEGER NOT NULL,

            FOREIGN KEY (sheet_id)
                REFERENCES sheets(id)
                ON DELETE CASCADE
        );
        ",
        )?;

        Ok(())
    }

    pub fn create_defaults(&self, user_id: i64) -> rusqlite::Result<()> {
        let main_id: i64 = self.get_or_create_collection(user_id, "Main")?;
        let _incomes_id = self.get_or_create_sheet(main_id, "Incomes", 100)?;
        let _essentials_id = self.get_or_create_sheet(main_id, "Essentials", 50)?;
        let _stability_id = self.get_or_create_sheet(main_id, "Stability", 15)?;
        let _growth_id = self.get_or_create_sheet(main_id, "Growth", 25)?;
        let _prizes_id = self.get_or_create_sheet(main_id, "Prizes", 10)?;
        let planning_id: i64 = self.get_or_create_collection(user_id, "Planning")?;
        let _incomes_id = self.get_or_create_sheet(planning_id, "Incomes", 100)?;
        let _expenses_id = self.get_or_create_sheet(planning_id, "Expenses", 100)?;
        Ok(())
    }
//    pub fn get_user(&self, name: &str) -> rusqlite::Result<i64> {
//        self.connection.query_row("SELECT id FROM users WHERE name = ?1", [name], |row| row.get(0))
//    }
    pub fn create_user(&self, username: &str, password_hash: &str) -> Result<i64, CreateUserError> {
        match self.connection.execute("INSERT INTO users (username, password_hash) VALUES (?1, ?2)", (username, password_hash)) {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(error, _)) if error.code == rusqlite::ErrorCode::ConstraintViolation => return Err(CreateUserError::UsernameTaken),
            Err(error) => return Err(CreateUserError::Database(error)),
        }

        Ok(self.connection.last_insert_rowid())
    }
    pub fn remove_user(&self, user_id: i64) -> rusqlite::Result<()> {
        let affected_rows: usize = self.connection.execute("DELETE FROM users WHERE id = ?1", [user_id])?;
        if affected_rows == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }
    pub fn create_user_with_defaults(&self, username: &str, password_hash: &str) -> Result<i64, CreateUserError> {
        let id = self.create_user(username, password_hash)?;
        if let Err(error) = self.create_defaults(id) {
            println!("Failed to create defaults for {}: {}", id, error);
        }
        Ok(id)
    }
    pub fn get_session_user(&self, token_hash: &str) -> rusqlite::Result<Option<i64>> {
        let now: String = Utc::now().to_rfc3339();
        self.connection.query_row("SELECT user_id FROM sessions WHERE token_hash = ?1 AND expires_at > ?2", (token_hash, now), |row| row.get(0)).optional()
    }
    pub fn create_session(&self, user_id: i64, token_hash: &str, expires_at: &str) -> rusqlite::Result<()> {
        self.connection.execute("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?1, ?2, ?3)", (token_hash, user_id, expires_at))?;
        Ok(())
    }
    pub fn remove_session(&self, token_hash: &str) -> rusqlite::Result<()> {
        self.connection.execute("DELETE FROM sessions WHERE token_hash = ?1", [token_hash])?;
        Ok(())
    }

    pub fn get_user_for_login(&self, username: &str) -> rusqlite::Result<Option<(i64, String)>> {
        self.connection.query_row("SELECT id, password_hash FROM users WHERE username = ?1", [username], |row| {
            let id: i64 = row.get(0)?;
            let password_hash: String = row.get(1)?;
            Ok((id, password_hash))
        }).optional()
    }

    pub fn get_collections(&self, user_id: i64) -> rusqlite::Result<Vec<SheetCollection>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name
            FROM collections
            WHERE user_id = ?1
            ORDER BY id"
        )?;
        let collections = statement.query_map([user_id], |row| {
            Ok(SheetCollection::new(
                    row.get(0)?,
                    &row.get::<_,String>(1)?,
            ))
        })?;

        collections.collect()
    }
    pub fn create_collection(&self, user_id: i64, name: &str) -> rusqlite::Result<i64> {
        self.connection.execute("INSERT INTO collections (user_id, name) VALUES (?1, ?2)", (user_id, name))?;
        Ok(self.connection.last_insert_rowid())
    }
    pub fn get_collection(&self, user_id: i64, name: &str) -> rusqlite::Result<i64> {
        self.connection.query_row("SELECT id FROM collections WHERE user_id = ?1 AND name = ?2", (user_id, name), |row| row.get(0))
    }
    pub fn get_or_create_collection(&self, user_id: i64, name: &str) -> rusqlite::Result<i64> {
        match self.get_collection(user_id, name) {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.create_collection(user_id, name)
            }
            Err(error) => Err(error),
        }
    }

    pub fn get_sheets(&self, collection_id: i64) -> rusqlite::Result<Vec<Sheet>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, fraction
            FROM sheets
            WHERE collection_id = ?1
            ORDER BY id"
        )?;

        let sheets = statement.query_map([collection_id], |row| {
            Ok(Sheet::new(
                    row.get(0)?,
                    &row.get::<_,String>(1)?,
                    row.get(2)?
            ))})?;

        sheets.collect()
    }
    pub fn create_sheet(&self, collection_id: i64, name: &str, fraction: i64) -> rusqlite::Result<i64> {
        self.connection.execute("INSERT INTO sheets (collection_id, name, fraction) VALUES (?1, ?2, ?3)", (collection_id, name, fraction))?;
        Ok(self.connection.last_insert_rowid())
    }
    pub fn get_sheet(&self, collection_id: i64, name: &str) -> rusqlite::Result<i64> {
        self.connection.query_row("SELECT id FROM sheets WHERE collection_id = ?1 AND name = ?2", (collection_id, name), |row| row.get(0))
    }
    pub fn get_or_create_sheet(&self, collection_id: i64, name: &str, fraction: i64) -> rusqlite::Result<i64> {
        match self.get_sheet(collection_id, name) {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.create_sheet(collection_id, name, fraction)
            }
            Err(error) => Err(error),
        }
    }
    pub fn import_records_to_sheet(&mut self, user_id: i64, sheet_id: i64, input: Vec<ParsedImportRecord>) -> rusqlite::Result<Vec<Record>> {
        let tx = self.connection.transaction()?;

        let owns_sheet: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM sheets JOIN collections ON collections.id = sheets.collection_id WHERE sheets.id = ?1 AND collections.user_id = ?2)", (sheet_id, user_id), |row| row.get(0))?;
        if !owns_sheet {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        let mut imported = Vec::with_capacity(input.len());

        for record in input {
            tx.execute("
                INSERT INTO records
                (sheet_id, description, date, value)
                VALUES
                (?1, ?2, ?3, ?4)
            ",
            (sheet_id, &record.description, record.date.to_string(), record.value)
            )?;
            let id = tx.last_insert_rowid();
            imported.push(Record {
                id: id,
                description: record.description,
                date: record.date,
                value: record.value,
            });
        }
        tx.commit()?;
        Ok(imported)
    }
    pub fn get_records(&self, user_id: i64, sheet_id: i64) -> rusqlite::Result<Vec<Record>> {
        let mut statement = self.connection.prepare(
            "SELECT records.id, records.description, records.date, records.value
            FROM records
            INNER JOIN sheets
                ON records.sheet_id = sheets.id
            INNER JOIN collections
                ON sheets.collection_id = collections.id
            WHERE records.sheet_id = ?1
            AND collections.user_id = ?2
            ORDER BY records.id"
        )?;
        let records = statement.query_map((sheet_id, user_id), |row| {
            let date_string: String = row.get(2)?;

            let date = NaiveDate::parse_from_str(&date_string, "%Y-%m-%d").expect("Failed to parse date from database's string");

            Ok(Record {
                id: row.get(0)?,
                description: row.get(1)?,
                date: date,
                value: row.get(3)?,
            })
        },
        )?;

        let mut result: Vec<Record> = Vec::new();
        for record in records{
            result.push(record?);
        }

        Ok(result)
    }
    pub fn create_record(&self, user_id: i64, sheet_id: i64, description: &str, date: NaiveDate, value: i64) -> rusqlite::Result<i64> {
        let affected_rows: usize = self.connection.execute("
            INSERT INTO records
            (sheet_id, description, date, value)
            SELECT sheets.id, ?2, ?3, ?4 
            FROM sheets
            INNER JOIN collections
                ON sheets.collection_id = collections.id
            WHERE collections.user_id = ?5
            AND sheets.id = ?1
            ", (sheet_id, description, date.to_string(), value, user_id))?;
        if affected_rows == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(self.connection.last_insert_rowid())
    }
    #[allow(dead_code)]
    pub fn get_record(&self, sheet_id: i64, description: &str, date: NaiveDate, value: i64) -> rusqlite::Result<i64> {
        self.connection.query_row("
            SELECT id
            FROM records
            WHERE records.sheet_id = ?1
            AND description = ?2
            AND date = ?3
            AND value = ?4
            ", (sheet_id, description, date.to_string(), value), |row| row.get(0))
    }
    pub fn update_record(&self, user_id: i64, id: i64, description: &str, date: NaiveDate, value: i64) -> rusqlite::Result<()> {
        let affected_rows: usize = self.connection.execute("
            UPDATE records
            SET description = ?1,
            date = ?2,
            value = ?3
            WHERE id = ?4
            AND sheet_id IN (
                SELECT sheets.id
                FROM sheets
                INNER JOIN collections
                    ON sheets.collection_id = collections.id
                WHERE collections.user_id = ?5
            )",
            (description, date.to_string(), value, id, user_id)
        )?;

        if affected_rows == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }
    pub fn remove_record(&self, user_id: i64, id: i64) -> rusqlite::Result<()> {
        let affected_rows: usize = self.connection.execute(
            "DELETE FROM records
            WHERE id = ?1
            AND sheet_id IN (
                SELECT sheets.id
                FROM sheets
                INNER JOIN collections
                    ON sheets.collection_id = collections.id
                WHERE collections.user_id = ?2
            )",
            [id, user_id]
        )?;

        if affected_rows == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }
}
