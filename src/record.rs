use chrono::{ NaiveDate, Local };
use serde::{ Serialize, Deserialize };

#[derive(Debug)]
pub enum RecordError {
    EmptyDescription,
    InvalidYear,
    InvalidMonth,
    InvalidDay,
    ValueError(ValueError),
}

#[derive(Debug)]
pub enum ValueError {
    InvalidValueZl,
    InvalidValueGr,
    TooBigGr,
}
impl From<ValueError> for RecordError {
    fn from(error: ValueError) -> Self {
        RecordError::ValueError(error)
    }
}

impl RecordError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::EmptyDescription =>
                "Description can't be empty",

            Self::InvalidYear =>
                "Year has to be an integer between 1900 and 2200",

            Self::InvalidMonth =>
                "Month has to be an integer between 1 and 12",

            Self::InvalidDay =>
                "Invalid day for the selected month and year",

            Self::ValueError(ValueError::InvalidValueZl) =>
                "There's an unwanted sign in the value field",

            Self::ValueError(ValueError::InvalidValueGr) =>
                "There's an unwanted sign in the decimal value field",

            Self::ValueError(ValueError::TooBigGr) =>
                "Decimal value must be between 0 and 99",
        }
    }
}

// Record
//
// description - description of specific money flow
// date - date of that money flow
// value - value of that flow; i64 because computers have problem with calculating 
// decimal fractions so to display it's just gonna be display value / 100

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
    pub description: String,
    pub date: chrono::NaiveDate,
    pub value: i64,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            id: 0,
            description: String::new(),
            date: Local::now().date_naive(),
            value: 0,
        }
    }
}

#[allow(dead_code)]
impl Record {
    pub fn parse_value(value_zl: &str, value_gr: &str) -> Result<i64, ValueError> {
        let zl = if value_zl.is_empty() { 0 } else {
            match value_zl.parse::<i64>() {
                Ok(zl) => zl,
                Err(_error) => {
                    println!("Failed to parse zl from input, {}", RecordError::ValueError(ValueError::InvalidValueZl).message());
                    return Err(ValueError::InvalidValueZl);
                }
            }
        };
        let gr = if value_gr.is_empty() { 0 } else {
            match value_gr.parse::<i64>() {
                Ok(gr) => gr,
                Err(_error) => {
                    println!("Failed to parse gr from input, {}", RecordError::ValueError(ValueError::InvalidValueGr).message());
                    return Err(ValueError::InvalidValueGr);
                }
            }
        };
        if gr.abs() >= 100 { return Err(ValueError::TooBigGr); }
        Ok(zl * 100 + gr)
    }
    #[allow(unused_assignments)]
    pub fn from_input(description: &str, year: &i32, month: &u32, day: &u32, value_zl: &str, value_gr: &str) -> Result<Self, RecordError> {
        if description.trim().is_empty() {
            return Err(RecordError::EmptyDescription);
        }
        if !(1900..=2200).contains(year) {
            return Err(RecordError::InvalidYear);
        }
        if !(1..=12).contains(month) {
            return Err(RecordError::InvalidMonth);
        }
        if *day < 1 ||
            *day > 30 + ((*month % 2 != 0) ^ (*month > 7)) as u32 ||
                2 == *month && *day > 28 + (*year % 4 == 0 && *year % 100 != 0 || *year % 400 == 0) as u32 {
            return Err(RecordError::InvalidDay);
        }
        let mut date: NaiveDate = NaiveDate::default();
        match NaiveDate::from_ymd_opt(*year, *month, *day) {
            Some(date_from_parse) => date = date_from_parse,
            None => {
                println!("Failed to parse input into date");
                return Err(RecordError::InvalidYear);
            }
        }
        let mut value: i64 = 0;
        match Self::parse_value(value_zl, value_gr) {
            Ok(value_from_parse) => value = value_from_parse,
            Err(_error) => {
                println!("Failed to parse input into value, {}", RecordError::ValueError(ValueError::InvalidValueZl).message());
                return Err(RecordError::ValueError(ValueError::InvalidValueZl));
            }
        }
        Ok(Self {
            id: 0,
            description: description.to_string(),
            date: date,
            value: value,
        })
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn id_set(&mut self,id: i64) {
        self.id = id;
    }

    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn description_set(&mut self, description: &str) {
        self.description = description.to_string();
    }

    pub fn value(&self) -> i64 {
        self.value
    }
    pub fn value_display(&self) -> String {
        (self.value as f64 / 100.0).to_string()
    }
    pub fn value_set(&mut self, value_zl: &String, value_gr: &String) {
        self.value = match value_zl.is_empty() {
            false => (match value_zl.parse::<i64>() {
                Ok(value) => value,
                Err(_)    => 0,
            }) * 100,
            true  => 0,
        } + match value_gr.is_empty() {
            false => match value_gr.parse::<i64>() {
                Ok(value) => value,
                Err(_)    => 0,
            },
            true  => 0,
        };
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }
    pub fn date_display(&self) -> String {
        self.date.format("%d.%m.%Y").to_string()
    }
    pub fn date_set(&mut self, year: &i32, month: &u32, day: &u32) {
        match NaiveDate::from_ymd_opt(*year, *month, *day) {
            Some(date) => self.date = date,
            None => eprintln!("Failed to parse input into date"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_empty_description() {
        let result = Record::from_input(
            "",
            &2026,
            &9,
            &6,
            "100",
            "50",
        );
        assert!(matches!(result, Err(RecordError::EmptyDescription)));
    }
    #[test]
    fn reject_too_big_year() {
        let result = Record::from_input(
            "Nazwa",
            &2226,
            &9,
            &6,
            "100",
            "50",
        );
        assert!(matches!(result, Err(RecordError::InvalidYear)));
    }
    #[test]
    fn reject_too_small_year() {
        let result = Record::from_input(
            "Nazwa",
            &1826,
            &9,
            &6,
            "100",
            "50",
        );
        assert!(matches!(result, Err(RecordError::InvalidYear)));
    }
    #[test]
    fn reject_too_big_month() {
        let result = Record::from_input(
            "Nazwa",
            &2026,
            &19,
            &6,
            "100",
            "50",
        );
        assert!(matches!(result, Err(RecordError::InvalidMonth)));
    }
    #[test]
    fn reject_too_small_month() {
        let result = Record::from_input(
            "Nazwa",
            &2026,
            &0,
            &6,
            "100",
            "50",
        );
        assert!(matches!(result, Err(RecordError::InvalidMonth)));
    }
    #[test]
    fn reject_too_big_day() {
        let result = Record::from_input(
            "Nazwa",
            &2026,
            &9,
            &36,
            "100",
            "50",
        );
        assert!(matches!(result, Err(RecordError::InvalidDay)));
    }
    #[test]
    fn reject_too_small_day() {
        let result = Record::from_input(
            "Nazwa",
            &2026,
            &9,
            &0,
            "100",
            "50",
        );
        assert!(matches!(result, Err(RecordError::InvalidDay)));
    }
    #[test]
    fn reject_bad_days() {
        for year in 1900..=2200 {
            for month in 1..=12{
                for day in 1..=31 {
                    let result = Record::from_input(
                        "Nazwa",
                        &year,
                        &month,
                        &day,
                        "100",
                        "50",
                    );
                    match month {
                        1 => assert!(matches!(result, Ok(_record))),
                        2 => if day <= 28 || day <= 29 && ((year % 4 == 0 && year % 100 != 0) || year % 400 == 0) { assert!(matches!(result, Ok(_record))); } else { assert!(matches!(result, Err(RecordError::InvalidDay))); },
                        3 => assert!(matches!(result, Ok(_record))),
                        4 => if day <= 30 { assert!(matches!(result, Ok(_record))) } else { assert!(matches!(result, Err(RecordError::InvalidDay))); },
                        5 => assert!(matches!(result, Ok(_record))),
                        6 => if day <= 30 { assert!(matches!(result, Ok(_record))) } else { assert!(matches!(result, Err(RecordError::InvalidDay))); },
                        7 => assert!(matches!(result, Ok(_record))),
                        8 => assert!(matches!(result, Ok(_record))),
                        9 => if day <= 30 { assert!(matches!(result, Ok(_record))) } else { assert!(matches!(result, Err(RecordError::InvalidDay))); },
                        10 => assert!(matches!(result, Ok(_record))),
                        11 => if day <= 30 { assert!(matches!(result, Ok(_record))) } else { assert!(matches!(result, Err(RecordError::InvalidDay))); },
                        12 => assert!(matches!(result, Ok(_record))),
                        _  => panic!("Shouldn't have reached this number anyway"),
                    }
                }
            }
        }
    }
    #[test]
    fn reject_invalid_value_zl() {
        let result = Record::from_input(
            "Nazwa",
            &2026,
            &9,
            &6,
            "xyz",
            "50",
        );
        assert!(matches!(result, Err(RecordError::ValueError(ValueError::InvalidValueZl))));
    }
    #[test]
    fn reject_invlaid_value_gr() {
        let result = Record::from_input(
            "Nazwa",
            &2026,
            &9,
            &6,
            "100",
            "xyz",
        );
        assert!(matches!(result, Err(RecordError::ValueError(ValueError::InvalidValueGr))));
    }
    #[test]
    fn reject_too_big_value_gr() {
        let result = Record::from_input(
            "Nazwa",
            &2026,
            &9,
            &6,
            "100",
            "100",
        );
        assert!(matches!(result, Err(RecordError::ValueError(ValueError::TooBigGr))));
    }
    #[test]
    fn valid_input() {
        let result = Record::from_input(
            "Nazwa",
            &2026,
            &9,
            &6,
            "100",
            "50",
        );
        assert!(matches!(result, Ok(_record)));
    }
}
