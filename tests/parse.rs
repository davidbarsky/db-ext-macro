use db_ext_macro::db_ext;

#[db_ext]
pub trait HelloWorldDatabase: salsa::Database {
    // #[db_ext_macro::input]
    // fn input_string(&self, key: ()) -> String;

    #[db_ext_macro::invoke(length)]
    fn length2(&self, key: ()) -> usize;

    fn length3(&self, key: ()) -> usize;
}

fn length(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    let _ = db;
    let _key = key;
    todo!()
}

fn length3(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    let _ = db;
    let _key = key;
    todo!()
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
