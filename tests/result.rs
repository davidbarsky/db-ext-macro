use query_group::query_group;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error;

#[query_group]
pub trait ResultDatabase: salsa::Database {
    #[salsa::input]
    fn input_string(&self) -> String;

    // unadorned query
    fn length(&self, key: ()) -> Result<usize, Error>;

    #[salsa::cycle(recover)]
    fn length2(&self, key: ()) -> Result<usize, Error>;
}

fn length(db: &dyn ResultDatabase, key: ()) -> Result<usize, Error> {
    let _ = key;
    Ok(db.input_string().len())
}

fn length2(db: &dyn ResultDatabase, key: ()) -> Result<usize, Error> {
    let _ = key;
    Ok(db.input_string().len())
}

fn recover(
    _db: &dyn ResultDatabase,
    _cycle: &salsa::Cycle,
    _input: ResultDatabaseData, // TODO: figure out how to not rely on this generated struct?
    _key: (),
) -> Result<usize, Error> {
    Ok(0)
}
