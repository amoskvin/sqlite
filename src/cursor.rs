use ffi;
use statement::{State, Statement};
use {Result, Value};

/// An iterator over rows.
pub struct Cursor<'l> {
    state: Option<State>,
    values: Option<Vec<Value>>,
    statement: Statement<'l>,
}

impl<'l> Cursor<'l> {
    /// Bind values to all parameters.
    pub fn bind(&mut self, values: &[Value]) -> Result<()> {
        self.state = None;
        self.statement.reset()?;
        for (i, value) in values.iter().enumerate() {
            self.statement.bind(i + 1, value)?;
        }
        Ok(())
    }

    /// Return the number of columns.
    #[inline]
    pub fn column_count(&self) -> usize {
        self.statement.column_count()
    }

    /// Advance to the next row and read all columns.
    pub fn next(&mut self) -> Result<Option<&[Value]>> {
        match self.state {
            Some(State::Row) => {}
            Some(State::Done) => return Ok(None),
            _ => {
                self.state = Some(self.statement.next()?);
                return self.next();
            }
        }
        self.values = match self.values.take() {
            Some(mut values) => {
                for (i, value) in values.iter_mut().enumerate() {
                    *value = self.statement.read(i)?;
                }
                Some(values)
            }
            _ => {
                let count = self.statement.column_count();
                let mut values = Vec::with_capacity(count);
                for i in 0..count {
                    values.push(self.statement.read(i)?);
                }
                Some(values)
            }
        };
        self.state = Some(self.statement.next()?);
        Ok(Some(self.values.as_ref().unwrap()))
    }

    /// Return the raw pointer.
    #[inline]
    pub fn as_raw(&self) -> *mut ffi::sqlite3_stmt {
        self.statement.as_raw()
    }
}

#[inline]
pub fn new<'l>(statement: Statement<'l>) -> Cursor<'l> {
    Cursor {
        state: None,
        values: None,
        statement: statement,
    }
}
