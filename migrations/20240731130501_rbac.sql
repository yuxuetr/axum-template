-- Add migration script here
-- create `roles` table
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- create `permissions` table
CREATE TABLE permissions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- create `user_roles` table
CREATE TABLE user_roles (
    user_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

-- create `role_permissions` table
CREATE TABLE role_permissions (
    role_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- create `user_permissions` table
CREATE TABLE user_permissions (
    user_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, permission_id)
);

-- insert predefined roles
INSERT INTO roles (name) VALUES ('User');
INSERT INTO roles (name) VALUES ('Moderator');
INSERT INTO roles (name) VALUES ('Admin');

-- insert predefined permissions
INSERT INTO permissions (name) VALUES ('READ');
INSERT INTO permissions (name) VALUES ('WRITE');
INSERT INTO permissions (name) VALUES ('DELETE');
INSERT INTO permissions (name) VALUES ('MANAGE_PERMISSIONS');
INSERT INTO permissions (name) VALUES ('MANAGE_USERS');
INSERT INTO permissions (name) VALUES ('MANAGE_ROLES');
INSERT INTO permissions (name) VALUES ('VIEWRE_PORTS');
INSERT INTO permissions (name) VALUES ('EDIT_SETTINGS');

-- roles and permissions relationship
-- User role permissions
INSERT INTO role_permissions (role_id, permission_id) VALUES (1, 1); -- READ
INSERT INTO role_permissions (role_id, permission_id) VALUES (1, 2); -- WRITE
INSERT INTO role_permissions (role_id, permission_id) VALUES (1, 3); -- DELETE

-- Moderator role permissions
INSERT INTO role_permissions (role_id, permission_id) VALUES (2, 1); -- READ
INSERT INTO role_permissions (role_id, permission_id) VALUES (2, 2); -- WRITE
INSERT INTO role_permissions (role_id, permission_id) VALUES (2, 3); -- DELETE
INSERT INTO role_permissions (role_id, permission_id) VALUES (2, 4); -- MANAGEPERMISSIONS

-- Admin role permissions
INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 1); -- READ
INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 2); -- WRITE
INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 3); -- DELETE
INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 4); -- MANAGEPERMISSIONS
INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 5); -- MANAGEUSERS
INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 6); -- MANAGEROLES
INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 7); -- VIEWREPORTS
INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 8); -- EDITSETTINGS
