// tests
// :PROPERTIES:
// :header-args: :tangle tests/test.rs
// :END:

// [[file:~/Workspace/Programming/gosh-rs/database/database.note::*tests][tests:1]]
use gosh_core::*;
use gosh_db::prelude::*;
use gosh_db::DbConnection;

use guts::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Test {
    data: f64,
}

impl Collection for Test {
    /// Return an unique name as the container for your data.
    fn collection_name() -> String {
        "test-checkpoint-collection-tmp".into()
    }
}

impl Checkpoint for Test {
    /// Return an unique name as the container for your data.
    fn checkpoint_key(&self) -> String {
        "test-obj-chk".into()
    }
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
