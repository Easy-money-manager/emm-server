use crate::sheet::Sheet;
use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize)]
pub struct SheetCollection {
    pub id: i64,
    pub name: String,
    pub sheets: Vec<Sheet>,
    pub active_sheet: usize,
}

/*impl Default for SheetCollection {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            sheets: Vec::new(),
        }
    }
}*/

#[allow(dead_code)]
impl SheetCollection {
    pub fn new(id: i64, name: &str) -> Self {
        Self {
            id : id,
            name: name.to_string(),
            sheets: Vec::new(),
            active_sheet: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.sheets.len()
    }

    pub fn active_sheet_index(&self) -> usize {
        self.active_sheet
    }
    pub fn active_sheet_set(&mut self, active_sheet: usize) {
        self.active_sheet = active_sheet;
    }

    pub fn active_sheet(&self) -> &Sheet {
        let sheet_index = self.active_sheet;

        &self.sheets[sheet_index]
    }
    pub fn active_sheet_mut(&mut self) -> &mut Sheet {
        let sheet_index = self.active_sheet;

        &mut self.sheets[sheet_index]
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn id_set(&mut self, id: i64) {
        self.id = id;
    }

    pub fn create(name: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            sheets: Vec::new(),
            active_sheet: 0,
        }
    }
    pub fn push(&mut self, id: i64, name: &str, fraction: i64) {
        self.sheets.push(Sheet::new(id, name, fraction));
    }
}
