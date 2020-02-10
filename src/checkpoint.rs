use gosh_core::*;

use crate::schema::*;
use crate::*;

use gut::prelude::*;

pub trait Checkpoint
where
    Self: Clone + serde::Serialize + serde::de::DeserializeOwned,
{
    /// Return a key associated with a group of checkpoints.
    fn checkpoint_key(&self) -> String;

    /// Set a checkpoint
    fn commit_checkpoint(&self, db: &DbConnection) -> Result<()> {
        use crate::schema::checkpoints::dsl::*;

        let ckpt_key = &self.checkpoint_key();
        let conn = db.get();

        let row = (
            key.eq(ckpt_key),
            data.eq({ bincode::serialize(&self).unwrap() }),
        );

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

    /// Return the number of available checkpoints in database.
    #[cfg(feature="adhoc")]
    fn get_number_of_checkpoints(&self, db: &DbConnection) -> Result<i64> {
        use crate::schema::checkpoints::dsl::*;

        let conn = db.get();
        let ckpt_key = self.checkpoint_key();
        let count = checkpoints
            .filter(key.eq(&ckpt_key))
            .count()
            .get_result(&*conn)?;
        Ok(count)
    }

    /// Restore state from the specified checkpoint `n` (ordered by create
    /// time).
    fn restore_from_checkpoint_n(&mut self, db: &DbConnection, n: i32) -> Result<()> {
        use crate::schema::checkpoints::dsl::*;

        let conn = db.get();
        let ckpt_key = self.checkpoint_key();
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
        let encoded: Vec<u8> = checkpoints
            .filter(id.eq(&ckpts[k]))
            .select(data)
            .first(&*conn)?;

        let x = bincode::deserialize(&encoded).with_context(|| {
            format!(
                "Failed to deserialize from data for checkpoint: {}/{}",
                ckpt_key, n
            )
        })?;
        self.clone_from(&x);
        Ok(())
    }
}

#[cfg(feature = "adhoc")]
impl Checkpoint for gosh_models::ModelProperties {
    fn checkpoint_key(&self) -> String {
        "DEFAULT-MP-CKPT".into()
    }
}

#[cfg(feature = "adhoc")]
impl Checkpoint for gchemol::Molecule {
    fn checkpoint_key(&self) -> String {
        format!("CKPT-MOLE-{}", self.title())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct TestObject {
        data: f64,
    }

    impl Checkpoint for TestObject {
        /// Return an unique name as the container for your data.
        fn checkpoint_key(&self) -> String {
            "test-obj-chk".into()
        }
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
        #[cfg(feature="adhoc")]
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
