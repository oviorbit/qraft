use flex::{Builder, Postgres};

fn main() {
    let mut builder = Builder::table("users");
    builder.where_eq("username", "foo");

    for _ in 0..100 {
        builder.where_eq("username", "foo");
    }

    let sql = builder.to_sql::<Postgres>();

    println!("{}", sql);
}
