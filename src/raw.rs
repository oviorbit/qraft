use smol_str::SmolStr;

use crate::{
    dialect::Dialect,
    writer::{self, FormatWriter},
};

#[derive(Debug, Clone)]
pub struct Raw(SmolStr);

impl Raw {
    pub fn new<T>(value: T) -> Self
    where
        T: Into<SmolStr>,
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
        context: &mut writer::FormatContext<'_, W>,
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
            Tag,
        }

        let mut state = State::Normal;

        let mut span_start = 0;

        let mut lit_start = 0;
        let mut lit_end = 0;

        let mut ident_start = 0;
        let mut ident_end = 0;

        let mut tag_start = 0;
        let mut tag_end = 0;

        let max_len = sql.len();

        let mut chars = sql.char_indices().peekable();
        while let Some((index, char)) = chars.next() {
            match state {
                State::Normal => {
                    match char {
                        '\'' => {
                            context.writer.write_str(&sql[span_start..index])?;
                            // escaped
                            lit_start = index;
                            lit_end = index + char.len_utf8();

                            span_start = index;
                            state = State::Lit;
                        }
                        '"' => {
                            context.writer.write_str(&sql[span_start..index])?;
                            // goto ident
                            ident_start = index;
                            ident_end = index + char.len_utf8();

                            span_start = index;
                            state = State::Ident;
                        }
                        '?' => {
                            // could be placeholder but not 100%
                            // if jsonb or if double ?? then not placeholder
                            let is_placeholder = if let Some(&(_, next_ch)) = chars.peek() {
                                next_ch != '?' && next_ch != '|' && next_ch != '&'
                            } else {
                                true
                            };

                            if let Some(&(_, next_ch)) = chars.peek() {
                                if next_ch == '?' || next_ch == '|' || next_ch == '&' {
                                    let _ = chars.next();
                                    continue;
                                }
                            }

                            if is_placeholder {
                                context.writer.write_str(&sql[span_start..index])?;
                                context.write_placeholder()?;
                                span_start = index + char.len_utf8();
                            }
                        }
                        '$' => {
                            context.writer.write_str(&sql[span_start..index])?;
                            // goto tag
                            tag_start = index;
                            tag_end = index + char.len_utf8();

                            span_start = index;
                            state = State::Tag;
                        }
                        _ => {}
                    }
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
                }
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
                State::Tag => {
                    while let Some(&(next_idx, next_ch)) = chars.peek() {
                        let w = next_ch.len_utf8();
                        chars.next();
                        tag_end = next_idx + w;

                        if next_ch == '$' {
                            state = State::Normal;
                            break;
                        }
                    }

                    context.writer.write_str(&sql[tag_start..tag_end])?;
                    span_start = tag_end;
                }
            }
        }

        if span_start < max_len {
            context.writer.write_str(&sql[span_start..])?;
        }

        Ok(())
    }
}

pub trait IntoRaw {
    fn into_raw(self) -> Raw;
}

impl<T> IntoRaw for T
where
    T: Into<SmolStr>,
{
    fn into_raw(self) -> Raw {
        Raw::new(self.into())
    }
}

impl IntoRaw for Raw {
    fn into_raw(self) -> Raw {
        self
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
    fn test_placeholder_escaped() {
        let value = Raw::new_static("test ??");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("test ??", raw);
    }

    #[test]
    fn test_placeholder_jsonb() {
        let value = Raw::new_static("test ?| some");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("test ?| some", raw);
        let value = Raw::new_static("test ?& some");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("test ?& some", raw);
    }

    #[test]
    fn test_placeholder_tag() {
        let value = Raw::new_static("test $so?me$");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("test $so?me$", raw);
    }

    #[test]
    fn test_placeholder_tag_unfinish() {
        let value = Raw::new_static("test $so?me$$");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("test $so?me$$", raw);
    }

    #[test]
    fn test_placeholder_quote_unfinished() {
        let value = Raw::new_static("test $so?me$ test '");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("test $so?me$ test '", raw);
        let value = Raw::new_static("test $so?me$ test $");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("test $so?me$ test $", raw);
        let value = Raw::new_static("test $so?me$ test ' bob");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("test $so?me$ test ' bob", raw);
    }

    #[test]
    fn test_double_single_quote() {
        let value = Raw::new_static("''");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!("''", raw);
    }

    #[test]
    fn test_full_query() {
        let value = Raw::new_static("select * from users where \"userna?me\" = ? and \"id\" = ?");
        let raw = format_writer(value, Dialect::Postgres);
        assert_eq!(
            "select * from users where \"userna?me\" = $1 and \"id\" = $2",
            raw
        );
    }
}
