use smol_str::SmolStr;

use crate::{dialect::Dialect, writer::FormatWriter};

#[derive(Debug)]
pub struct Raw(SmolStr);

impl Raw {
    pub fn new<T>(value: T) -> Self
    where
        T: Into<SmolStr>
    {
        Self(value.into())
    }

    pub fn new_static(value: &'static str) -> Self {
        Self(SmolStr::new_static(value))
    }
}

impl FormatWriter for Raw {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        let sql = self.0.as_str();

        if !matches!(context.dialect, Dialect::Postgres) {
            return context.writer.write_str(sql);
        }

        // state
        enum State {
            Normal,
            Ident,
            Lit,
        }

        let mut state = State::Normal;

        let mut span_start = 0;

        let mut lit_start = 0;
        let mut lit_end = 0;

        let mut ident_start = 0;
        let mut ident_end = 0;

        let mut chars = sql.char_indices().peekable();
        while let Some((index, char)) = chars.next() {
            match state {
                State::Normal => {
                    if char == '\'' {
                        context.writer.write_str(&sql[span_start..index])?;
                        // escaped
                        lit_start = index;
                        lit_end = index + char.len_utf8();

                        state = State::Lit;
                    } else if char == '"' {
                        context.writer.write_str(&sql[span_start..index])?;
                        // goto ident
                        ident_start = index;
                        ident_end = index + char.len_utf8();

                        state = State::Ident;
                    } else if char == '?' {
                        // could be placeholder but not 100%
                        // if jsonb or if double ?? then not placeholder
                        let is_placeholder = if let Some(&(_, next_ch)) = chars.peek() {
                            next_ch != '?' && next_ch != '|' && next_ch != '&'
                        } else {
                            true
                        };

                        if is_placeholder {
                            context.writer.write_str(&sql[span_start..index])?;
                            context.write_placeholder()?;
                            span_start = index + char.len_utf8();
                        }
                        chars.next();
                    }
                    // write the rest of raw string
                }
                State::Ident => {
                     while let Some(&(next_idx, next_ch)) = chars.peek() {
                        let w = next_ch.len_utf8();
                        chars.next();
                        ident_end = next_idx + w;

                        if next_ch == '"' {
                            if let Some(&(_, '"')) = chars.peek() {
                                if let Some((esc_idx, _)) = chars.next() {
                                    // double quotted
                                    ident_end = esc_idx + w;
                                    continue;
                                }
                            }
                            state = State::Normal;
                            break;
                        }
                    }

                    context.writer.write_str(&sql[ident_start..ident_end])?;
                    span_start = ident_end;
                },
                State::Lit => {
                     while let Some(&(next_idx, next_ch)) = chars.peek() {
                        let w = next_ch.len_utf8();
                        chars.next();
                        lit_end = next_idx + w;

                        if next_ch == '\'' {
                            if let Some(&(_, '\'')) = chars.peek() {
                                if let Some((esc_idx, _)) = chars.next() {
                                    // double quotted
                                    lit_end = esc_idx + w;
                                    continue;
                                }
                            }
                            state = State::Normal;
                            break;
                        }
                    }

                    context.writer.write_str(&sql[lit_start..lit_end])?;
                    span_start = lit_end;
                }
            }
        }

        if span_start < sql.len() {
            context.writer.write_str(&sql[span_start..])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::format_writer;

    use super::*;

    #[test]
    fn test_raw_str() {
        let value = Raw::new_static("'te?st'");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("'te?st'", raw);
    }

    #[test]
    fn test_raw_double_quote() {
        let value = Raw::new_static("'te''? st'");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("'te''? st'", raw);
    }

    #[test]
    fn test_raw_bind() {
        let value = Raw::new_static("'test' = ?");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("'test' = $1", raw);
    }

    #[test]
    fn test_raw_ident() {
        let value = Raw::new_static("\"te? ? \"\"st\" = ?");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("\"te? ? \"\"st\" = $1", raw);
    }

    #[test]
    fn test_placeholder_double() {
        let value = Raw::new_static("test ??");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("test ??", raw);
    }
}
