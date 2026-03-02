-- Add UNIQUE constraint to username column
ALTER TABLE users ADD CONSTRAINT users_username_unique UNIQUE (username);
