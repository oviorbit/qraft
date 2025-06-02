#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
use sqlx::{Arguments, IntoArguments};

use crate::{expr::TakeBindings, writer::FormatWriter};
use qraft_derive::Bindable;

// max size is 32 bytes
#[derive(Debug, Default, Clone, Bindable)]
pub enum Bind {
    #[default]
    #[bindable(ignore)]
    Consumed,
    String(Option<String>),
    StaticString(Option<&'static str>),
    Bool(Option<bool>),
    F32(Option<f32>),
    F64(Option<f64>),
    I8(Option<i8>),
    I16(Option<i16>),
    I32(Option<i32>),
    I64(Option<i64>),

    // unsigned not so sure about it ?
    U8(Option<u8>),
    U16(Option<u16>),
    U32(Option<u32>),
    U64(Option<u64>),

    #[bindable(ignore)]
    VecBytes(Option<Vec<u8>>),

    #[cfg(feature = "time")]
    Date(Option<time::Date>),
    #[cfg(feature = "time")]
    Time(Option<time::Time>),
    #[cfg(feature = "time")]
    Timestamptz(Option<time::OffsetDateTime>),
    #[cfg(feature = "time")]
    Timestamp(Option<time::PrimitiveDateTime>),

    #[cfg(feature = "chrono")]
    ChronoDate(Option<chrono::NaiveDate>),
    #[cfg(feature = "chrono")]
    ChronoTime(Option<chrono::NaiveTime>),
    #[cfg(feature = "chrono")]
    ChronoTimestamptzUtc(Option<chrono::DateTime<chrono::Utc>>),
    #[cfg(feature = "chrono")]
    ChronoTimestamptzLocal(Option<chrono::DateTime<chrono::Local>>),
    #[cfg(feature = "chrono")]
    ChronoTimestamp(Option<chrono::NaiveDateTime>),

    #[cfg(feature = "uuid")]
    Uuid(Option<uuid::Uuid>),

    #[cfg(feature = "json")]
    Json(Option<serde_json::Value>),
}

impl Bind {
    pub fn new<V>(value: V) -> Bind
    where
        V: IntoBind,
    {
        value.into_bind()
    }

    pub fn is_consumed(&self) -> bool {
        matches!(self, Self::Consumed)
    }

    pub fn new_static_str(value: &'static str) -> Bind {
        Bind::StaticString(Some(value))
    }
}

#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
trait EncodeBind<'q, DB: sqlx::Database> {
    fn encode_bind(self, binds: &mut <DB as sqlx::Database>::Arguments<'q>);
}

pub type Binds = Array<Bind>;

#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
impl<'q> EncodeBind<'q, sqlx::Postgres> for Bind {
    fn encode_bind(self, binds: &mut <sqlx::Postgres as sqlx::Database>::Arguments<'q>) {
        let _ = match self {
            Bind::Consumed => {
                debug_assert!(false, "Can't encode a consumed bind");
                Ok(())
            }
            Bind::String(value) => binds.add(value),
            Bind::StaticString(value) => binds.add(value),
            Bind::Bool(value) => binds.add(value),
            Bind::F32(value) => binds.add(value),
            Bind::F64(value) => binds.add(value),
            Bind::I8(value) => binds.add(value),
            Bind::I16(value) => binds.add(value),
            Bind::I32(value) => binds.add(value),
            Bind::I64(value) => binds.add(value),
            Bind::U8(value) => binds.add(value.map(|v| v as i8)),
            Bind::U16(value) => binds.add(value.map(|v| v as i16)),
            Bind::U32(value) => binds.add(value.map(|v| v as i32)),
            Bind::U64(value) => binds.add(value.map(|v| v as i64)),
            Bind::VecBytes(items) => binds.add(items),
            #[cfg(feature = "time")]
            Bind::Date(value) => binds.add(value),
            #[cfg(feature = "time")]
            Bind::Time(value) => binds.add(value),
            #[cfg(feature = "time")]
            Bind::Timestamptz(value) => binds.add(value),
            #[cfg(feature = "time")]
            Bind::Timestamp(value) => binds.add(value),
            #[cfg(feature = "chrono")]
            Bind::ChronoDate(value) => binds.add(value),
            #[cfg(feature = "chrono")]
            Bind::ChronoTime(value) => binds.add(value),
            #[cfg(feature = "chrono")]
            Bind::ChronoTimestamptzUtc(value) => binds.add(value),
            #[cfg(feature = "chrono")]
            Bind::ChronoTimestamptzLocal(value) => binds.add(value),
            #[cfg(feature = "chrono")]
            Bind::ChronoTimestamp(value) => binds.add(value),
            #[cfg(feature = "uuid")]
            Bind::Uuid(value) => binds.add(value),
            #[cfg(feature = "json")]
            Bind::Json(value) => binds.add(value),
        };
    }
}

#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
impl<'q, DB> IntoArguments<'q, DB> for Binds
where
    DB: sqlx::Database,
    Bind: EncodeBind<'q, DB>,
{
    fn into_arguments(self) -> <DB as sqlx::Database>::Arguments<'q> {
        let mut arguments = <DB as sqlx::Database>::Arguments::default();
        for bind in self {
            bind.encode_bind(&mut arguments);
        }
        arguments
    }
}

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

    pub fn push(&mut self, other: T) {
        self.append(Array::One(other));
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

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    pub fn len(&self) -> usize {
        match self {
            Array::None => 0,
            Array::One(_) => 1,
            Array::Many(items) => items.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

impl Binds {
    pub fn normalize(&mut self) {
        let old = std::mem::take(self);

        *self = match old {
            Array::None => Array::None,

            Array::One(b) => {
                if b.is_consumed() {
                    Array::None
                } else {
                    Array::One(b)
                }
            }

            Array::Many(mut vec) => {
                vec.retain(|b| !b.is_consumed());

                match vec.len() {
                    0 => Array::None,
                    1 => {
                        // safe we have at least 1
                        let only = vec.into_iter().next().expect("we already checked the len");
                        Array::One(only)
                    }
                    _ => Array::Many(vec),
                }
            }
        };
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
