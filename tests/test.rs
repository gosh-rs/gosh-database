// [[file:../database.note::*tests][tests:1]]
use gosh_core::*;
use gosh_database::prelude::*;
use gosh_database::DbConnection;

use gut::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Test {
    data: f64,
}

#[test]
fn test_checkpoint_and_collection() -> Result<()> {
    // setup db in a temp directory
    let tdir = tempfile::tempdir()?;
    let tmpdb = tdir.path().join("test.sqlite");
    std::env::set_var("GOSH_DATABASE_URL", tmpdb);
    let db = DbConnection::establish().unwrap();

    let x = Test { data: -12.0 };
    // save into collection
    x.put_into_collection(&db, "test1")?;
    // commit a checkpoint
    x.commit_checkpoint(&db)?;

    let mut x = Test { data: 12.0 };
    // restore data from checkpoint
    x.restore_from_checkpoint(&db)?;
    assert_eq!(x.data, -12.0);

    Ok(())
}
// tests:1 ends here
