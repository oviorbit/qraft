use qraft::{
    bind::Bind, col::TableSchema, ident::{Ident, IntoIdent, TableRef}, Builder
};

#[derive(Debug)]
pub struct User {
    id: i64,
    username: String,
    email: String,
    team_id: i64,
}

#[derive(Debug)]
pub struct Team {
    id: i64,
}

pub trait Relationship<From: Model, To: Model> {
    type Output;
    fn resolve(parent: &From) -> Self::Output;
}

impl<From: Model, To: Model> Relationship<From, To> for BelongsTo<From, To>
where
    From: Model,
    To: Model,
{
    type Output = Option<To>;

    fn resolve(parent: &From) -> Self::Output {
        let owner_key = From::primary_key();
        let foreign_key = To::primary_key();
        todo!()
    }
}

#[derive(Debug)]
pub struct BelongsTo<F, To> {
    inner: Builder,
    owner_key: Ident,
    foreign_key: Ident,
    _marker: std::marker::PhantomData<(F, To)>,
}

impl<F, To> BelongsTo<F, To>
where
    F: Model,
    To: Model,
{
    pub fn new(inner: Builder, owner_key: Ident, foreign_key: Ident) -> Self {
        Self {
            inner,
            owner_key,
            foreign_key,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn owner_key<I>(mut self, ident: I) -> Self
    where
        I: IntoIdent,
    {
        self.owner_key = ident.into_ident();
        self
    }

    pub fn foreign_key<I>(mut self, ident: I) -> Self
    where
        I: IntoIdent,
    {
        self.foreign_key = ident.into_ident();
        self
    }
}

pub trait Model: Query + PrimaryKey {
    fn get_field(&self, field: &str) -> Option<Bind>;
}

pub trait PrimaryKey {
    fn primary_key() -> Ident;
}

impl TableSchema for Team {
    fn table() -> qraft::ident::TableRef {
        TableRef::ident_static("teams")
    }
}

impl PrimaryKey for Team {
    fn primary_key() -> Ident {
        Ident::new_static("id")
    }
}

impl PrimaryKey for User {
    fn primary_key() -> Ident {
        Ident::new_static("id")
    }
}

pub trait Query: TableSchema {
    fn query() -> Builder;
}

impl<T> Query for T
where
    T: TableSchema,
{
    fn query() -> Builder {
        Builder::table_schema::<T>()
    }
}

impl TableSchema for User {
    fn table() -> qraft::ident::TableRef {
        TableRef::ident_static("users")
    }
}

pub fn main() {
    let user = User {
        id: 10,
        username: "bob".into(),
        email: "test".into(),
        team_id: 1,
    };
}
