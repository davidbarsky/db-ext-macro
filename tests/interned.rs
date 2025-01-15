use db_ext_macro::query_group;

use expect_test::expect;

mod logger_db;
use logger_db::LoggerDb;
use salsa::plumbing::{AsId, FromId};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct InternedStringId(salsa::Id);

impl AsId for InternedStringId {
    fn as_id(&self) -> salsa::Id {
        self.0
    }
}

impl FromId for InternedStringId {
    fn from_id(id: salsa::Id) -> Self {
        InternedStringId(id)
    }
}

#[salsa::interned_sans_lifetime(id = InternedStringId)]
pub struct InternedString {
    data: String,
}

#[query_group]
pub trait InternedDB: salsa::Database {
    #[db_ext_macro::interned]
    fn intern_string(&self, data: String) -> InternedStringId;

    fn interned_len(&self, id: InternedStringId) -> usize;
}

fn interned_len(db: &dyn InternedDB, id: InternedStringId) -> usize {
    db.lookup_intern_string(id).len()
}

#[test]
fn intern_round_trip() {
    let db = LoggerDb::default();

    let id = db.intern_string(String::from("Hello, world!"));
    let s = db.lookup_intern_string(id);

    assert_eq!(s.len(), 13);
    db.assert_logs(expect![[r#"[]"#]]);
}

#[test]
fn intern_with_query() {
    let db = LoggerDb::default();

    let id = db.intern_string(String::from("Hello, world!"));
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
