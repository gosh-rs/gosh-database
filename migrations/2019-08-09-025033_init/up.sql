CREATE TABLE molecule_properties (
       model_id INTEGER NOT NULL,
       molecule_id INTEGER NOT NULL,
       energy DOUBLE,
       PRIMARY KEY (model_id, molecule_id)
);

CREATE TABLE atom_properties (
       model_id INTEGER NOT NULL,
       atom_id INTEGER NOT NULL,
       force BLOB,
       dipole BLOB,
       PRIMARY KEY (model_id, atom_id)
);

CREATE TABLE chemical_models (
       id INTEGER PRIMARY KEY NOT NULL,
       name TEXT NOT NULL
);

CREATE TABLE atoms (
       id INTEGER PRIMARY KEY NOT NULL,
       molecule_id INTEGER NOT NULL,
       element INTEGER NOT NULL,
       position BLOB NOT NULL,
       tag TEXT,
       mass DOUBLE
);

CREATE TABLE molecules (
       id INTEGER PRIMARY KEY NOT NULL,
       name TEXT,
       lattice BLOB
);
