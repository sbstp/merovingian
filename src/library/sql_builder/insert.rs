use std::fmt;

use super::shared::{Bind, Ident};

enum Upsert<'t> {
    Unset,
    Nothing,
    Update(&'t [&'t str]),
}

pub struct InsertInto<'t> {
    table: &'t str,
    fields: &'t [&'t str],
    upsert: Upsert<'t>,
}

impl<'t> InsertInto<'t> {
    pub fn on_conflict_update(mut self, conflict_columns: &'t [&'t str]) -> Self {
        self.upsert = Upsert::Update(conflict_columns);
        self
    }

    pub fn on_conflict_do_nothing(mut self) -> Self {
        self.upsert = Upsert::Nothing;
        self
    }
}

impl fmt::Display for InsertInto<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "INSERT INTO {} (", Ident(self.table))?;

        for (idx, field) in self.fields.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", Ident(field))?;
        }

        write!(f, ") VALUES (")?;

        for (idx, field) in self.fields.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", Bind(field))?;
        }

        write!(f, ")")?;

        match self.upsert {
            Upsert::Unset => {}
            Upsert::Nothing => {
                write!(f, " ON ONFLICT DO NOTHING")?;
            }
            Upsert::Update(cols) => {
                write!(f, " ON CONFLICT (")?;
                for (idx, col) in cols.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", Ident(col))?;
                }

                write!(f, ") DO UPDATE SET (")?;

                for (idx, field) in self.fields.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", Ident(field))?;
                }

                write!(f, ") = (")?;

                for (idx, field) in self.fields.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "excluded.{}", Ident(field))?;
                }

                write!(f, ")")?;
            }
        }

        Ok(())
    }
}

pub fn insert_into<'t>(table: &'t str, fields: &'t [&'t str]) -> InsertInto<'t> {
    InsertInto {
        table,
        fields: fields.as_ref(),
        upsert: Upsert::Unset,
    }
}
