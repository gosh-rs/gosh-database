use crate::common::*;
use crate::schema::*;
use crate::*;

pub trait Checkpointing {
    /// Load checkpointed data associated with `k`.
    fn load_checkpoint<T>(&self, k: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned;

    /// Insert `value` into checkpoint database.
    fn save_checkpoint<T: ?Sized>(&self, k: &str, value: &T) -> Result<()>
    where
        T: serde::Serialize;
}

impl Checkpointing for DbConnection {
    /// Load checkpointed data associated with `k`.
    fn load_checkpoint<T>(&self, k: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        use crate::schema::checkpoints::dsl::*;

        let conn = self.get();
        let encoded: Vec<u8> = checkpoints.filter(key.eq(k)).select(data).first(&*conn)?;
        let d = bincode::deserialize(&encoded[..]).unwrap();
        Ok(d)
    }

    /// Insert `value` into checkpoint database.
    fn save_checkpoint<T: ?Sized>(&self, k: &str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        use crate::schema::checkpoints::dsl::*;

        let conn = self.get();
        let row = (key.eq(k), data.eq({ bincode::serialize(value).unwrap() }));
        diesel::insert_into(checkpoints)
            .values(&row)
            .execute(&*conn)
            .with_context(|_| {
                format!(
                    "Failed to save checkpoint\n chk key: {}\n db source: {}",
                    k,
                    self.database_url()
                )
            })?;

        Ok(())
    }
}
