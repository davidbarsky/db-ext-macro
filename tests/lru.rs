use expect_test::expect;
use salsa::Setter as _;

mod logger_db;
use logger_db::LoggerDb;

#[db_ext_macro::query_group]
pub trait LruDB: salsa::Database {
    // // input with no params
    #[db_ext_macro::input]
    fn input_string(&self) -> String;

    #[db_ext_macro::lru]
    fn length_query(&self, key: ()) -> usize;

    #[db_ext_macro::lru]
    #[db_ext_macro::invoke(invoked_query)]
    fn length_query_invoke(&self, key: ()) -> usize;
}

fn length_query(db: &dyn LruDB, key: ()) -> usize {
    let _ = key;
    db.input_string().len()
}

fn invoked_query(db: &dyn LruDB, key: ()) -> usize {
    let _ = key;
    db.input_string().len()
}

#[test]
fn plain_lru() {
    let mut db = LoggerDb::default();

    db.set_input_string(String::from("Hello, world!"));
    let len = db.length_query(());

    assert_eq!(len, 13);
    db.assert_logs(expect![[r#"
        [
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: create_data_LruDB(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(DidValidateMemoizedValue { database_key: create_data_LruDB(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: length_query_shim(Id(800)) })",
            "salsa_event(WillCheckCancellation)",
        ]"#]]);
}

#[test]
fn invoke_lru() {
    let mut db = LoggerDb::default();

    db.set_input_string(String::from("Hello, world!"));
    let len = db.length_query_invoke(());

    assert_eq!(len, 13);
    db.assert_logs(expect![[r#"
        [
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: create_data_LruDB(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(DidValidateMemoizedValue { database_key: create_data_LruDB(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: length_query_invoke_shim(Id(800)) })",
            "salsa_event(WillCheckCancellation)",
        ]"#]]);
}
