use db_ext_macro::query_group;

use expect_test::expect;

mod logger_db;
use logger_db::LoggerDb;

#[salsa::interned]
pub struct InternedStringId<'db> {
    data: InternedString,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct InternedString {
    data: String,
}

#[query_group]
pub trait InternedDB: salsa::Database {
    #[db_ext_macro::input]
    fn string(&self) -> String;

    #[db_ext_macro::interned]
    fn intern_string(&self, data: InternedString) -> InternedStringId<'_>;

    fn interned_len<'db>(&self, id: InternedStringId<'db>) -> usize;
}

fn interned_len<'db>(db: &dyn InternedDB, id: InternedStringId<'db>) -> usize {
    db.lookup_intern_string(id).data.len()
}

#[test]
fn intern_round_trip() {
    let db = LoggerDb::default();

    let id = db.intern_string(InternedString {
        data: String::from("Hello, world!"),
    });
    let s = db.lookup_intern_string(id);

    assert_eq!(s.data.len(), 13);
    db.assert_logs(expect![[r#"[]"#]]);
}

#[test]
fn intern_with_query() {
    let db = LoggerDb::default();

    let id = db.intern_string(InternedString {
        data: String::from("Hello, world!"),
    });
    let len = db.interned_len(id);

    assert_eq!(len, 13);
    db.assert_logs(expect![[r#"
        [
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: create_data_InternedDB(Id(400)) })",
            "salsa_event(WillCheckCancellation)",
            "salsa_event(WillExecute { database_key: interned_len_shim(Id(c00)) })",
        ]"#]]);
}
