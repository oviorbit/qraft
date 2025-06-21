use qraft::Builder;

#[derive(Debug)]
pub struct User {
}

impl User {
    pub fn create() -> InsertBuilder {
        Builder::table("users")
            .inserting();
    }
}
