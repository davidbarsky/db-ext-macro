use db_ext_macro::query_group;
use salsa::Setter;

#[query_group]
pub trait DatabaseOne: salsa::Database {
    #[db_ext_macro::input]
    fn input_string(&self) -> String;

    // unadorned query
    fn length(&self, key: ()) -> usize;
}

#[query_group]
pub trait DatabaseTwo: DatabaseOne {
    fn second_length(&self, key: ()) -> usize;
}

fn length(db: &dyn DatabaseOne, key: ()) -> usize {
    let _ = key;
    db.input_string().len()
}

fn second_length(db: &dyn DatabaseTwo, key: ()) -> usize {
    let _ = key;
    db.input_string().len()
}