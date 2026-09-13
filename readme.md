# Consistency is key...
... and small improvements do suck but also do work

Todo:
 - [x] compiles
 - [x] runs
 - [x] unit tests passed
 - [x] logic tests passed
 - [ ] grip strenght tests passed
 - [ ] mental stability tests passed

## License

This project is licensed under the GNU Affero General Public License v3.0.
See the `LICENSE` file for details.

Alternative commercial licensing may be available separately.







# Docs

Server
|
|
Api
|
|------|
|     \|
Client WebClient


## record.rs
### Record -> struct
#### Vars
##### name
|> String 
Name of flow
##### date
|> chrono::NaiveDate 
Date of flow
##### value
|> i64 
Value of flow
##### id
|> i64 
database id, NOT INDEX
#### Methods struct
##### clone
\(\)
-> Self
From Clone, makes a clone of record
##### default
\(\)
-> Self
Default zeroed record
##### parse_value
\(value_zl: String, value_gr: String\)
-> Result<i64, ValueError>
Parser to values from strings, for any invalidities returns corresponding error
##### fn from_input
\(description: &str, year: &i32, month: &u32, day: &u32, value_zl: &str, value_gr: &str\)
-> Result<Self, RecordError>
Effectively parser to Record from series of strings \(just dates are numbers since we get those as numerical values from client's input\)
Has best day validity checker :-)
##### id
\(&self\)
-> i64
Returns copy of its id
##### id_set
\(&mut self, id: i64\)
-> \(\)
Sets its id to given
##### description
\(&self\)
-> &str
Returns copy of its description as string slice
##### description_set
\(&mut self, description: String\)
-> \(\_
Sets its description to given
##### value
\(&self\)
-> i64
Returns copy of its value
##### value_display
\(&self\)
-> String
Returns its value as string parsed for being displayed \(with decimal value\)
##### value_set
\(&mut self, value_zl: &String, value_gr: &String\)
-> \(\)
Sets value to value parsed from given, on fail DOESN'T RETURN ERROR
??? Is it even needed? Never used, not rly usable... it's not even using parser higher
##### date
\(&self\)
-> chrono::NaiveDate
##### date_display
\(&self\)
-> String
##### date_set
\(&mut self, year: i32, month: u32, day: u32\)
-> \(\)
Sets date to given, on fail DOESN'T RETURNS ERROR \(assumption is that getting values from gui we are past value sanitization\) but prints into std_err
### RecordError -> enum
#### Values
Quite self-explainatory
##### EmptyDescription
##### InvalidYear
##### InvalidMonth
##### InvalidDay
##### ValueError\(ValueError\)
### ValueError -> enum
Quite self-explainatory
#### Values
##### InvalidValueZl
##### InvalidValueGr
##### TooBigGr
### Some unit tests
## sheet.rs
### Sheet
#### Vars
##### name
String
Name of sheet
##### records
Vec<Record>
Sheet's contents
##### fraction
i64
Fraction for calculations \(made into i64 to not loose calulative quality\)
##### id
i64
Database id, NOT INDEX
#### Methods
##### new
##### id
##### id_set
##### sum
##### sum_display
##### balance
##### balance_display
##### push
##### is_empty
##### len
##### remove
##### edit
\(&mut self, index: usize, record: Record\)
-> Result<(), SheetError>
Edits record of given index to have given contents
### SheetError
#### IndexOutOfBounds
Quite self-explainatory
## sheetcollection.rs
### SheetCollection
#### Vars

#### Methods

## database.rs
### Database
#### Vars
##### connection
|> rusqlite::Connection
Streamline for sqlite queries
#### Methods
##### new
\(path: &str\)
-> rusqlite::Result<Self>
Creates new database automatically initializing connection to it \(searches path\)
##### initialize
\(\)
-> rusqlite::Recult<\(\)>
Initializes database with tables we want
Returns Ok\(\(\)\) only if database initialized with all tables correctly
##### get_collections
\(&self\)
-> rusqlite::Result<Vec<SheetCollection>>
Returns everything nicely ordered essentially or error on fail
Use that on bootstrap
##### get_collection
\(&self, name: &str\)
-> rusqlite::Result<i64>
Returns id of collection with given name or error on fail
##### create_collection
\(&self, name: &str\)
-> rusqlite::Result<i64>
Creates collection with given name
Returns its id or error on fail
##### get_or_create_collection
\(&self, name: &str\)
-> rusqlite::Result<i64>
Checks if collection of given name exists and creates it if it doesn't
Returns its id or error on fail
##### get_sheets
\(&self, collection_id: i64\)
-> rusqlite::Result<Vec<Sheet>>
Returns sheet with its contents or error on fail
It's used in get_collections for bootstraping
##### create_sheet
\(&self, collection_id: i64, name: &str, fraction: i64\)
-> rusqlite::Result<i64>
Creates sheet with given attributes
Returns its id or error on fail
##### get_sheet
\(&self, collection_id: i64, name: &str\)
-> rusqlite::Result<i64>
Returns id of sheet with given attributes or error on fail
##### get_or_create_sheet
\(&self, collection_id: i64, name: &str, fraction: i64\)
-> rusqlite::Result<i64>
Checks if sheet of given attributes exists and creates it if it doesn't
Returns its id or error on fail
##### get_records
\(&self, sheet_id: i64\)
-> rusqlite::Result<Vec<Record>>
Returns specific sheet's contents or error on fail
##### create_redord
\(&self, sheet_id: i64, description: &str, date: chrono::NaiveDate, value: i64\)
-> rusqlite::Result<i64>
Creates record with given attributes
Returns its id or error on fail
##### get_record
\(&self, sheet_id: i64, description: &str, date: chrono::NaiveDate, value: i64\)
-> rusqlite::Result<i64>
Returns id of record with given attributes or error on fail
##### update_record
\(&self, id: i64, description: &str, date: chrono::NaiveDate, value: i64\)
-> rusqlite::Result<()>
Edits record of given id to have given contents
##### remove_record
\(&self, id: i64\)
-> rusqlite::Result<()>
Removes record of given id
Returns error on fail
