/// Options to print tables
#[derive(Clone, Debug)]
pub struct TableOptions {
    /// Indicates the header/row division character
    pub header_division_pattern: Option<String>,
    /// Inicates if a pattern should be used to bridge between columns
    pub join_columns_pattern: Option<String>
}

/// Helper structure to format text
///
/// The Formatter structure helps create some simple shapes through text, like tables and just formatted text. By default, for tables a header division pattern of `-` and no pattern for bridging columns will be used. This can be modified by using either [set_table_options](Formatter::set_table_options) or by [modify_table_options](Formatter::modify_table_options).
pub struct Formatter {
    /// Inner table options
    table_options: TableOptions,
    /// Width to use for formatting
    width: u8
}

impl Formatter {
    /// Creates a new formatter with a default width
    pub fn new(width: u8) -> Formatter {
        Formatter{
            table_options: TableOptions {
                header_division_pattern: Some("-".into()),
                join_columns_pattern: None
            },
            width
        }
    }

    /// Sets a new set of table options
    ///
    /// To modify just one parameter in a simpler way, check the [modify_table_options](self::Formatter::modify_table_options) method.
    ///
    /// ```rust
    /// # use escpos_rs::{Formatter, TableOptions};
    /// let mut formatter = Formatter::new(20);
    /// formatter.set_table_options(TableOptions {
    ///     header_division_pattern: Some(".-".into()),
    ///     join_columns_pattern: Some(".".into())
    /// });
    /// ```
    pub fn set_table_options(&mut self, table_options: TableOptions) {
        self.table_options = table_options
    }

    /// Gives back a reference to the active table options
    ///
    /// ```rust
    /// # use escpos_rs::Formatter;
    /// let mut formatter = Formatter::new(20);
    /// assert_eq!(Some("-".to_string()), formatter.get_table_options().header_division_pattern);
    /// ```
    pub fn get_table_options(&self) -> &TableOptions {
        &self.table_options
    }

    /// Modify the table options through a callback
    ///
    /// Allows table options modification with a function or closure. Sometimes it may come in hand.
    ///
    /// ```rust
    /// # use escpos_rs::Formatter;
    /// let mut formatter = Formatter::new(20);
    /// formatter.modify_table_options(|table_options| {
    ///     table_options.header_division_pattern = Some("=".to_string());
    /// });
    /// assert_eq!(Some("=".to_string()), formatter.get_table_options().header_division_pattern);
    /// ```
    pub fn modify_table_options<F: Fn(&mut TableOptions)>(&mut self, modifier: F) {
        modifier(&mut self.table_options);
    }

    /// Splits a string by whitespaces, according to the given width
    ///
    /// Notice that the final line will not contain a new line at the end.
    ///
    /// ```rust
    /// use escpos_rs::Formatter;
    /// 
    /// let formatter = Formatter::new(16);
    /// let res = formatter.space_split("Sentence with two lines.");
    /// assert_eq!("Sentence with\ntwo lines.", res.as_str());
    /// ```
    pub fn space_split<A: AsRef<str>>(&self, source: A) -> String {
        let mut result = source.as_ref().split("\n").map(|line| {
            // Now, for each line, we split it into words.
            let mut current_line = String::new();
            let mut broken_lines = Vec::new();
            for word in line.split_whitespace() {
                let num_chars = word.chars().count();
                // The one being added marks the space
                if current_line.len() + num_chars + 1 < self.width.into() {
                    // Easy to add to the current line, the conditional if is for the first word of them all.
                    current_line += &format!("{}{}", if current_line.len() == 0 {""} else {" "}, word);
                } else {
                    // We have to terminate the current line, in case it contains something
                    if !current_line.is_empty() {
                        broken_lines.push(current_line.clone());
                    }
                    if num_chars < self.width.into() {
                        // We start the next line with the current word
                        current_line = word.to_string();
                    } else {
                        // We use a char iterator to split this into lines
                        let mut chars = word.chars();
                        let mut word_fragment: String = chars.by_ref().take(self.width.into()).collect();
                        broken_lines.push(format!("{}",word_fragment));
                        while !word_fragment.is_empty() {
                            word_fragment = chars.by_ref().take(self.width.into()).collect();
                            broken_lines.push(format!("{}",word_fragment));
                        }
                    }
                }
            }
            if !current_line.is_empty() {
                broken_lines.push(current_line);
            }
            broken_lines.join("\n")
        }).collect::<Vec<_>>().join("");
        // If the last character is a new line, we need to add it back in
        if let Some(last_char) = source.as_ref().chars().last() {
            if last_char == '\n' {
                result += "\n";
            }
        }
        result
    }

    /// Creates a table with two columns
    ///
    /// In case the headers do not fit with at least one space between, priority will be given to the second header, and the last remaining character from the first header will be replaced by a dot. If the second header would need to be shortened to less than 3 characters, then the first header will now also be truncated, with the same dot replacing the last charcater from the remaining part of the first header.
    ///
    /// ```rust
    /// # use escpos_rs::Formatter;
    /// let formatter = Formatter::new(20);
    /// let header = ("Product", "Price");
    /// let rows = vec![
    ///     ("Milk", "5.00"),
    ///     ("Cereal", "10.00")
    /// ];
    ///
    /// // We use trim_start just to show the table nicer in this example.
    /// let target = r#"
    /// Product        Price
    /// --------------------
    /// Milk            5.00
    /// Cereal         10.00
    /// "#.trim_start();
    /// 
    /// assert_eq!(target, formatter.duo_table(header, rows));
    /// ```
    pub fn duo_table<A: Into<String>, B: Into<String>, C: IntoIterator<Item = (D, E)>, D: Into<String>, E: Into<String>>(&self, header: (A, B), rows: C) -> String {
        // Aux closure to create each row.
        let aux_duo_table = |mut first: String, mut second: String, width: u8, replace_last: Option<char>| -> String {
            let row_width = first.len() + second.len();
            let (column_1, column_2) = if row_width < width as usize {
                (first, second)
            } else {
                // If the second column requires all the space, we give it
                if second.len() + 4 > (width as usize) {
                    if let Some(replacement) = replace_last {
                        second.truncate((width as usize) - 5);
                        second += &replacement.to_string();
                    } else {
                        second.truncate((width as usize) - 4);
                    }
                }

                // We calculate the remaining space for the second word now.
                let remaining = (width as usize) - second.len();
                // We just need to shorten the second word. We need to include the separating whitespace
                if first.len() > remaining {
                    if let Some(replacement) = replace_last {
                        first.truncate(remaining - 2);
                        first += &replacement.to_string();
                    } else {
                        first.truncate(remaining - 1);
                    }
                }

                (first, second)
            };

            format!("{} {:>2$}\n",
                column_1,
                column_2,
                (width as usize) - (column_1.len() + 1)
            )
        };

        let mut content = aux_duo_table(header.0.into(), header.1.into(), self.width, Some('.'));

        if let Some(hdp) = self.print_header_division_pattern() {
            content += &hdp;
        }

        for row in rows {
            let (first, second) = (row.0.into(), row.1.into());
            content += &aux_duo_table(first, second, self.width, None);
        }
        content
    }

    /// Creates a table with three columns
    ///
    /// In case the headers do not fit with at least one space between, priority will be given to the first header, and the last remaining character from the second header will be replaced by a dot. If the second header would need to be shortened to less than 3 characters, then the first header will now also be truncated, with the same dot replacing the last charcater from the remaining part of the first header.
    ///
    /// ```rust
    /// # use escpos_rs::Formatter;
    /// let formatter = Formatter::new(20);
    /// let header = ("Product", "Price", "Qty.");
    /// let rows = vec![
    ///     ("Milk", "5.00", "3"),
    ///     ("Cereal", "10.00", "1")
    /// ];
    ///
    /// // We use trim_start just to show the table nicer in this example.
    /// let target = r#"
    /// Product  Price  Qty.
    /// --------------------
    /// Milk     5.00      3
    /// Cereal   10.00     1
    /// "#.trim_start();
    /// 
    /// assert_eq!(target, formatter.trio_table(header, rows));
    /// ```
    pub fn trio_table<A: Into<String>, B: Into<String>, C: Into<String>, D: IntoIterator<Item = (E, F, G)>, E: Into<String>, F: Into<String>, G: Into<String>>(&self, header: (A, B, C), rows: D) -> String {
        // Auxiliary closure for printing
        let aux_trio_table = |mut first: String, mut second: String, mut third: String, width: u8, limits: (u8, u8), replace_last: Option<char>| -> String {
            if first.len() > limits.0 as usize {
                let max_width = (limits.0 as usize) - 1;
                if let Some(replacement) = replace_last {
                    first.truncate(max_width);
                    first += &replacement.to_string();
                } else {
                    first.truncate(max_width);
                }
            }
            if second.len() > (limits.1 - limits.0) as usize {
                let max_width = (limits.1 - limits.0) as usize;
                if let Some(replacement) = replace_last {
                    second.truncate(max_width);
                    second += &replacement.to_string();
                } else {
                    second.truncate(max_width);
                }
            }
            if third.len() - 1 > (width - limits.1) as usize {
                let max_width = (width - limits.1) as usize;
                if let Some(replacement) = replace_last {
                    third.truncate(max_width);
                    third += &replacement.to_string();
                } else {
                    third.truncate(max_width);
                }
            }
            format!("{:<3$} {:^4$} {:>5$}\n",
                first,
                second,
                third,
                (limits.0 - 1) as usize,
                (limits.1 - limits.0) as usize,
                (width - limits.1 - 1) as usize
            )
        };

        // First step, is to find the maximum desirable width of a column.
        let header: (String, String, String) = (header.0.into(), header.1.into(), header.2.into());
        let mut max_left = header.0.len();
        let mut max_middle = header.1.len();
        let mut max_right = header.2.len();

        // I was not able to do 2 for loops with the IntoIterator trait with borrowed items :(
        let rows: Vec<(String, String, String)> = rows.into_iter().map(|(a, b, c)| (a.into(), b.into(), c.into())).collect();
        
        // Now we compare to all rows
        for row in &rows {
            if row.0.len() > max_left {
                max_left = row.0.len();
            }
            if row.1.len() > max_middle {
                max_middle = row.1.len();
            }
            if row.2.len() > max_right {
                max_right = row.2.len();
            }
        }

        let limits = if max_left + max_middle + max_right + 2 < self.width as usize {
            // Nothing to do, easy peasy
            ((max_left + 1) as u8, (self.width as usize - max_right - 1) as u8)
        } else {
            let mut limits = (0u8, self.width as u8);
            // The left-most column must be at least 4 characters wide, with the lowest priority
            if max_middle + max_right + 4 > (self.width as usize) {
                limits.0 = 4;
            } else {
                limits.0 = ((self.width as usize) - max_middle - max_right) as u8;
            }

            // Ahora para el segundo límite
            let remaining = self.width - limits.0;

            if (max_right as u8) + 4 > remaining {
                limits.1 = limits.0 + 4;
            } else {
                limits.1 = limits.0 + remaining - (max_right as u8);
            }
            limits
        };

        let mut content = aux_trio_table(header.0, header.1, header.2, self.width, limits, None);

        if let Some(hdp) = self.print_header_division_pattern() {
            content += &hdp;
        }

        for row in rows {
            content += &aux_trio_table(row.0, row.1, row.2, self.width, limits, None);
        }
        content
    }

    fn print_header_division_pattern(&self) -> Option<String> {
        if let Some(header_division_pattern) = &self.table_options.header_division_pattern {
            let mut line = header_division_pattern.repeat((self.width as usize) / header_division_pattern.len() + 1);
            line.truncate(self.width as usize);
            Some(line + "\n")
        } else {
            None
        }
    }
}