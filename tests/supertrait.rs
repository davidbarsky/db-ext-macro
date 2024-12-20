#[salsa::db]
pub trait SourceDb: salsa::Database {
    /// Text of the file.
    fn file_text(&self, id: usize) -> String;
}

#[db_ext_macro::query_group]
pub trait RootDb: SourceDb {
    fn parse(&self, id: usize) -> String;
}

fn parse(db: &dyn RootDb, id: usize) -> String {
    db.file_text(id);

    todo!()
}
