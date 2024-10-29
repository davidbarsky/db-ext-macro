use db_ext_macro::db_ext;

#[db_ext]
pub trait HelloWorldDatabase: salsa::Database {
    #[db_ext_macro::input]
    fn input_string(&self, key: ()) -> String;

    #[db_ext_macro::invoke(length)]
    fn length_query(&self, key: ()) -> usize;

    #[db_ext_macro::invoke(length2)]
    fn lengthhhhhh(&self, key: ()) -> usize;

    #[db_ext_macro::transparent]
    fn length3(&self, key: ()) -> usize;

    #[db_ext_macro::transparent]
    #[db_ext_macro::invoke(completely_different_length)]
    fn length4(&self, key: ()) -> usize;
}

fn length(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    db.input_string(key).len()
}

fn length2(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    db.input_string(key).len()
}

fn length3(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    db.input_string(key).len()
}

fn completely_different_length(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    db.input_string(key).len()
}

#[salsa::db]
#[derive(Default)]
pub struct HelloWorldDb {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for HelloWorldDb {
    fn salsa_event(&self, _event: &dyn Fn() -> salsa::Event) {}
}

#[test]
fn parses() {
    let mut db = HelloWorldDb::default();
    db.set_input_string((), String::from("Hello, world!"));

    let len = db.length_query(());
    assert_eq!(len, 13);
}
