use gosh_core::*;

use crate::schema::*;
use crate::*;

use gut::prelude::*;

pub trait Checkpoint
where
    Self: Clone + serde::Serialize + serde::de::DeserializeOwned,
{
    // Return a key associated with a group of checkpoints.
    // const CKPT_KEY: &'static str;

    /// Return an unique name as the container for your data.
    fn checkpoint_name() -> String {
        format!("{}.ckpt", std::any::type_name::<Self>())
    }

    /// Load from the specified checkpoint `n` (ordered by create time).
    fn from_checkpoint_n(db: &DbConnection, n: i32) -> Result<Self> {
        use crate::schema::checkpoints::dsl::*;

        let conn = db.get();
        let ckpt_key = Self::checkpoint_name();
        let ckpts: Vec<i32> = checkpoints
            .filter(key.eq(&ckpt_key))
            .select(id)
            .order(ctime.asc())
            .load(&*conn)?;
        let nckpts = ckpts.len();
        info!("Found {} checkpoints with key {}", nckpts, &ckpt_key);

        // Allow negative index into the list.
        let k = if n < 0 { nckpts as i32 + n } else { n } as usize;
        // Avoid panic when n is invalid.
        if k >= nckpts {
            bail!("specified checkpoint {} is out of range.", n);
        }

        // Get encoded data.
        let encoded: Vec<u8> = checkpoints.filter(id.eq(&ckpts[k])).select(data).first(&*conn)?;

        let x = bincode::deserialize(&encoded)
            .with_context(|| format!("Failed to deserialize from data for checkpoint: {}/{}", ckpt_key, n))?;
        Ok(x)
    }

    /// Set a checkpoint
    fn commit_checkpoint(&self, db: &DbConnection) -> Result<()> {
        use crate::schema::checkpoints::dsl::*;

        let ckpt_key = Self::checkpoint_name();
        let conn = db.get();

        let row = (key.eq(&ckpt_key), data.eq({ bincode::serialize(&self).unwrap() }));

        diesel::insert_into(checkpoints)
            .values(&row)
            .execute(&*conn)
            .with_context(|| {
                format!(
                    "Failed to save checkpoint\n chk key: {}\n db source: {}",
                    ckpt_key,
                    db.database_url()
                )
            })?;

        Ok(())
    }

    /// Restore state from the latest checkpoint.
    fn restore_from_checkpoint(&mut self, db: &DbConnection) -> Result<()> {
        self.restore_from_checkpoint_n(db, -1)
    }

    /// List available checkpoints in `db`.
    #[cfg(feature = "adhoc")]
    fn list_checkpoints(db: &DbConnection) -> Result<()> {
        use crate::schema::checkpoints::dsl::*;

        let conn = db.get();
        let ckpt_key = Self::checkpoint_name();
        let ckpts: Vec<(i32, String, String)> = checkpoints
            .filter(key.eq(&ckpt_key))
            .select((id, key, ctime))
            .order(ctime.asc())
            .load(&*conn)?;
        let nckpts = ckpts.len();
        info!("Found {} checkpoints with key {}", nckpts, &ckpt_key);

        println!("{:^5}\t{:^}", "slot", "create time");
        for (i, (_, _, t)) in ckpts.iter().enumerate() {
            println!("{:^5}\t{:^}", i, t);
        }

        Ok(())
    }

    /// Return the number of available checkpoints in database.
    #[cfg(feature = "adhoc")]
    fn get_number_of_checkpoints(&self, db: &DbConnection) -> Result<i64> {
        use crate::schema::checkpoints::dsl::*;

        let conn = db.get();
        let ckpt_key = Self::checkpoint_name();
        let count = checkpoints.filter(key.eq(&ckpt_key)).count().get_result(&*conn)?;
        Ok(count)
    }

    /// Restore state from the specified checkpoint `n` (ordered by create
    /// time).
    fn restore_from_checkpoint_n(&mut self, db: &DbConnection, n: i32) -> Result<()> {
        let x = Self::from_checkpoint_n(db, n)?;
        self.clone_from(&x);
        Ok(())
    }
}

// #[cfg(feature = "adhoc")]
// impl Checkpoint for gosh_model::ModelProperties {
//     const CKPT_KEY: &'static str = "DEFAULT-MP-CKPT";
// }

// #[cfg(feature = "adhoc")]
// impl Checkpoint for gchemol::Molecule {
//     const CKPT_KEY: &'static str = "DEFAULT-MOL-CKPT";
// }

impl<T> Checkpoint for T where T: Clone + serde::Serialize + serde::de::DeserializeOwned {}

use gut::cli::*;
use std::path::{Path, PathBuf};

#[derive(StructOpt, Default, Clone, Debug)]
pub struct CheckpointDb {
    /// Path to a checkpoint file for resuming calculation later.
    #[structopt(long)]
    chk_file: Option<PathBuf>,

    /// Index of checkpoint frame to restore (0-base). The default is to restore
    /// from the lastest (--chk-slot=-1)
    #[structopt(long)]
    chk_slot: Option<i32>,

    // internal: database connection
    #[structopt(skip)]
    db_connection: Option<DbConnection>,
}

impl CheckpointDb {
    /// Construct `Checkpoint` from directory.
    ///
    /// # Arguments
    ///
    /// * d: root directory for checkpoint files
    ///
    pub fn new<P: AsRef<Path>>(d: P) -> Self {
        let mut chk = Self::default();
        chk.chk_file = Some(d.as_ref().to_path_buf());
        chk.create()
    }

    /// Construct with checkpoint slot `n`.
    pub fn slot(mut self, n: i32) -> Self {
        self.chk_slot = Some(n);
        self
    }

    /// Create missing db_connection field if `chk_file` is not None. Mainly for cmdline uses.
    pub fn create(&self) -> Self {
        if let Some(dbfile) = &self.chk_file {
            let url = format!("{}", dbfile.display());
            let dbc = DbConnection::connect(&url).expect("failed to connect to db src");
            let mut chk = self.clone();
            chk.db_connection = Some(dbc);
            chk
        } else {
            self.to_owned()
        }
    }
}

impl CheckpointDb {
    /// Restore `chain` from checkpoint. Return true if restored successfuly,
    /// false otherwise.
    pub fn restore<T: Checkpoint>(&self, data: &mut T) -> Result<bool> {
        // use resumed `data` from checkpoint if possible
        if let Some(db) = &self.db_connection {
            if let Some(n) = self.chk_slot {
                if let Err(e) = data.restore_from_checkpoint_n(db, n) {
                    warn!("failed to restore from checkpoint");
                    dbg!(e);
                }
            } else {
                if let Err(e) = data.restore_from_checkpoint(db) {
                    warn!("failed to restore from checkpoint");
                    dbg!(e);
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Return checkpointed `T`
    pub fn restored<T: Checkpoint>(&self) -> Result<T> {
        let n = self.chk_slot.unwrap_or(-1);
        let db = self.db_connection.as_ref().expect("no db connection");

        let x = T::from_checkpoint_n(db, n)?;
        Ok(x)
    }

    /// Commit a checkpoint into database. Return true if committed, false
    /// otherwise.
    pub fn commit<T: Checkpoint>(&self, data: &T) -> Result<bool> {
        if let Some(db) = &self.db_connection {
            data.commit_checkpoint(db)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List available checkpoints in database.
    #[cfg(feature = "adhoc")]
    pub fn list<T: Checkpoint>(&self) -> Result<bool> {
        if let Some(db) = &self.db_connection {
            T::list_checkpoints(db)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct TestObject {
        data: f64,
    }

    #[test]
    fn test_checkpoint() -> Result<()> {
        // setup database in a temp directory
        let tdir = tempfile::tempdir()?;
        let tmpdb = tdir.path().join("test.sqlite");
        let url = format!("{}", tmpdb.display());
        let db = DbConnection::connect(&url)?;

        // commit checkpoint
        let mut x = TestObject { data: -12.0 };
        x.commit_checkpoint(&db)?;
        // commit a new checkpoint
        x.data = 1.0;
        x.commit_checkpoint(&db)?;
        // commit a new checkpoint again
        x.data = 0.0;
        x.commit_checkpoint(&db)?;
        assert_eq!(x.data, 0.0);

        // restore from checkpoint
        #[cfg(feature = "adhoc")]
        assert_eq!(x.get_number_of_checkpoints(&db)?, 3);
        x.restore_from_checkpoint(&db)?;
        assert_eq!(x.data, 0.0);
        x.restore_from_checkpoint_n(&db, 0)?;
        assert_eq!(x.data, -12.0);
        x.restore_from_checkpoint_n(&db, 1)?;
        assert_eq!(x.data, 1.0);

        Ok(())
    }
}
