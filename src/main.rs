// main.rs
// :PROPERTIES:
// :header-args: :tangle src/main.rs
// :END:

// [[file:~/Workspace/Programming/gosh-rs/database/database.note::*main.rs][main.rs:1]]
#[macro_use]
extern crate diesel_migrations;

use diesel::prelude::*;
use gosh_db::models::*;

use quicli::prelude::*;
type Result<T> = ::std::result::Result<T, Error>;

fn main() -> Result<()> {
    use gosh_db::schema::molecule_properties::dsl::*;

    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .with_context(|e| format!("DATABASE_URL var not set: {}", e))?;
    let db_conn = SqliteConnection::establish(&database_url)?;

    // create sql database if not exists?
    embed_migrations!();

    // This will run the necessary migrations.
    embedded_migrations::run(&db_conn)?;

    // create properties
    let new_prop = NewProperties {
        model_id: 0,
        molecule_id: 1,
        energy: None
    };

    diesel::insert_into(molecule_properties)
        .values(&new_prop)
        .execute(&db_conn)?;

    Ok(())
}
// main.rs:1 ends here
