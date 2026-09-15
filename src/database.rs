use rusqlite::Connection;
use chrono::NaiveDate;
use emm_shared::record::Record;
use emm_shared::sheet::Sheet;
use emm_shared::sheetcollection::SheetCollection;

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
            CREATE TABLE IF NOT EXISTS users (
                id INTEAGER PRIMARY KEY,
                username TEXT NOT NULL,
                password_hash NOT NULL,

                UNIQUE(id, username)
            );

            CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY,
                user_id INTEAGER NOT NULL,
                name TEXT NOT NULL,

                UNIQUE(id, name),

                FOREIGN KEY (user_id)
                    REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS sheets (
                id INTEGER PRIMARY KEY,
                collection_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                fraction INTEGER NOT NULL,

                UNIQUE(collection_id, name),

                FOREIGN KEY (collection_id)
                    REFERENCES collections(id)
            );

            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY,
                sheet_id INTEGER NOT NULL,
                description TEXT NOT NULL,
                date TEXT NOT NULL,
                value INTEGER NOT NULL,

                UNIQUE(id),

                FOREIGN KEY (sheet_id)
                    REFERENCES sheets(id)
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
    pub fn create_user_with_defaults(&self, username: &str, password_hash: &str) -> Result<i64, CreateUserError> {
        let id = self.create_user(username, password_hash)?;
        match self.create_defaults(id) {
            Ok(()) => Ok(id),
            Err(error) => Err(CreateUserError::Database(error)),
        }
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

    pub fn get_records(&self, sheet_id: i64) -> rusqlite::Result<Vec<Record>> {
        let mut statement = self.connection.prepare(
            "SELECT id, description, date, value
            FROM records
            WHERE sheet_id = ?1
            ORDER BY id"
            )?;
        let records = statement.query_map([sheet_id], |row| {
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
    pub fn create_record(&self, sheet_id: i64, description: &str, date: NaiveDate, value: i64) -> rusqlite::Result<i64> {
        self.connection.execute("INSERT INTO records (sheet_id, description, date, value) VALUES (?1, ?2, ?3, ?4)", (sheet_id, description, date.to_string(), value))?;
        Ok(self.connection.last_insert_rowid())
    }
    #[allow(dead_code)]
    pub fn get_record(&self, sheet_id: i64, description: &str, date: NaiveDate, value: i64) -> rusqlite::Result<i64> {
        self.connection.query_row("SELECT id from records WHERE sheet_id = ?1 and description = ?2 AND date = ?3 AND value = ?4", (sheet_id, description, date.to_string(), value), |row| row.get(0))
    }
    pub fn update_record(&self, id: i64, description: &str, date: NaiveDate, value: i64) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE records
            SET description = ?1,
            date = ?2,
            value = ?3
            WHERE id = ?4",
            (description, date.to_string(), value, id)
            )?;

        Ok(())
    }
    pub fn remove_record(&self, id: i64) -> rusqlite::Result<()> {
        self.connection.execute(
            "DELETE FROM records
            WHERE id = ?1",
            [id]
        )?;

        Ok(())
    }
}
