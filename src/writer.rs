use std::{fmt::Write, ops::Deref};

use crate::{dialect::Dialect, Ident};

pub(crate) trait FormatWriter {
    fn format_writer<W: Write>(&self, context: &mut FormatContext<'_, W>) -> std::fmt::Result;
}

pub(crate) struct FormatContext<'a, W: Write> {
    pub(crate) writer: &'a mut W,
    pub(crate) dialect: Dialect,
    pub(crate) placeholder: u16,
}

impl<'a, W: Write> FormatContext<'a, W> {
    pub fn new(writer: &'a mut W, dialect: Dialect) -> Self {
        Self {
            writer,
            dialect,
            placeholder: 0,
        }
    }

    pub(crate) fn write_table(&mut self, ident: &str) -> std::fmt::Result {
        for (i, part) in ident.split('.').enumerate() {
            if i > 0 {
                self.writer.write_char('.')?;
            }
            self.write_ident(part)?;
        }
        Ok(())
    }

    pub(crate) fn write_ident(&mut self, part: &str) -> std::fmt::Result {
        if part == "*" {
            return self.writer.write_char('*');
        }

        let quote = match self.dialect {
            Dialect::Postgres | Dialect::Sqlite => '"',
            Dialect::MySql => '`',
        };
        self.writer.write_char(quote)?;
        // duplicate the quote if present
        let dbl = if quote == '"' { "\"\"" } else { "``" };

        let mut last = 0;
        for (index, char) in part.char_indices() {
            if char == quote {
                if index != last {
                    self.writer.write_str(&part[last..index])?;
                }
                self.writer.write_str(dbl)?;
                last = index + char.len_utf8();
            }
        }

        // write trailing slice
        if last < part.len() {
            self.writer.write_str(&part[last..])?;
        }

        self.writer.write_char(quote)?;
        Ok(())
    }

    pub fn write_alias(&mut self, alias: Option<&Ident>) -> std::fmt::Result {
        if let Some(ref alias) = alias {
            self.writer.write_str(" as ")?;
            alias.format_writer(self)?;
        }
        Ok(())
    }

    pub(crate) fn write_placeholder(&mut self) -> std::fmt::Result {
        self.placeholder += 1;
        write!(self.writer, "${}", self.placeholder)?;
        Ok(())
    }
}

impl<D> FormatWriter for D
where
    D: Deref,
    D::Target: FormatWriter,
{
    fn format_writer<W: std::fmt::Write>(
        &self,
        ctx: &mut FormatContext<'_, W>,
    ) -> std::fmt::Result {
        self.deref().format_writer(ctx)
    }
}
