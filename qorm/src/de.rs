use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeSeed, Error, MapAccess, SeqAccess, Visitor, value::Error as DeError},
    forward_to_deserialize_any,
};
use sqlx::{
    Column, Row, TypeInfo, ValueRef,
    postgres::{PgRow, PgValueRef},
};

pub fn from_pg_row<T>(row: PgRow) -> Result<T, DeError>
where
    T: for<'de> Deserialize<'de>,
{
    let deserializer = PgRowDeserializer::new(&row);
    T::deserialize(deserializer)
}

#[derive(Debug, Clone, Copy)]
pub struct PgRowDeserializer<'a> {
    pub(crate) row: &'a PgRow,
    pub(crate) index: usize,
}

impl<'a> PgRowDeserializer<'a> {
    pub fn new(row: &'a PgRow) -> Self {
        PgRowDeserializer { row, index: 0 }
    }
}

impl<'de, 'a> Deserializer<'de> for PgRowDeserializer<'a> {
    type Error = DeError;

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw_value = self.row.try_get_raw(0).unwrap();

        if raw_value.is_null() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.row.columns().len() {
            0 => return visitor.visit_unit(),
            1 => {}
            _n => {
                return self.deserialize_seq(visitor);
            }
        };

        let raw_value = self.row.try_get_raw(self.index).unwrap();
        let type_info = raw_value.type_info();

        if raw_value.is_null() {
            return visitor.visit_none();
        }

        // If this is a BOOL[], TEXT[], etc
        //if type_name.ends_with("[]") {
        //    return self.deserialize_seq(visitor);
        //}

        // Direct all "basic" types down to `PgValueDeserializer`
        let deserializer = PgValueDeserializer { value: raw_value };

        deserializer.deserialize_any(visitor)
    }

    /// We treat the row as a map (each column is a key/value pair)
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(PgRowMapAccess {
            deserializer: self,
            num_cols: self.row.columns().len(),
        })
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw_value = self.row.try_get_raw(self.index).unwrap();
        let type_info = raw_value.type_info();
        let type_name = type_info.name();

        match type_name {
            "TEXT[]" | "VARCHAR[]" => {
                let seq_access = PgArraySeqAccess::<String>::new(raw_value)?;
                visitor.visit_seq(seq_access)
            }
            "INT4[]" => {
                let seq_access = PgArraySeqAccess::<i32>::new(raw_value)?;
                visitor.visit_seq(seq_access)
            }
            "JSON[]" | "JSONB[]" => {
                todo!()
            }
            "BOOL[]" => {
                let seq_access = PgArraySeqAccess::<bool>::new(raw_value)?;
                visitor.visit_seq(seq_access)
            }
            _ => {
                let seq_access = PgRowSeqAccess {
                    deserializer: self,
                    num_cols: self.row.columns().len(),
                };

                visitor.visit_seq(seq_access)
            }
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    // For other types, forward to deserialize_any.
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf unit unit_struct
        tuple_struct enum identifier ignored_any
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

fn decode_raw_pg<'a, T>(raw_value: PgValueRef<'a>) -> Result<T, DeError>
where
    T: sqlx::Decode<'a, sqlx::Postgres>,
{
    T::decode(raw_value).map_err(|err| {
        panic!(
            "Failed to decode {} value: {:?}",
            std::any::type_name::<T>(),
            err,
        )
    })
}

#[derive(Clone)]
pub(crate) struct PgValueDeserializer<'a> {
    pub(crate) value: PgValueRef<'a>,
}

impl<'de, 'a> Deserializer<'de> for PgValueDeserializer<'a> {
    type Error = DeError;

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_null() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_null() {
            return visitor.visit_none();
        }
        let type_info = self.value.type_info();

        let type_name = type_info.name();

        match type_name {
            "FLOAT4" => {
                let v = decode_raw_pg::<f32>(self.value)?;
                visitor.visit_f32(v)
            }
            "FLOAT8" => {
                let v = decode_raw_pg::<f64>(self.value)?;
                visitor.visit_f64(v)
            }
            "NUMERIC" => {
                todo!()
            }
            "INT8" => {
                let v = decode_raw_pg::<i64>(self.value)?;
                visitor.visit_i64(v)
            }
            "INT4" => {
                let v = decode_raw_pg::<i32>(self.value)?;
                visitor.visit_i32(v)
            }
            "INT2" => {
                let v = decode_raw_pg::<i16>(self.value)?;
                visitor.visit_i16(v)
            }
            "BOOL" => {
                let v = decode_raw_pg::<bool>(self.value)?;
                visitor.visit_bool(v)
            }
            "DATE" => {
                todo!()
            }
            "TIME" | "TIMETZ" => {
                todo!()
            }
            "TIMESTAMP" | "TIMESTAMPTZ" => {
                visitor.visit_unit()
            }
            "UUID" => {
                todo!()
            }
            "BYTEA" => {
                let bytes = decode_raw_pg::<&[u8]>(self.value)?;
                visitor.visit_bytes(bytes)
            }
            "INTERVAL" => {
                todo!()
            }
            "CHAR" | "TEXT" => {
                let s = decode_raw_pg::<String>(self.value)?;
                visitor.visit_string(s)
            }
            "JSON" | "JSONB" => {
                todo!()
            }
            _other => {
                let as_string = decode_raw_pg::<String>(self.value.clone())?;
                visitor.visit_string(as_string)
            }
        }
    }

    // For other types, forward to deserialize_any.
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct struct
        tuple_struct enum identifier ignored_any tuple seq map
    }
}

use std::fmt::Debug;

/// A SeqAccess implementation that iterates over the row’s columns
pub(crate) struct PgRowSeqAccess<'a> {
    pub(crate) deserializer: PgRowDeserializer<'a>,
    pub(crate) num_cols: usize,
}

impl<'de, 'a> SeqAccess<'de> for PgRowSeqAccess<'a> {
    type Error = DeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.deserializer.index < self.num_cols {
            let value = self
                .deserializer
                .row
                .try_get_raw(self.deserializer.index)
                .map_err(DeError::custom)?;

            // Create a PgValueDeserializer for the current column.
            let pg_value_deserializer = PgValueDeserializer { value };

            self.deserializer.index += 1;

            // Deserialize the value and return it wrapped in Some.
            seed.deserialize(pg_value_deserializer).map(Some)
        } else {
            Ok(None)
        }
    }
}

use serde::de::IntoDeserializer;

/// SeqAccess implementation for Postgres arrays
/// It decodes a raw Postgres array, such as TEXT[] into a `Vec<Option<T>>` and
/// then yields each element during deserialization
pub struct PgArraySeqAccess<T> {
    iter: std::vec::IntoIter<Option<T>>,
}

impl<'a, T> PgArraySeqAccess<T>
where
    T: sqlx::Decode<'a, sqlx::Postgres> + Debug,
{
    pub fn new(value: PgValueRef<'a>) -> Result<Self, DeError>
    where
        Vec<Option<T>>: sqlx::Decode<'a, sqlx::Postgres> + Debug,
    {
        let vec: Vec<Option<T>> = decode_raw_pg(value)?;

        Ok(PgArraySeqAccess {
            iter: vec.into_iter(),
        })
    }
}

impl<'de, T> SeqAccess<'de> for PgArraySeqAccess<T>
where
    T: IntoDeserializer<'de, DeError>,
{
    type Error = DeError;

    fn next_element_seed<U>(&mut self, seed: U) -> Result<Option<U::Value>, Self::Error>
    where
        U: DeserializeSeed<'de>,
    {
        let Some(value) = self.iter.next() else {
            return Ok(None);
        };

        seed.deserialize(PgArrayElementDeserializer { value })
            .map(Some)
    }
}

/// Yet another deserializer, this time to handles Options
struct PgArrayElementDeserializer<T> {
    pub value: Option<T>,
}

impl<'de, T> de::Deserializer<'de> for PgArrayElementDeserializer<T>
where
    T: IntoDeserializer<'de, DeError>,
{
    type Error = DeError;

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(v) => visitor.visit_some(v.into_deserializer()),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(v) => v.into_deserializer().deserialize_any(visitor),
            None => Err(DeError::custom(
                "unexpected null in non-optional array element",
            )),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

pub(crate) struct PgRowMapAccess<'a> {
    pub(crate) deserializer: PgRowDeserializer<'a>,
    pub(crate) num_cols: usize,
}

impl<'de, 'a> MapAccess<'de> for PgRowMapAccess<'a> {
    type Error = DeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.deserializer.index < self.num_cols {
            let col_name = self.deserializer.row.columns()[self.deserializer.index].name();
            // Use the column name as the key
            seed.deserialize(col_name.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self
            .deserializer
            .row
            .try_get_raw(self.deserializer.index)
            .unwrap();
        let pg_type_deserializer = PgValueDeserializer { value };

        self.deserializer.index += 1;

        seed.deserialize(pg_type_deserializer)
    }
}
