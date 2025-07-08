CREATE TABLE IF NOT EXISTS "rustperms_user" (
    username TEXT PRIMARY KEY
);
CREATE INDEX IF NOT EXISTS "rustperms_user_username_idx" ON "rustperms_user" (username);

CREATE TABLE IF NOT EXISTS "rustperms_user_permissions" (
    username TEXT NOT NULL REFERENCES "rustperms_user" (username) ON DELETE CASCADE,
    permission TEXT,
    enabled BOOL DEFAULT NULL,
    PRIMARY KEY (username, permission)
);
CREATE INDEX IF NOT EXISTS "rustperms_user_permissions_username_idx" ON "rustperms_user_permissions" (username);
CREATE INDEX IF NOT EXISTS "rustperms_user_permissions_permission_idx" ON "rustperms_user_permissions" (permission);