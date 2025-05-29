#[derive(Debug, Clone)]
pub enum Bind {
    Null,
    String(String),
    StaticString(&'static str),
    Bool(bool),
    F32(f32),
    F64(f64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    // unsigned not so sure about it ?
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

pub trait IntoBind {
    fn into_bind(self) -> Bind;
}

impl<T> IntoBind for Option<T>
where
    T: IntoBind
{
    fn into_bind(self) -> Bind {
        if let Some(value) = self {
            value.into_bind()
        } else {
            Bind::Null
        }
    }
}

impl IntoBind for i32 {
    fn into_bind(self) -> Bind {
        Bind::I32(self)
    }
}

impl IntoBind for u64 {
    fn into_bind(self) -> Bind {
        Bind::U64(self)
    }
}
