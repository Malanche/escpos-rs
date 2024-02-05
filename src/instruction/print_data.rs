use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Contains custom information for each print
///
/// Some instructions require custom information in order to get printed. The [PrintData](self::PrintData) structure contains such custom information. The builder pattern is used to construct this structure, see [PrintDataBuilder](self::PrintDataBuilder).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PrintData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) replacements: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duo_tables: Option<HashMap<String, Vec<(String, String)>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) trio_tables: Option<HashMap<String, Vec<(String, String, String)>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) quad_tables: Option<HashMap<String, Vec<(String, String, String, String)>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) qr_contents: Option<HashMap<String, String>>
}

impl PrintData {
    /// Constructs a new print data builder
    pub fn builder() -> PrintDataBuilder {
        PrintDataBuilder::new()
    }

    /// Merges two instances of print data, where the caller (left hand side, or main instance) overrides the data of the callee (right hand side) in the case of a label collision.
    pub fn merge(self, rhs: PrintData) -> PrintData {
        let replacements: HashMap<_,_> = self.replacements.unwrap_or_else(|| HashMap::new()).into_iter().chain(rhs.replacements.unwrap_or_else(|| HashMap::new())).collect();
        let duo_tables: HashMap<_,_> = self.duo_tables.unwrap_or_else(|| HashMap::new()).into_iter().chain(rhs.duo_tables.unwrap_or_else(|| HashMap::new())).collect();
        let trio_tables: HashMap<_,_> = self.trio_tables.unwrap_or_else(|| HashMap::new()).into_iter().chain(rhs.trio_tables.unwrap_or_else(|| HashMap::new())).collect();
        let quad_tables: HashMap<_,_> = self.quad_tables.unwrap_or_else(|| HashMap::new()).into_iter().chain(rhs.quad_tables.unwrap_or_else(|| HashMap::new())).collect();
        let qr_contents: HashMap<_,_> = self.qr_contents.unwrap_or_else(|| HashMap::new()).into_iter().chain(rhs.qr_contents.unwrap_or_else(|| HashMap::new())).collect();

        PrintData {
            replacements: if replacements.is_empty() {None} else {Some(replacements)},
            duo_tables: if duo_tables.is_empty() {None} else {Some(duo_tables)},
            trio_tables: if trio_tables.is_empty() {None} else {Some(trio_tables)},
            quad_tables: if quad_tables.is_empty() {None} else {Some(quad_tables)},
            qr_contents: if qr_contents.is_empty() {None} else {Some(qr_contents)}
        }
    }
}

/// Helps build a valid [PrintData](self::PrintData)
pub struct PrintDataBuilder {
    replacements: Option<HashMap<String, String>>,
    duo_tables: Option<HashMap<String, Vec<(String, String)>>>,
    trio_tables: Option<HashMap<String, Vec<(String, String, String)>>>,
    quad_tables: Option<HashMap<String, Vec<(String, String, String, String)>>>,
    qr_contents: Option<HashMap<String, String>>
}

impl Default for PrintDataBuilder {
    fn default() -> Self {
        PrintDataBuilder {
            replacements: None,
            duo_tables: None,
            trio_tables: None,
            quad_tables: None,
            qr_contents: None
        }
    }
}

impl PrintDataBuilder {
    /// Creates a new print data builder
    pub fn new() -> PrintDataBuilder {
        PrintDataBuilder::default()
    }

    /// Adds a replacement string
    ///
    /// Replacement strings are a simple pattern matching replacement, where all matching instances of `target` get replaces by `replacement`
    ///
    /// ```rust
    /// # use escpos_rs::PrintDataBuilder;
    /// let print_data = PrintDataBuilder::new()
    ///     // Instances of "%name%" will get replaced with "Carlos"
    ///     .replacement("%name%", "Carlos")
    ///     .build();
    /// ```
    ///
    /// Note that there is no particular syntax for the `target` string. `"%name%"` is used in the example so that the word "name" (in case it appears in the text) is safe from this instruction.
    pub fn replacement<A: Into<String>, B: Into<String>>(mut self, target: A, replacement: B) -> Self {
        if let Some(replacements) = &mut self.replacements {
            replacements.insert(target.into(), replacement.into());
        } else {
            self.replacements = Some(vec![(target.into(), replacement.into())].into_iter().collect());
        }
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
            replacements: self.replacements,
            duo_tables: self.duo_tables,
            trio_tables: self.trio_tables,
            quad_tables: self.quad_tables,
            qr_contents: self.qr_contents
        }
    }
}