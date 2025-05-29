mod dialect;
mod writer;
mod ident;
mod operator;
mod raw;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use crate::{dialect, writer};

    pub(crate) fn format_writer<W: writer::FormatWriter>(writer: W, dialect: dialect::Dialect) -> String {
        let mut str = String::new();
        let mut context = writer::FormatContext::new(&mut str, dialect);
        writer.format_writer(&mut context).unwrap();
        str
    }
}
