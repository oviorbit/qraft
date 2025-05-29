// max size is 32 bytes
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

pub type Binds = Array<Bind>;

// if T <= 32 bytes we are good and it's a free data structure.
#[derive(Debug, Default)]
pub enum Array<T> {
    #[default]
    None,
    One(T),
    Many(Vec<T>)
}

impl<T> Array<T> {
    pub fn append(&mut self, other: Self) {
        let combined = match (std::mem::replace(self, Self::None), other) {
            (Self::None, cols) | (cols, Self::None) => cols,
            (Self::One(a), Self::One(b)) =>
                Self::Many(vec![a, b]),
            (Self::One(a), Self::Many(mut b)) => {
                b.insert(0, a);
                Self::Many(b)
            }
            (Self::Many(mut a), Self::One(b)) => {
                a.push(b);
                Self::Many(a)
            }
            (Self::Many(mut a), Self::Many(mut b)) => {
                a.append(&mut b);
                Self::Many(a)
            }
        };
        *self = combined;
    }

    pub fn reset(&mut self) {
        *self = Self::None;
    }

    pub fn into_vec(self) -> Vec<T> {
        match self {
            Self::None => Vec::new(),
            Self::One(one) => Vec::from([one]),
            Self::Many(many) => many,
        }
    }
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
