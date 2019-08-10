// schema.rs
// :PROPERTIES:
// :header-args: :tangle src/schema.rs
// :END:

// [[file:~/Workspace/Programming/gosh-rs/database/database.note::*schema.rs][schema.rs:1]]
table! {
    molecules {
        id -> Integer,
        name -> Nullable<Text>,
        lattice -> Nullable<Blob>,
    }
}

table! {
    /// Chemical models for calculating molecular properties.
    chemical_models {
        id -> Integer,
        name -> Nullable<Text>,
    }
}

table! {
    /// Atoms related properties calculated using various models.
    atom_properties(model_id, atom_id) {
        model_id -> Integer,
        atom_id -> Integer,
        force -> Nullable<Blob>,
        dipole -> Nullable<Blob>,
    }
}

table! {
    /// Atoms in molecule.
    atoms {
        id -> Integer,
        molecule_id -> Integer,
        element -> Integer,
        position -> Blob,
        tag -> Nullable<Text>,
        mass -> Nullable<Double>,
    }
}

table! {
    /// Molecular properties calculated using various models.
    molecule_properties(model_id, molecule_id) {
        model_id -> Integer,
        molecule_id -> Integer,
        energy -> Nullable<Double>,
    }
}
// schema.rs:1 ends here
