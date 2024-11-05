use db_ext_macro::query_group;
use expect_test::expect;

mod logger_db;
use logger_db::LoggerDb;

#[query_group]
pub trait HelloWorldDatabase: salsa::Database {
    // input
    #[db_ext_macro::input]
    fn input_string(&self, key: ()) -> String;

    // unadorned query
    fn length_query(&self, key: ()) -> usize;

    // input with no params
    #[db_ext_macro::input]
    fn input_string_with_no_params(&self) -> String;

    // unadorned query
    fn length_query_with_no_params(&self) -> usize;

    // renamed/invoke query
    #[db_ext_macro::invoke(invoke_length_query_actual)]
    fn invoke_length_query(&self, key: ()) -> usize;

    // not a query. should not invoked
    #[db_ext_macro::transparent]
    fn transparent_length(&self, key: ()) -> usize;

    #[db_ext_macro::transparent]
    #[db_ext_macro::invoke(transparent_and_invoke_length_actual)]
    fn transparent_and_invoke_length(&self, key: ()) -> usize;
}

fn length_query(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    db.input_string(key).len()
}

fn length_query_with_no_params(db: &dyn HelloWorldDatabase) -> usize {
    db.input_string_with_no_params().len()
}

fn invoke_length_query_actual(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    db.input_string(key).len()
}

fn transparent_length(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    db.input_string(key).len()
}

fn transparent_and_invoke_length_actual(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    db.input_string(key).len()
}

#[test]
fn unadorned_query() {
    let mut db = LoggerDb::default();

    db.set_input_string((), String::from("Hello, world!"));
    let len = db.length_query(());

    assert_eq!(len, 13);
    db.assert_logs(expect![[r#"
        [
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(DidValidateMemoizedValue { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: length_query_shim(Id(800)) })",
            "salsa_event(WillCheckCancellation)",
        ]"#]]);
}

#[test]
fn input_with_no_params() {
    let mut db = LoggerDb::default();

    db.set_input_string_with_no_params(String::from("Hello, world!"));
    let len = db.length_query_with_no_params();

    assert_eq!(len, 13);
    db.assert_logs(expect![[r#"
        [
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(DidValidateMemoizedValue { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: length_query_with_no_params_shim(Id(400)) })",
            "salsa_event(WillCheckCancellation)",
        ]"#]]);
}

#[test]
fn invoke_query() {
    let mut db = LoggerDb::default();

    db.set_input_string((), String::from("Hello, world!"));
    let len = db.invoke_length_query(());

    assert_eq!(len, 13);
    db.assert_logs(expect![[r#"
        [
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(DidValidateMemoizedValue { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: invoke_length_query_shim(Id(800)) })",
            "salsa_event(WillCheckCancellation)",
        ]"#]]);
}

#[test]
fn transparent() {
    let mut db = LoggerDb::default();

    db.set_input_string((), String::from("Hello, world!"));
    let len = db.transparent_length(());

    assert_eq!(len, 13);
    db.assert_logs(expect![[r#"
        [
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(DidValidateMemoizedValue { database_key: create_data(Id(0)) })",
        ]"#]]);
}

#[test]
fn transparent_invoke() {
    let mut db = LoggerDb::default();

    db.set_input_string((), String::from("Hello, world!"));
    let len = db.transparent_and_invoke_length(());

    assert_eq!(len, 13);
    db.assert_logs(expect![[r#"
        [
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(DidValidateMemoizedValue { database_key: create_data(Id(0)) })",
        ]"#]]);
}
