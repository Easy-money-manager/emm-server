use serde::{ Serialize, Deserialize };
use crate::record::Record;
//use crate::sheet::Sheet;
use crate::sheetcollection::SheetCollection;
//use axum::http::StatusCode;


//#[derive(Serialize)]
//struct GetRecordsRequest {
//}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct GetRecordsResponse {
    pub records: Vec<Record>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRecordRequest {
    pub description: String,
    pub date: chrono::NaiveDate,
    pub value: i64,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRecordResponse {
    pub id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateRecordRequest {
    pub description: String,
    pub date: chrono::NaiveDate,
    pub value: i64,
}

//#[derive(Serialize, Deserialize)]
//pub struct UpdateRecordResponse {
//}

//#[derive(Serialize)]
//pub struct RemoveRecordRequest {
//}

//#[derive(Serialize, Deserialize)]
//pub struct RemoveRecordResponse {
//}

#[derive(Serialize, Deserialize)]
pub struct BootstrapResponse {
    pub collections: Vec<SheetCollection>,
}
