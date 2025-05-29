mod dialect;
mod writer;
mod ident;

struct QueryBuilder {
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    //fn test_basic_select() {
    //    let mut builder = QueryBuilder::table("users");
    //    let sql = builder.to_sql();
    //    assert_eq!(sql, r#"select * from "users""#);
    //
    //    let sql = builder.to_sql::<MySql>();
    //    assert_eq!(sql, r#"select * from `users`"#);
    //}
    //
    //#[test]
    //fn test_select_column() {
    //    let mut builder = QueryBuilder::select("foo")
    //        .from("users");
    //    let sql = builder.to_sql();
    //    assert_eq!(sql, r#"select "foo" from "users""#);
    //
    //    let sql = builder.to_sql::<MySql>();
    //    assert_eq!(sql, r#"select `foo` from `users`"#);
    //
    //    let builder = builder.select(["foo", "bar"]).from("users");
    //
    //    let sql = builder.to_sql();
    //    assert_eq!(sql, r#"select "foo", "bar" from "users""#);
    //}
    //
    //#[test]
    //fn test_quoted_table() {
    //    let mut builder = QueryBuilder::table("test\"table");
    //
    //    let sql = builder.to_sql();
    //    assert_eq!(sql, r#"select * from "some""table""#);
    //
    //    let sql = builder.to_sql::<MySql>();
    //    assert_eq!(sql, r#"select * from `some"table`"#);
    //
    //    // escape mysql table also
    //    let mut builder = QueryBuilder::table("test`table");
    //    let sql = builder.to_sql::<MySql>();
    //    assert_eq!(sql, r#"select * from `some``table`"#);
    //}
    //
    //#[test]
    //fn test_alias_ident() {
    //    let mut builder = QueryBuilder::table("users");
    //    builder.select("x.y as foo.bar");
    //
    //    let sql = builder.to_sql();
    //    assert_eq!(sql, r#"select "x"."y" as "foo.bar" from "users""#);
    //}
    //
    //#[test]
    //fn test_alias_space_ident() {
    //    let mut builder = QueryBuilder::table("users");
    //    builder.select("w x.y.z as foo.bar");
    //
    //    let sql = builder.to_sql();
    //    assert_eq!(sql, r#"select "w x"."y"."z" as "foo.bar" from "users""#);
    //}
}
