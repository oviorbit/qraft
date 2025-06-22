use std::ops::{Deref, DerefMut};

use qraft::{
    Builder,
    bind::{Bind, IntoBind},
    col::TableSchema,
    dialect::MySql,
    ident::{Ident, IntoIdent},
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
    builder: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
}

impl Deref for BelongsTo {
    type Target = Builder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl DerefMut for BelongsTo {
    fn deref_mut(&mut self) -> &mut Builder {
        self.ensure_inner();
        &mut self.builder
    }
}

impl BelongsTo {
    pub fn new(table: Ident, owner_key: Ident, foreign_value: Bind) -> Self {
        Self {
            builder: Builder::table(table.clone()),
            owner_key,
            foreign_value,
            table,
        }
    }

    fn ensure_inner(&mut self) {
        if ! self.builder.is_dirty() {
            self.builder.where_eq(
                self.table.dot(self.owner_key.clone()),
                std::mem::take(&mut self.foreign_value),
            );
        }
    }

    pub fn owner_key<I>(mut self, ident: I) -> Self
    where
        I: IntoIdent,
    {
        debug_assert!(
            !self.builder.is_dirty(),
            "Cannot set owner_key when the builder is already dirty"
        );

        self.owner_key = ident.into_ident();
        self
    }

    pub fn foreign_value<B>(mut self, ident: B) -> Self
    where
        B: IntoBind,
    {
        debug_assert!(
            !self.builder.is_dirty(),
            "Cannot set foreign_value when the builder is already dirty"
        );

        self.foreign_value = ident.into_bind();
        self
    }
}

pub struct HasOne {
    builder: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
}

impl Deref for HasOne {
    type Target = Builder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl DerefMut for HasOne {
    fn deref_mut(&mut self) -> &mut Builder {
        self.ensure_inner();
        &mut self.builder
    }
}

impl HasOne {
    pub fn new(table: Ident, owner_key: Ident, foreign_value: Bind) -> Self {
        Self {
            builder: Builder::table(table.clone()),
            owner_key,
            foreign_value,
            table,
        }
    }

    fn ensure_inner(&mut self) {
        // "select * from "users" where "users"."team_id" = 1 and "users"."team_id" is not null"
        if !self.builder.is_dirty() {
            let owner_key = self.table.dot(self.owner_key.clone());
            self.builder.where_eq(
                owner_key.clone(),
                std::mem::take(&mut self.foreign_value),
            )
                .where_not_null(owner_key);
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

pub struct HasMany {
    builder: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
}

impl Deref for HasMany {
    type Target = Builder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl DerefMut for HasMany {
    fn deref_mut(&mut self) -> &mut Builder {
        self.ensure_inner();
        &mut self.builder
    }
}

impl HasMany {
    pub fn new(table: Ident, owner_key: Ident, foreign_value: Bind) -> Self {
        Self {
            builder: Builder::table(table.clone()),
            owner_key,
            foreign_value,
            table,
        }
    }

    fn ensure_inner(&mut self) {
        // "select * from "users" where "users"."team_id" = 1 and "users"."team_id" is not null"
        if !self.builder.is_dirty() {
            let owner_key = self.table.dot(self.owner_key.clone());
            self.builder.where_eq(
                owner_key.clone(),
                std::mem::take(&mut self.foreign_value),
            )
                .where_not_null(owner_key);
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

    pub fn one(self) -> HasOne {
        HasOne {
            builder: self.builder,
            table: self.table,
            owner_key: self.owner_key,
            foreign_value: self.foreign_value,
        }
    }
}

impl Relation for User {}
impl Relation for Team {}

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

pub trait Relation: GetField + ForeignKey {
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

    fn has_one<M: Model>(&self) -> HasOne {
        self.has_many::<M>().one()
    }

    fn has_many<M: Model>(&self) -> HasMany {
        // "select * from "users" where "users"."team_id" = ? and "users"."team_id" is not null"
        let m_table = M::table(); // users
        let s_fk = Self::foreign_key(); // team_id
        // self is team, so I need to get primary key
        let m_pk = M::primary_key(); // id
        let value = self
            .get_field(m_pk.as_str())
            .expect("Primary key not found in Team model");
        HasMany::new(m_table, s_fk, value)
    }

    fn belongs_to_many<M: Model>(&self) -> Builder {
        todo!()
    }
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
    team.where_true("active");
    team = team.owner_key("bob");
    let sql = team.to_sql::<MySql>();
    let bindings = team.bindings();

    // select * from teams where teams.id = users.team_id
    println!("SQL: {} and binds {:?}", sql, bindings);
}
