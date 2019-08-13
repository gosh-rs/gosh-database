use crate::common::*;
use crate::schema::*;
use crate::*;

pub trait Checkpoint
where
    Self: Clone + serde::Serialize + serde::de::DeserializeOwned,
{
    /// Return a key associated with a group of checkpoints.
    fn checkpoint_key(&self) -> String;

    /// Set a checkpoint
    fn checkpoint(&self, db: &DbConnection) -> Result<()> {
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
            .with_context(|_| {
                format!(
                    "Failed to save checkpoint\n chk key: {}\n db source: {}",
                    ckpt_key,
                    db.database_url()
                )
            })?;

        Ok(())
    }

    /// Restore state from the latest checkpoint.
    fn restore_from_latest(&mut self, db: &DbConnection) -> Result<()> {
        self.restart_from_checkpoint(db, -1)
    }

    /// Restore state from the specified checkpoint `n` (ordered by create
    /// time).
    fn restart_from_checkpoint(&mut self, db: &DbConnection, n: i32) -> Result<()> {
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

        let x = bincode::deserialize(&encoded).with_context(|_| {
            format!(
                "Failed to deserialize from data for checkpoint: {}/{}",
                ckpt_key, n
            )
        })?;
        self.clone_from(&x);
        Ok(())
    }
}

impl Checkpoint for gosh_models::ModelProperties {
    fn checkpoint_key(&self) -> String {
        "DEFAULT-MP-CKPT".into()
    }
}

impl Checkpoint for gchemol::Molecule {
    fn checkpoint_key(&self) -> String {
        format!("CKPT-MOLE-{}", self.title())
    }
}
