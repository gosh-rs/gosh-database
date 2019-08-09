// models.rs
// :PROPERTIES:
// :header-args: :tangle src/models.rs
// :END:

// [[file:~/Workspace/Programming/gosh-rs/database/database.note::*models.rs][models.rs:1]]
use crate::schema::*;

#[derive(Queryable, Debug, Clone)]
pub struct Structure {
    pub id: i32,
}

#[derive(Deserialize, Debug, Insertable)]
#[table_name = "properties"]
pub struct NewProperties {
    pub molecule_id: i32,
    pub energy: Option<f64>,
    pub chemical_model: String,
}
// models.rs:1 ends here
