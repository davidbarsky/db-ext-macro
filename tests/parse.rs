use db_ext_macro::db_ext;

#[db_ext]
pub trait HelloWorldDatabase: salsa::Database {
    fn input_string(&self, key: ()) -> String;

    fn length(&self, key: ()) -> usize;
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
