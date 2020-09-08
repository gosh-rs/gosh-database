// [[file:../database.note::*imports][imports:1]]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate derivative;

use std::sync::{Arc, Mutex, MutexGuard};

use diesel::prelude::*;
// imports:1 ends here

// [[file:../database.note::*mods][mods:1]]
mod checkpoint;
mod collection;
mod core;

pub mod prelude {
    pub use crate::checkpoint::*;
    pub use crate::collection::*;
}

pub(crate) mod schema;
// mods:1 ends here

// [[file:../database.note::*base][base:1]]
use gosh_core::*;

use gut::prelude::*;

embed_migrations!();

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct DbConnection {
    database_url: String,
    #[derivative(Debug="ignore")]
    connection: Arc<Mutex<SqliteConnection>>,
}

impl DbConnection {
    /// Eastablish connection to database specified using env var
    /// `GOSH_DATABASE_URL`.
    pub fn establish() -> Result<DbConnection> {
        // read vars from .env file
        dotenv::dotenv().ok();

        let database_url = std::env::var("GOSH_DATABASE_URL")
            .with_context(|| format!("GOSH_DATABASE_URL var not set"))?;
        debug!("Database: {}", database_url);

        Self::connect(&database_url)
    }

    /// Connect to database specified using `database_url`.
    pub fn connect(database_url: &str) -> Result<DbConnection> {
        // diesel accept &str, not Path
        let conn = SqliteConnection::establish(database_url)?;

        // see: https://sqlite.org/faq.html#q19
        //
        // With synchronous OFF, SQLite continues without syncing as soon as
        // it has handed data off to the operating system. If the application
        // running SQLite crashes, the data will be safe, but the database might
        // become corrupted if the operating system crashes or the computer
        // loses power before that data has been written to the disk surface. On
        // the other hand, commits can be orders of magnitude faster with
        // synchronous OFF.
        conn.execute("PRAGMA synchronous = OFF")?;

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
