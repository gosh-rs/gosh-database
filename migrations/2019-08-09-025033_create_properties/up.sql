-- Your SQL goes here
CREATE TABLE properties (
      id INTEGER PRIMARY KEY,
      molecule_id INTEGER NOT NULL,
      energy DOUBLE,
      chemical_model TEXT NOT NULL
)
