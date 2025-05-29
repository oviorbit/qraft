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

impl IntoBinds for Binds {
    fn into_binds(self) -> Binds {
        self
    }
}

impl IntoBinds for () {
    fn into_binds(self) -> Binds {
        Binds::None
    }
}

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

    pub fn len(&self) -> usize {
        match self {
            Array::None => 0,
            Array::One(_) => 1,
            Array::Many(items) => items.len(),
        }
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

pub trait IntoBinds {
    fn into_binds(self) -> Binds;
}

impl<T> IntoBinds for T
where
    T: IntoBind
{
    fn into_binds(self) -> Binds {
        Binds::One(self.into_bind())
    }
}

impl<T> IntoBinds for Vec<T>
where
    T: IntoBind
{
    fn into_binds(self) -> Binds {
        Binds::Many(self.into_iter().map(IntoBind::into_bind).collect())
    }
}

impl<T, const N: usize> IntoBinds for [T; N]
where
    T: IntoBind
{
    fn into_binds(self) -> Binds {
        let mut iter = self.into_iter().map(IntoBind::into_bind);
        match N {
            0 => Binds::None,
            1 => {
                let one = iter.next().expect("safe since N is 1");
                Binds::One(one)
            }
            _ => {
                Binds::Many(iter.collect())
            }
        }
    }
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
