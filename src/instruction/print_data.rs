use crate::command::{Font};
use std::collections::HashMap;

pub struct PrintData {
    pub(crate) widths: HashMap<Font, u8>,
    pub(crate) replacements: HashMap<String, String>,
    pub(crate) duo_tables: Option<HashMap<String, Vec<(String, String)>>>,
    pub(crate) trio_tables: Option<HashMap<String, Vec<(String, String, String)>>>,
    pub(crate) quad_tables: Option<HashMap<String, Vec<(String, String, String, String)>>>,
    pub(crate) qr_contents: Option<HashMap<String, String>>
}

pub struct PrintDataBuilder {
    widths: HashMap<Font, u8>,
    replacements: HashMap<String, String>,
    duo_tables: Option<HashMap<String, Vec<(String, String)>>>,
    trio_tables: Option<HashMap<String, Vec<(String, String, String)>>>,
    quad_tables: Option<HashMap<String, Vec<(String, String, String, String)>>>,
    qr_contents: Option<HashMap<String, String>>
}

impl PrintDataBuilder {
    pub fn new() -> PrintDataBuilder {
        PrintDataBuilder {
            widths: HashMap::new(),
            replacements: HashMap::new(),
            duo_tables: None,
            trio_tables: None,
            quad_tables: None,
            qr_contents: None
        }
    }

    pub fn set_font_width(mut self, font: Font, width: u8) -> Self {
        self.widths.insert(font, width);
        self
    }

    pub fn add_replacement<A: Into<String>, B: Into<String>>(mut self, target: A, replacement: B) -> Self {
        self.replacements.insert(target.into(), replacement.into());
        self
    }

    pub fn add_duo_table<A: Into<String>>(mut self, name: A, rows: Vec<(String, String)>) -> Self {
        if let Some(duo_tables) = &mut self.duo_tables {
            duo_tables.insert(name.into(), rows);
        } else {
            self.duo_tables = Some(vec![(name.into(), rows)].into_iter().collect());
        }
        self
    }

    pub fn add_trio_table<A: Into<String>>(mut self, name: A, rows: Vec<(String, String, String)>) -> Self {
        if let Some(trio_tables) = &mut self.trio_tables {
            trio_tables.insert(name.into(), rows);
        } else {
            self.trio_tables = Some(vec![(name.into(), rows)].into_iter().collect());
        }
        self
    }

    pub fn add_quad_table<A: Into<String>>(mut self, name: A, rows: Vec<(String, String, String, String)>) -> Self {
        if let Some(quad_tables) = &mut self.quad_tables {
            quad_tables.insert(name.into(), rows);
        } else {
            self.quad_tables = Some(vec![(name.into(), rows)].into_iter().collect());
        }
        self
    }

    pub fn add_qr_code<A: Into<String>, B: Into<String>>(mut self, name: A, content: B) -> Self {
        if let Some(qr_contents) = &mut self.qr_contents {
            qr_contents.insert(name.into(), content.into());
        } else {
            self.qr_contents = Some(vec![(name.into(), content.into())].into_iter().collect());
        }
        self
    }

    pub fn build(self) -> PrintData {
        PrintData {
            widths: self.widths,
            replacements: self.replacements,
            duo_tables: self.duo_tables,
            trio_tables: self.trio_tables,
            quad_tables: self.quad_tables,
            qr_contents: self.qr_contents
        }
    }
}