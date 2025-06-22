use std::ops::{Deref, DerefMut};

use qraft::{
    bind::{Bind, IntoBind}, col::TableSchema, dialect::MySql, ident::{Ident, IntoIdent}, Builder
};

#[derive(Debug)]
pub struct User {
    id: i64,
    username: String,
    email: String,
    team_id: i64,
}

impl User {
    pub fn team(&self) -> BelongsTo {
        self.belongs_to::<Team>()
    }
}

#[derive(Debug)]
pub struct Team {
    id: i64,
}

impl GetField for Team {
    fn get_field(&self, field: &str) -> Option<Bind> {
        match field {
            "id" => Some(Bind::from(self.id)),
            _ => None,
        }
    }
}

pub struct BelongsTo {
    inner: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
}

impl Deref for BelongsTo {
    type Target = Builder;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for BelongsTo {
    fn deref_mut(&mut self) -> &mut Builder {
        self.ensure_inner();
        &mut self.inner
    }
}

impl BelongsTo {
    pub fn new(table: Ident, owner_key: Ident, foreign_value: Bind) -> Self {
        Self {
            inner: Builder::table(table.clone()),
            owner_key,
            foreign_value,
            table,
        }
    }

    fn ensure_inner(&mut self) {
        if !self.inner.has_where() {
            // cheap arc clone of ident
            self.inner.where_eq(
                self.table.dot(self.owner_key.clone()),
                self.foreign_value.clone(),
            );
        }
    }

    pub fn owner_key<I>(mut self, ident: I) -> Self
    where
        I: IntoIdent,
    {
        self.owner_key = ident.into_ident();
        self
    }

    pub fn foreign_value<B>(mut self, ident: B) -> Self
    where
        B: IntoBind,
    {
        self.foreign_value = ident.into_bind();
        self
    }
}

impl BelongsToModel for User {
    fn belongs_to<M: Model>(&self) -> BelongsTo {
        // select * from teams where teams.id = users.team_id
        let m_table = M::table();
        let m_pk = M::primary_key();
        let m_fk = M::foreign_key();
        let value = self
            .get_field(m_fk.as_str())
            .expect("Foreign key not found in User model");
        BelongsTo::new(m_table, m_pk, value)
    }
}

impl GetField for User {
    fn get_field(&self, field: &str) -> Option<Bind> {
        match field {
            "id" => Some(Bind::from(self.id)),
            "username" => Some(Bind::from(self.username.clone())),
            "email" => Some(Bind::from(self.email.clone())),
            "team_id" => Some(Bind::from(self.team_id)),
            _ => None,
        }
    }
}

impl<T> Model for T where T: TableSchema + ForeignKey + PrimaryKey + GetField {}

pub trait BelongsToModel {
    fn belongs_to<M: Model>(&self) -> BelongsTo;
}

pub trait GetField {
    fn get_field(&self, field: &str) -> Option<Bind>;
}

pub trait Model: Query + PrimaryKey + ForeignKey + GetField {}

pub trait ForeignKey {
    fn foreign_key() -> Ident;
}

pub trait PrimaryKey {
    fn primary_key() -> Ident;
}

impl TableSchema for Team {
    fn table() -> Ident {
        Ident::new_static("teams")
    }
}

impl PrimaryKey for Team {
    fn primary_key() -> Ident {
        Ident::new_static("id")
    }
}

impl ForeignKey for Team {
    fn foreign_key() -> Ident {
        Ident::new_static("team_id")
    }
}

impl ForeignKey for User {
    fn foreign_key() -> Ident {
        Ident::new_static("user_id")
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
        Builder::table_as::<T>()
    }
}

impl TableSchema for User {
    fn table() -> Ident {
        Ident::new_static("users")
    }
}

pub fn main() {
    let user = User {
        id: 10,
        username: "bob".into(),
        email: "test".into(),
        team_id: 1,
    };

    let mut team = user.team();
    let sql = team.to_sql::<MySql>();
    let bindings = team.bindings();

    // select * from teams where teams.id = users.team_id
    println!("SQL: {} and binds {:?}", sql, bindings);
}
