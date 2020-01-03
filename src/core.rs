// pub

// [[file:~/Workspace/Programming/gosh-rs/database/database.note::*pub][pub:1]]
use crate::schema::*;
use crate::*;
use guts::prelude::*;

use gosh_models::ModelProperties;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name = "models"]
pub struct Model {
    pub id: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name = "molecules"]
pub struct Molecule {
    pub id: i32,
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Molecule, foreign_key = "molecule_id")]
#[belongs_to(Model, foreign_key = "model_id")]
#[table_name = "properties"]
#[primary_key(model_id, molecule_id)]
pub struct Properties {
    pub model_id: i32,
    pub molecule_id: i32,
    pub data: Vec<u8>,
}

// #[derive(Queryable, Debug, Clone)]
// pub struct NewModel {
//     pub name: String,
// }

// #[derive(Deserialize, Debug, Insertable)]
// #[table_name = "properties"]
// pub struct NewProperties {
//     pub model_id: i32,
//     pub molecule_id: i32,
//     pub data: Vec<u8>,
// }

// #[derive(Debug, Insertable)]
// #[table_name = "molecules"]
// pub struct NewMolecule {
//     pub name: String,
//     pub data: Vec<u8>,
// }

pub fn save_model_results(mp: &ModelProperties, db: &DbConnection) -> Result<()> {
    let mol = mp.get_molecule().expect("model properties has no structure!");

    // save molecule record
    let conn = db.get();

    // insert a new properties record
    {
        use crate::schema::properties::dsl::*;

        let row = (
            model_id.eq(1),
            molecule_id.eq(1),
            data.eq({ bincode::serialize(&mol).unwrap() }),
        );
        diesel::insert_into(properties)
            .values(&row)
            .execute(&*conn)?;
    }

    // let row = NewProperties {
    //     data: { bincode::serialize(&mp).unwrap() },
    // };

    // diesel::insert_into(properties::table)
    //     .values(&row)
    //     .execute(&*conn)?;

    Ok(())
}
// pub:1 ends here
