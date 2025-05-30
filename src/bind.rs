use crate::{scalar::TakeBindings, writer::FormatWriter};

// max size is 32 bytes
#[derive(Debug, Clone)]
pub enum Bind {
    Null,
    Consumed,
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

impl Bind {
    pub fn new<V>(value: V) -> Bind
    where
        V: IntoBind,
    {
        value.into_bind()
    }

    pub fn new_static_str(value: &'static str) -> Bind {
        Bind::StaticString(value)
    }
}

pub type Binds = Array<Bind>;

impl TakeBindings for Binds {
    fn take_bindings(&mut self) -> Binds {
        let b: Vec<Bind> = self
            .iter_mut()
            .map(|v| std::mem::replace(v, Bind::Consumed))
            .collect();
        Binds::Many(b)
    }
}

impl IntoBinds for Binds {
    fn into_binds(self) -> Binds {
        self
    }
}

// if T <= 32 bytes we are good and it's a free data structure.
#[derive(Debug, Default, Clone)]
pub enum Array<T> {
    #[default]
    None,
    One(T),
    Many(Vec<T>),
}

impl FormatWriter for Binds {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        for (index, _) in self.iter().enumerate() {
            if index > 0 {
                context.writer.write_str(", ")?;
            }
            context.write_placeholder()?;
        }
        Ok(())
    }
}

pub enum ArrayIter<'a, T> {
    None,
    One(Option<&'a T>),
    Many(std::slice::Iter<'a, T>),
}

impl<'a, T> Iterator for ArrayIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ArrayIter::None => None,
            ArrayIter::One(o) => o.take(),
            ArrayIter::Many(i) => i.next(),
        }
    }
}

pub enum ArrayIterMut<'a, T> {
    None,
    One(Option<&'a mut T>),
    Many(std::slice::IterMut<'a, T>),
}

impl<'a, T> Iterator for ArrayIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ArrayIterMut::None => None,
            ArrayIterMut::One(o) => o.take(),
            ArrayIterMut::Many(i) => i.next(),
        }
    }
}

impl<'a, T> IntoIterator for &'a Array<T> {
    type Item = &'a T;
    type IntoIter = ArrayIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Array<T> {
    type Item = &'a mut T;
    type IntoIter = ArrayIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub enum ArrayIntoIter<T> {
    None,
    One(Option<T>),
    Many(std::vec::IntoIter<T>),
}

impl<T> Iterator for ArrayIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ArrayIntoIter::None => None,
            ArrayIntoIter::One(opt) => opt.take(),
            ArrayIntoIter::Many(iter) => iter.next(),
        }
    }
}

impl<T> IntoIterator for Array<T> {
    type Item = T;
    type IntoIter = ArrayIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Array::None => ArrayIntoIter::None,
            Array::One(val) => ArrayIntoIter::One(Some(val)),
            Array::Many(vec) => ArrayIntoIter::Many(vec.into_iter()),
        }
    }
}

impl<T> Array<T> {
    pub fn iter(&self) -> ArrayIter<'_, T> {
        match self {
            Array::None => ArrayIter::None,
            Array::One(x) => ArrayIter::One(Some(x)),
            Array::Many(xs) => ArrayIter::Many(xs.iter()),
        }
    }

    pub fn iter_mut(&mut self) -> ArrayIterMut<'_, T> {
        match self {
            Array::None => ArrayIterMut::None,
            Array::One(x) => ArrayIterMut::One(Some(x)),
            Array::Many(xs) => ArrayIterMut::Many(xs.iter_mut()),
        }
    }

    pub fn append(&mut self, other: Self) {
        let combined = match (std::mem::replace(self, Self::None), other) {
            (Self::None, cols) | (cols, Self::None) => cols,
            (Self::One(a), Self::One(b)) => Self::Many(vec![a, b]),
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
    T: IntoBind,
{
    fn into_binds(self) -> Binds {
        Binds::One(self.into_bind())
    }
}

impl<T> IntoBinds for Vec<T>
where
    T: IntoBind,
{
    fn into_binds(self) -> Binds {
        Binds::Many(self.into_iter().map(IntoBind::into_bind).collect())
    }
}

impl<T, const N: usize> IntoBinds for [T; N]
where
    T: IntoBind,
{
    fn into_binds(self) -> Binds {
        let mut iter = self.into_iter().map(IntoBind::into_bind);
        match N {
            0 => Binds::None,
            1 => {
                let one = iter.next().expect("safe since N is 1");
                Binds::One(one)
            }
            _ => Binds::Many(iter.collect()),
        }
    }
}

impl<T> IntoBind for Option<T>
where
    T: IntoBind,
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

impl IntoBind for String {
    fn into_bind(self) -> Bind {
        Bind::String(self)
    }
}

impl IntoBind for &str {
    fn into_bind(self) -> Bind {
        Bind::String(self.into())
    }
}
