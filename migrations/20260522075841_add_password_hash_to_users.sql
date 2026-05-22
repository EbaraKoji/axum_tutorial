-- Add migration script here
ALTER TABLE USERS ADD COLUMN password_hash VARCHAR(255) NOT NULL;