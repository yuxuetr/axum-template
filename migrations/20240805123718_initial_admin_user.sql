-- Add migration script here
-- create admin user
INSERT INTO users (username, password) VALUES ('superman', '$argon2id$v=19$m=19456,t=2,p=1$DHb3PSYPGgh4iugQcYCqBg$pfUnimzEo0EpNBVTnQ2SBPGu/nL0jpg0jhDOZEx0qvQ');

-- assign admin role to admin user
INSERT INTO user_roles (user_id, role_id, created_at, updated_at)
VALUES ((SELECT id FROM users WHERE username = 'superman'), 3, NOW(), NOW());

-- assign admin permission to admin role
INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
VALUES ((SELECT id FROM users WHERE username = 'superman'), 1, NOW(), NOW());
INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
VALUES ((SELECT id FROM users WHERE username = 'superman'), 2, NOW(), NOW());
INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
VALUES ((SELECT id FROM users WHERE username = 'superman'), 3, NOW(), NOW());
INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
VALUES ((SELECT id FROM users WHERE username = 'superman'), 4, NOW(), NOW());
INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
VALUES ((SELECT id FROM users WHERE username = 'superman'), 5, NOW(), NOW());
INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
VALUES ((SELECT id FROM users WHERE username = 'superman'), 6, NOW(), NOW());
INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
VALUES ((SELECT id FROM users WHERE username = 'superman'), 7, NOW(), NOW());
INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
VALUES ((SELECT id FROM users WHERE username = 'superman'), 8, NOW(), NOW());
