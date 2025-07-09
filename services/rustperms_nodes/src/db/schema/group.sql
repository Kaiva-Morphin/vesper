CREATE TABLE IF NOT EXISTS "rustperms_group" (
    group_uid TEXT PRIMARY KEY,
    weight INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS "rustperms_group_group_uid_idx" ON "rustperms_group" (group_uid);

CREATE TABLE IF NOT EXISTS "rustperms_group_permissions" (
    group_uid TEXT NOT NULL REFERENCES "rustperms_group" (group_uid) ON DELETE CASCADE,
    permission TEXT,
    enabled BOOL DEFAULT NULL,
    PRIMARY KEY (group_uid, permission)
);
CREATE INDEX IF NOT EXISTS "rustperms_group_permissions_group_uid_idx" ON "rustperms_group_permissions" (group_uid);
CREATE INDEX IF NOT EXISTS "rustperms_group_permissions_permission_idx" ON "rustperms_group_permissions" (permission);

CREATE TABLE IF NOT EXISTS "rustperms_group_relations" (
    group_uid TEXT NOT NULL REFERENCES "rustperms_group" (group_uid) ON DELETE CASCADE,
    parent_group_uid TEXT NOT NULL REFERENCES "rustperms_group" (group_uid) ON DELETE CASCADE,
    PRIMARY KEY (group_uid, parent_group_uid)
);
CREATE INDEX IF NOT EXISTS "rustperms_group_relations_group_uid_idx" ON "rustperms_group_relations" (group_uid);
CREATE INDEX IF NOT EXISTS "rustperms_group_relations_parent_idx" ON "rustperms_group_relations" (parent_group_uid);

CREATE TABLE IF NOT EXISTS "rustperms_user_groups" (
    user_uid TEXT NOT NULL REFERENCES "rustperms_user" (user_uid) ON DELETE CASCADE,
    group_uid TEXT NOT NULL REFERENCES "rustperms_group" (group_uid) ON DELETE CASCADE,
    PRIMARY KEY (user_uid, group_uid)
);
CREATE INDEX IF NOT EXISTS "rustperms_user_groups_user_uid_idx" ON "rustperms_user_groups" (user_uid);
CREATE INDEX IF NOT EXISTS "rustperms_user_groups_group_uid_idx" ON "rustperms_user_groups" (group_uid);