use crate::record::Record;
use serde::{ Serialize, Deserialize };

#[derive(Debug)]
pub enum SheetError {
    IndexOutOfBounds,
}

// Sheet
//
// name - name of sheet
// records - list (Vec<Record>) with records to store informations about money flow
// fraction - how much of your incomes you wanna commit to this type of money flow,
// tho save as with Record::value it's i64 and just getting divided by 100 for 
// calculations, so for example if for essential things you wanna spend up to 50%
// of your income it's fraction 50 (50% = 50 (= fraction) / 100)

#[derive(Serialize, Deserialize)]
pub struct Sheet {
    pub id: i64,
    pub name: String,
    pub records: Vec<Record>,
    pub fraction: i64,
}

/*impl Default for Sheet {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            records: Vec::new(),
            fraction: 0,
        }
    }
}*/

#[allow(dead_code)]
impl Sheet {
    pub fn new(id: i64, name: &str, fraction: i64) -> Self {
        Self {
            id: id,
            name: name.to_string(),
            fraction: fraction,
            records: Vec::new(),
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn id_set(&mut self, id: i64) {
        self.id = id;
    }

    pub fn sum(&self) -> i64 {
        let mut sum = 0;
        for record in &self.records {
            sum += record.value;
        }
        sum
    }
    pub fn sum_display(&self) -> String {
        (self.sum() as f64 / 100.0).to_string()
    }
    pub fn balance(&self, incomes: &i64) -> i64 {
        incomes * self.fraction / 100 - self.sum()
    }
    pub fn balance_display(&self, incomes: &i64) -> String {
        (self.balance(incomes) as f64 / 100.0).to_string()
    }

    pub fn push(&mut self, record: Record) {
        self.records.push(record);
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn remove(&mut self, index: usize) -> Result<(), SheetError> {
        if index >= self.len() {
            return Err(SheetError::IndexOutOfBounds);
        }
        self.records.remove(index);
        Ok(())
    }
    pub fn edit(&mut self, index: usize, record: Record) -> Result <(), SheetError> {
        if index >= self.len() {
            return Err(SheetError::IndexOutOfBounds);
        }
        self.records[index] = record;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn push_test() {

    }
}
