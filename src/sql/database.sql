DROP DATABASE IF EXISTS bank;
CREATE USER IF NOT EXISTS accountant;
CREATE DATABASE bank;
GRANT ALL ON DATABASE bank TO accountant;
use bank;
