# qraft

`qraft` is a lightweight SQL query builder for Rust. It provides a fluent API
for assembling `select`, `insert`, `update` and `delete` statements and
supports parameter binding out of the box. When the optional `sqlx` features
are enabled, queries can be executed directly against a database.

## Features

- Compose complex queries using a builder style API

- Support for PostgreSQL, MySQL and SQLite dialects

- Integration with [`sqlx`](https://crates.io/crates/sqlx) for executing queries

- Helper functions to embed raw SQL fragments or sub‑queries

- Optional features for `time`, `chrono`, `uuid` and `serde_json` bindings

## Installation

Add `qraft` to your `Cargo.toml` and enable the dialect features you need:

```toml
[dependencies]
qraft = { version = "0.1.0", features = ["postgres", "chrono"] }
```

## Example

```rust
use qraft::{Builder, Postgres};

let mut query = Builder::table("users");
query.select(["id", "username"]).where_eq("id", 1);

let sql = query.to_sql::<Postgres>();
assert_eq!("select \"id\", \"username\" from \"users\" where \"id\" = $1", sql);
```

### Joins

```rust
use qraft::{Builder, Postgres};

let mut query = Builder::table("users");
query
    .join("contacts", "users.id", "=", "contacts.user_id")
    .join("orders", "users.id", '=', "orders.user_id")
    .select(["users.*", "contacts.phone", "orders.price"]);

let sql = query.to_sql::<Postgres>();
assert_eq!(
    r#"select "users".*, "contacts"."phone", "orders"."price" from "users" inner join "contacts" on "users"."id" = "contacts"."user_id" inner join "orders" on "users"."id" = "orders"."user_id""#,
    sql,
);
```

### Aggregates

```rust
let mut query = Builder::table("users");
query.where_eq("id", 1).select_avg("price as avg_price");

let sql = query.to_sql::<Postgres>();
assert_eq!(r#"select avg("price") as "avg_price" from "users" where "id" = $1"#, sql);
```

### Inserts

```rust
let mut builder = Builder::table("users");
let insert_sql = builder
    .inserting()
    .values_with(|row| {
        row.field("id", 1).field("username", "ovior");
    })
    .to_sql::<Postgres>();

assert_eq!(
    r#"insert into "users" ("id", "username") values ($1, $2)"#,
    insert_sql,
);
```

### Exists Subquery

```rust
let mut query = Builder::new();
query.select_exists(|b| {
    b.select_one().from("users").where_eq("id", 1);
});

let sql = query.to_sql::<Postgres>();
assert_eq!(
    r#"select exists (select 1 from "users" where "id" = $1) as "exists""#,
    sql,
);
```

## License

This project is released under the MIT License.
