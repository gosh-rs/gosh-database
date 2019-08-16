// imports

// [[file:~/Workspace/Programming/gosh-rs/database/database.note::*imports][imports:1]]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate diesel_migrations;

use std::sync::{Arc, Mutex, MutexGuard};

use diesel::prelude::*;
// imports:1 ends here

// mods

// [[file:~/Workspace/Programming/gosh-rs/database/database.note::*mods][mods:1]]
mod checkpoint;
mod collection;
mod core;

pub mod prelude {
    pub use crate::checkpoint::*;
    pub use crate::collection::*;
}

pub(crate) mod schema;

pub(crate) mod common {
    pub use quicli::prelude::*;
    pub type Result<T> = ::std::result::Result<T, Error>;
}
// mods:1 ends here

// base

// [[file:~/Workspace/Programming/gosh-rs/database/database.note::*base][base:1]]
use crate::common::*;

embed_migrations!();

pub struct DbConnection {
    database_url: String,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl DbConnection {
    /// Eastablish connection to database specified using env var
    /// `GOSH_DATABASE_URL`.
    pub fn establish() -> Result<DbConnection> {
        // read vars from .env file
        dotenv::dotenv().ok();

        let database_url = std::env::var("GOSH_DATABASE_URL")
            .with_context(|e| format!("GOSH_DATABASE_URL var not set: {}", e))?;
        debug!("Database: {}", database_url);

        let conn = SqliteConnection::establish(&database_url)?;
        let conn = Arc::new(Mutex::new(conn));

        let db = DbConnection {
            database_url: database_url.into(),
            connection: conn.clone(),
        };

        db.migrate()?;

        Ok(db)
    }

    /// Show database url.
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub(crate) fn get(&self) -> MutexGuard<'_, SqliteConnection> {
        self.connection.lock().expect("cannot lock db connection!")
    }

    // for schema migrations, sql tables initialization
    fn migrate(&self) -> Result<()> {
        let conn = self.get();

        // This will run the necessary migrations.
        embedded_migrations::run(&*conn)?;

        Ok(())
    }
}
// base:1 ends here
