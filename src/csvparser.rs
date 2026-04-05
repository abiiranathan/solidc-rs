use std::ffi::{CStr, CString};
use std::path::Path;

use crate::ffi;

/// A CSV reader for parsing CSV files.
pub struct CsvReader {
    inner: *mut ffi::CsvReader,
}

/// A row from a CSV file.
pub struct CsvRow {
    fields: Vec<String>,
}

impl CsvRow {
    /// Returns the fields in this row.
    pub fn fields(&self) -> &[String] {
        &self.fields
    }

    /// Returns the number of fields.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Returns true if the row has no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Gets a field by index.
    pub fn get(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(|s| s.as_str())
    }
}

impl std::ops::Index<usize> for CsvRow {
    type Output = String;
    fn index(&self, index: usize) -> &String {
        &self.fields[index]
    }
}

/// Configuration for the CSV reader.
#[derive(Debug, Clone)]
pub struct ReaderConfig {
    pub delim: char,
    pub quote: char,
    pub comment: char,
    pub has_header: bool,
    pub skip_header: bool,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        ReaderConfig {
            delim: ',',
            quote: '"',
            comment: '#',
            has_header: false,
            skip_header: false,
        }
    }
}

impl CsvReader {
    /// Opens a CSV file for reading.
    pub fn open<P: AsRef<Path>>(path: P, arena_memory: usize) -> Option<Self> {
        let path_str = path.as_ref().to_str()?;
        let cstr = CString::new(path_str).ok()?;
        let ptr = unsafe { ffi::csv_reader_new(cstr.as_ptr(), arena_memory) };
        if ptr.is_null() {
            None
        } else {
            Some(CsvReader { inner: ptr })
        }
    }

    /// Sets reader configuration.
    pub fn set_config(&mut self, config: &ReaderConfig) {
        let c_config = ffi::CsvReaderConfig {
            delim: config.delim as i8,
            quote: config.quote as i8,
            comment: config.comment as i8,
            has_header: config.has_header,
            skip_header: config.skip_header,
        };
        unsafe { ffi::csv_reader_setconfig(self.inner, c_config) };
    }

    /// Parses all rows from the CSV file.
    pub fn parse(&mut self) -> Vec<CsvRow> {
        let rows_ptr = unsafe { ffi::csv_reader_parse(self.inner) };
        if rows_ptr.is_null() {
            return Vec::new();
        }

        let num_rows = unsafe { ffi::csv_reader_numrows(self.inner) };
        let mut result = Vec::with_capacity(num_rows);

        for i in 0..num_rows {
            let row_ptr = unsafe { *rows_ptr.add(i) };
            if row_ptr.is_null() {
                continue;
            }
            let row = unsafe { &*row_ptr };
            let mut fields = Vec::with_capacity(row.count);
            for j in 0..row.count {
                let field_ptr = unsafe { *row.fields.add(j) };
                if field_ptr.is_null() {
                    fields.push(String::new());
                } else {
                    let s = unsafe { CStr::from_ptr(field_ptr) };
                    fields.push(s.to_string_lossy().into_owned());
                }
            }
            result.push(CsvRow { fields });
        }

        result
    }

    /// Returns the number of rows.
    pub fn num_rows(&self) -> usize {
        unsafe { ffi::csv_reader_numrows(self.inner) }
    }
}

impl Drop for CsvReader {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::csv_reader_free(self.inner) };
        }
    }
}

unsafe impl Send for CsvReader {}

/// A CSV writer for creating CSV files.
pub struct CsvWriter {
    inner: *mut ffi::CsvWriter,
}

impl CsvWriter {
    /// Creates a new CSV writer for the given file.
    pub fn new<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path_str = path.as_ref().to_str()?;
        let cstr = CString::new(path_str).ok()?;
        let ptr = unsafe { ffi::csvwriter_new(cstr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(CsvWriter { inner: ptr })
        }
    }

    /// Writes a row of fields to the CSV file.
    pub fn write_row(&mut self, fields: &[&str]) -> bool {
        let c_strings: Vec<CString> = fields
            .iter()
            .filter_map(|s| CString::new(*s).ok())
            .collect();
        if c_strings.len() != fields.len() {
            return false;
        }

        let ptrs: Vec<*const i8> = c_strings.iter().map(|s| s.as_ptr()).collect();

        unsafe {
            ffi::csvwriter_write_row(
                self.inner,
                ptrs.as_ptr() as *mut *const i8,
                ptrs.len(),
            )
        }
    }
}

impl Drop for CsvWriter {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::csvwriter_free(self.inner) };
        }
    }
}

unsafe impl Send for CsvWriter {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_write_and_read_csv() {
        let path = "/tmp/test_solidc_csv.csv";

        // Write
        {
            let mut writer = CsvWriter::new(path).unwrap();
            assert!(writer.write_row(&["name", "age", "city"]));
            assert!(writer.write_row(&["Alice", "30", "NYC"]));
            assert!(writer.write_row(&["Bob", "25", "LA"]));
        }

        // Read
        {
            let mut reader = CsvReader::open(path, 0).unwrap();
            let rows = reader.parse();
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0].get(0), Some("name"));
            assert_eq!(rows[1].get(0), Some("Alice"));
            assert_eq!(rows[2].get(1), Some("25"));
        }

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_row_access() {
        let row = CsvRow {
            fields: vec!["a".into(), "b".into(), "c".into()],
        };
        assert_eq!(row.len(), 3);
        assert!(!row.is_empty());
        assert_eq!(row.get(1), Some("b"));
        assert_eq!(row.get(5), None);
        assert_eq!(&row[0], "a");
    }
}
