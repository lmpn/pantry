-- Add migration script here
CREATE TABLE item (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    quantity DOUBLE PRECISION NOT NULL,
    state INTEGER NOT NULL CHECK (state IN (0, 1, 2))
);
