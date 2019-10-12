use crate::common::*;
use crate::schema::*;
use crate::*;

pub trait Collection
where
    Self: serde::Serialize + serde::de::DeserializeOwned,
{
    /// Return an unique name as the container for your data.
    fn collection_name() -> String;

    /// Put self into collection.
    fn put_into_collection(&self, db: &DbConnection, new_key: &str) -> Result<()> {
        use crate::schema::kvstore::dsl::*;

        let conn = db.get();
        let cname = &Self::collection_name();

        let row = (
            collection.eq(cname),
            key.eq(new_key),
            data.eq({ bincode::serialize(&self).unwrap() }),
        );

        diesel::insert_into(kvstore)
            .values(&row)
            .execute(&*conn)
            .with_context(|_| {
                format!(
                    "Failed to put data into collection {} with key {}\n db source: {}",
                    cname,
                    new_key,
                    db.database_url()
                )
            })?;

        Ok(())
    }

    /// Return the object in this collection by `key`.
    fn get_from_collection(db: &DbConnection, obj_key: &str) -> Result<Self> {
        use crate::schema::kvstore::dsl::*;

        let conn = db.get();
        let cname = &Self::collection_name();
        let encoded: Vec<u8> = kvstore
            .filter(collection.eq(&cname))
            .filter(key.eq(&obj_key))
            .select(data)
            .first(&*conn)?;

        let x = bincode::deserialize(&encoded)
            .with_context(|_| format!("Failed to deserialize data for {}/{}", cname, obj_key))?;

        Ok(x)
    }

    /// Delete the object in this collection by `key`.
    fn del_from_collection(db: &DbConnection, obj_key: &str) -> Result<()> {
        use crate::schema::kvstore::dsl::*;

        let conn = db.get();
        let cname = &Self::collection_name();
        diesel::delete(
            kvstore
                .filter(collection.eq(&cname))
                .filter(key.eq(&obj_key)),
        )
        .execute(&*conn)?;

        Ok(())
    }

    /// Remove all objects in this collection.
    fn remove_collection(db: &DbConnection) -> Result<()> {
        use crate::schema::kvstore::dsl::*;

        let conn = db.get();
        let cname = &Self::collection_name();
        diesel::delete(kvstore.filter(collection.eq(&cname))).execute(&*conn)?;
        Ok(())
    }

    /// List all items in the collection.
    fn list_collection(db: &DbConnection) -> Result<Vec<Self>> {
        use crate::schema::kvstore::dsl::*;

        let conn = db.get();
        let cname = &Self::collection_name();
        let list: Vec<(String, Vec<u8>)> = kvstore
            .filter(collection.eq(&cname))
            .select((key, data))
            .load(&*conn)?;

        let mut items = vec![];
        for (obj_key, encoded) in list {
            let x = bincode::deserialize(&encoded).with_context(|_| {
                format!("Failed to deserialize data for {}/{}", cname, obj_key)
            })?;
            items.push(x);
        }
        Ok(items)
    }

    /// Return the number of items in collection.
    fn collection_size(db: &DbConnection) -> Result<i64> {
        use crate::schema::kvstore::dsl::*;

        let conn = db.get();
        let cname = &Self::collection_name();
        let count = kvstore
            .filter(collection.eq(&cname))
            .count()
            .get_result(&*conn)?;

        // conn.execute(&format!("DROP TABLE {}", "kvstore")).unwrap();
        // conn.execute("SELECT COUNT(*) FROM kvstore").unwrap();

        Ok(count)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct TestObject {
        data: f64,
    }

    impl Collection for TestObject {
        /// Return an unique name as the container for your data.
        fn collection_name() -> String {
            "t-obj-tmp".into()
        }
    }

    #[test]
    fn test_collection() -> Result<()> {
        // setup db in a temp directory
        let tdir = tempfile::tempdir()?;
        let tmpdb = tdir.path().join("test.sqlite");
        std::env::set_var("GOSH_DATABASE_URL", tmpdb);
        let db = DbConnection::establish().unwrap();

        let x = TestObject { data: -12.0 };

        x.put_into_collection(&db, "test1")?;

        TestObject::get_from_collection(&db, "test1")?;

        TestObject::del_from_collection(&db, "test1")?;

        TestObject::remove_collection(&db)?;

        let x = TestObject::list_collection(&db)?;
        assert!(x.is_empty());

        let x = TestObject { data: 12.0 };
        x.put_into_collection(&db, "test1")?;
        let x = TestObject::list_collection(&db)?;
        assert_eq!(1, x.len());

        Ok(())
    }
}
