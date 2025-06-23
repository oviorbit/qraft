use qraft::{
    Builder,
    bind::{Bind, Binds, IntoBind},
    col::TableSchema,
    dialect::{HasDialect, MySql},
    ident::{Ident, IntoIdent},
};
use sqlx::{PgPool, prelude::FromRow};
use std::ops::{Deref, DerefMut};

mod de;

#[derive(Debug, FromRow)]
pub struct User {
    // #[qraft(primary)] not needed since `id` by default the primary key.
    id: i64,
    name: String,
    email: String,
    team_id: i64,
    password: String,
}

impl User {
    pub fn team(&self) -> BelongsTo<Team> {
        self.belongs_to::<Team>()
    }
}

#[derive(Debug, FromRow)]
pub struct Team {
    id: i64,
}

impl Team {
    pub fn user(&self) -> HasOne<User> {
        self.has_one::<User>()
    }
}

impl GetField for Team {
    fn get_field(&self, field: &str) -> Option<Bind> {
        match field {
            "id" => Some(Bind::from(self.id)),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct BelongsToBuilder<M> {
    builder: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
    related: std::marker::PhantomData<M>,
}

impl<M> BelongsToBuilder<M> {
    pub fn new(table: Ident, owner_key: Ident, foreign_value: Bind) -> Self {
        Self {
            builder: Builder::table(table.clone()),
            owner_key,
            foreign_value,
            table,
            related: std::marker::PhantomData,
        }
    }

    fn ensure_inner(&mut self) {
        self.builder.where_eq(
            self.table.dot(self.owner_key.clone()),
            std::mem::take(&mut self.foreign_value),
        );
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

    pub fn finish(mut self) -> BelongsTo<M> {
        // Ensure the inner builder is initialized
        self.ensure_inner();
        BelongsTo {
            builder: self.builder,
            table: self.table,
            owner_key: self.owner_key,
            foreign_value: self.foreign_value,
            related: std::marker::PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct BelongsTo<M> {
    builder: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
    related: std::marker::PhantomData<M>,
}

impl<M> Deref for BelongsTo<M> {
    type Target = Builder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl<M> DerefMut for BelongsTo<M> {
    fn deref_mut(&mut self) -> &mut Builder {
        &mut self.builder
    }
}

impl<M: Model> BelongsTo<M> {
    pub async fn first<DB, E>(mut self, executor: E) -> Result<M, sqlx::Error>
    where
        DB: sqlx::Database + HasDialect,
        M: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        E: for<'c> sqlx::Executor<'c, Database = DB>,
        Binds: for<'c> sqlx::IntoArguments<'c, DB>,
    {
        let bindings = self.builder.bindings_mut().take();
        let sql = self.to_sql::<DB>();
        sqlx::query_as_with::<_, M, _>(&sql, bindings)
            .fetch_one(executor)
            .await
    }

    pub fn into_builder(self) -> Builder {
        self.builder
    }
}

pub struct BelongsToMany<From, M, P = InferredPivot<From, M>> {
    builder: Builder,
    _from: std::marker::PhantomData<From>,
    _to: std::marker::PhantomData<M>,
    _pivot: std::marker::PhantomData<P>,
}

pub type InferredPivot<From, To> = (From, To);

pub struct HasOneBuilder<M> {
    builder: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
    related: std::marker::PhantomData<M>,
}

pub struct HasOne<M> {
    builder: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
    related: std::marker::PhantomData<M>,
}

impl<M> Deref for HasOne<M> {
    type Target = Builder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl<M> DerefMut for HasOne<M> {
    fn deref_mut(&mut self) -> &mut Builder {
        &mut self.builder
    }
}

impl<M: Model> HasOne<M> {
    pub async fn first<DB, E>(mut self, executor: E) -> Result<M, sqlx::Error>
    where
        DB: sqlx::Database + HasDialect,
        M: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        E: for<'c> sqlx::Executor<'c, Database = DB>,
        Binds: for<'c> sqlx::IntoArguments<'c, DB>,
    {
        let bindings = self.builder.bindings_mut().take();
        let sql = self.to_sql::<DB>();
        sqlx::query_as_with::<_, M, _>(&sql, bindings)
            .fetch_one(executor)
            .await
    }

    pub fn into_builder(self) -> Builder {
        self.builder
    }
}

impl<M> HasOneBuilder<M> {
    pub fn new(table: Ident, owner_key: Ident, foreign_value: Bind) -> Self {
        Self {
            builder: Builder::table(table.clone()),
            owner_key,
            foreign_value,
            table,
            related: std::marker::PhantomData,
        }
    }

    fn ensure_inner(&mut self) {
        // "select * from "users" where "users"."team_id" = 1 and "users"."team_id" is not null"
        let owner_key = self.table.dot(self.owner_key.clone());
        self.builder
            .where_eq(owner_key.clone(), std::mem::take(&mut self.foreign_value))
            .where_not_null(owner_key);
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

    pub fn finish(mut self) -> HasOne<M> {
        // Ensure the inner builder is initialized
        self.ensure_inner();
        HasOne {
            builder: self.builder,
            table: self.table,
            owner_key: self.owner_key,
            foreign_value: self.foreign_value,
            related: std::marker::PhantomData,
        }
    }
}

pub struct HasMany<M> {
    builder: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
    related: std::marker::PhantomData<M>,
}

impl<M> HasMany<M> {
    pub fn one(self) -> HasOneBuilder<M> {
        HasOneBuilder {
            builder: self.builder,
            table: self.table,
            owner_key: self.owner_key,
            foreign_value: self.foreign_value,
            related: std::marker::PhantomData,
        }
    }
}

pub struct HasManyBuilder<M> {
    builder: Builder,
    table: Ident,
    owner_key: Ident,
    foreign_value: Bind,
    related: std::marker::PhantomData<M>,
}

impl<M> HasManyBuilder<M> {
    pub fn new(table: Ident, owner_key: Ident, foreign_value: Bind) -> Self {
        Self {
            builder: Builder::table(table.clone()),
            owner_key,
            foreign_value,
            table,
            related: std::marker::PhantomData,
        }
    }

    fn ensure_inner(&mut self) {
        // "select * from "users" where "users"."team_id" = 1 and "users"."team_id" is not null"
        let owner_key = self.table.dot(self.owner_key.clone());
        self.builder
            .where_eq(owner_key.clone(), std::mem::take(&mut self.foreign_value))
            .where_not_null(owner_key);
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

    pub fn finish(mut self) -> HasMany<M> {
        // Ensure the inner builder is initialized
        self.ensure_inner();
        HasMany {
            builder: self.builder,
            table: self.table,
            owner_key: self.owner_key,
            foreign_value: self.foreign_value,
            related: std::marker::PhantomData,
        }
    }
}

impl<M> Deref for HasMany<M> {
    type Target = Builder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl<M> DerefMut for HasMany<M> {
    fn deref_mut(&mut self) -> &mut Builder {
        &mut self.builder
    }
}

impl Relation for User {}
impl Relation for Team {}

impl GetField for User {
    fn get_field(&self, field: &str) -> Option<Bind> {
        match field {
            "id" => Some(Bind::from_bind(self.id)),
            "name" => Some(Bind::from_bind(self.name.clone())),
            "email" => Some(Bind::from_bind(self.email.clone())),
            "team_id" => Some(Bind::from_bind(self.team_id)),
            _ => None,
        }
    }
}

impl<T> Model for T where T: TableSchema + ForeignKey + PrimaryKey + GetField {}

pub trait Relation: GetField + ForeignKey {
    fn belongs_to<M: Model>(&self) -> BelongsTo<M> {
        self.belongs_to_with::<M>().finish()
    }

    fn belongs_to_with<M: Model>(&self) -> BelongsToBuilder<M> {
        // select * from teams where teams.id = users.team_id
        let m_table = M::table();
        let m_pk = M::primary_key();
        let m_fk = M::foreign_key();
        let value = self
            .get_field(m_fk.as_str())
            .expect("Foreign key not found in User model");
        BelongsToBuilder::new(m_table, m_pk, value)
    }

    fn has_one<M: Model>(&self) -> HasOne<M> {
        self.has_many::<M>().one().finish()
    }

    fn has_one_with<M: Model>(&self) -> HasOneBuilder<M> {
        self.has_many::<M>().one()
    }

    fn has_many<M: Model>(&self) -> HasMany<M> {
        self.has_many_with::<M>().finish()
    }

    fn has_many_with<M: Model>(&self) -> HasManyBuilder<M> {
        // "select * from "users" where "users"."team_id" = ? and "users"."team_id" is not null"
        let m_table = M::table(); // users
        let s_fk = Self::foreign_key(); // team_id
        // self is team, so I need to get primary key
        let m_pk = M::primary_key(); // id
        let value = self
            .get_field(m_pk.as_str())
            .expect("Primary key not found in Team model");
        HasManyBuilder::new(m_table, s_fk, value)
    }

    fn belongs_to_many<M: Model>(&self) -> BelongsToMany<Ident, M> {
        BelongsToMany {
            builder: M::query(),
            _from: std::marker::PhantomData,
            _to: std::marker::PhantomData,
            _pivot: std::marker::PhantomData,
        }
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

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost/cybersci").await?;

    let team = Team { id: 2 };

    println!("first team {:?}", team);

    let mut user = team.user();
    let sql = user.to_sql::<MySql>();
    let bindings = user.bindings();

    // select * from teams where teams.id = users.team_id
    //println!("SQL: {} and binds {:?}", sql, bindings);
    //
    //let first_row = sqlx::query("select 1, true")
    //    .fetch_one(&pool)
    //    .await?;
    let row: User = user.take().first(&pool).await?;
    println!("first user {:?}", row);

    // query
    //let user: User = de::from_pg_row(row)?;
    //println!("first team {:?}", user);
    //let row_json = serde_json::to_string(&row)?;
    //println!("row team {:?}", row_json);

    Ok(())
}
