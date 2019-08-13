use crate::common::*;
use crate::schema::*;
use crate::*;

pub trait Checkpoint
where
    Self: Clone + serde::Serialize + serde::de::DeserializeOwned,
{
    /// Set a checkpoint
    fn checkpoint(&self, db: &DbConnection, identifier: &str) -> Result<()> {
        use crate::schema::checkpoints::dsl::*;

        let conn = db.get();
        let row = (
            key.eq(identifier),
            data.eq({ bincode::serialize(&self).unwrap() }),
        );
        diesel::insert_into(checkpoints)
            .values(&row)
            .execute(&*conn)
            .with_context(|_| {
                format!(
                    "Failed to save checkpoint\n chk key: {}\n db source: {}",
                    identifier,
                    db.database_url()
                )
            })?;

        Ok(())
    }

    /// Load from checkpointed state.
    fn restore_from_checkpoint(&mut self, db: &DbConnection, identifier: &str) -> Result<()> {
        use crate::schema::checkpoints::dsl::*;

        let conn = db.get();
        let encoded: Vec<u8> = checkpoints
            .filter(key.eq(identifier))
            .select(data)
            .first(&*conn)?;
        let x = bincode::deserialize(&encoded[..]).unwrap();
        self.clone_from(&x);
        Ok(())
    }
}

impl Checkpoint for gosh_models::ModelProperties {}
impl Checkpoint for gchemol::Molecule {}
