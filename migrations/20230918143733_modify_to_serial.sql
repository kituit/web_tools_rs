-- Add migration script here
DROP TABLE IF EXISTS Urls;

CREATE TABLE Urls (
    id SERIAL PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    redirect VARCHAR(10) NOT NULL UNIQUE
);