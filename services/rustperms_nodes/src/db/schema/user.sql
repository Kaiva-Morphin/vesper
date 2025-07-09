CREATE TABLE IF NOT EXISTS "rustperms_user" (
    user_uid TEXT PRIMARY KEY
);
CREATE INDEX IF NOT EXISTS "rustperms_user_user_uid_idx" ON "rustperms_user" (user_uid);

CREATE TABLE IF NOT EXISTS "rustperms_user_permissions" (
    user_uid TEXT NOT NULL REFERENCES "rustperms_user" (user_uid) ON DELETE CASCADE,
    permission TEXT,
    enabled BOOL DEFAULT NULL,
    PRIMARY KEY (user_uid, permission)
);
CREATE INDEX IF NOT EXISTS "rustperms_user_permissions_user_uid_idx" ON "rustperms_user_permissions" (user_uid);
CREATE INDEX IF NOT EXISTS "rustperms_user_permissions_permission_idx" ON "rustperms_user_permissions" (permission);