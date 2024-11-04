use db_ext_macro::query_group;
use expect_test::expect;

mod logger_db;
use logger_db::LoggerDb;

#[query_group]
pub trait CycleDatabase: salsa::Database {
    // input
    #[db_ext_macro::input]
    fn input_string(&self, key: ()) -> String;

    #[db_ext_macro::cycle(recover)]
    fn length(&self, key: ()) -> usize;
}

fn length(db: &dyn CycleDatabase, key: ()) -> usize {
    db.input_string(key).len()
}

fn recover(
    _db: &dyn CycleDatabase,
    _cycle: &salsa::Cycle,
    _input: CycleDatabaseData, // TODO: figure out how to not rely on this generated struct?
    _key: (),
) -> usize {
    0
}

#[test]
fn parses() {
    let mut db = LoggerDb::default();

    db.set_input_string((), String::from("Hello, world!"));
    let len = db.length(());

    assert_eq!(len, 13);
    db.assert_logs(expect![[r#"
        [
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(DidValidateMemoizedValue { database_key: create_data(Id(0)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: length_shim(Id(800)) })",
            "salsa_event(WillCheckCancellation)",
        ]"#]]);
}
