CREATE TABLE IF NOT EXISTS "rustperms_group" (
    groupname TEXT PRIMARY KEY,
    weight INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS "rustperms_group_groupname_idx" ON "rustperms_group" (groupname);

CREATE TABLE IF NOT EXISTS "rustperms_group_permissions" (
    groupname TEXT NOT NULL REFERENCES "rustperms_group" (groupname) ON DELETE CASCADE,
    permission TEXT,
    enabled BOOL DEFAULT NULL,
    PRIMARY KEY (groupname, permission)
);
CREATE INDEX IF NOT EXISTS "rustperms_group_permissions_groupname_idx" ON "rustperms_group_permissions" (groupname);
CREATE INDEX IF NOT EXISTS "rustperms_group_permissions_permission_idx" ON "rustperms_group_permissions" (permission);

CREATE TABLE IF NOT EXISTS "rustperms_group_relations" (
    groupname TEXT NOT NULL REFERENCES "rustperms_group" (groupname) ON DELETE CASCADE,
    parent_groupname TEXT NOT NULL REFERENCES "rustperms_group" (groupname) ON DELETE CASCADE,
    PRIMARY KEY (groupname, parent_groupname)
);
CREATE INDEX IF NOT EXISTS "rustperms_group_relations_groupname_idx" ON "rustperms_group_relations" (groupname);
CREATE INDEX IF NOT EXISTS "rustperms_group_relations_parent_idx" ON "rustperms_group_relations" (parent_groupname);

CREATE TABLE IF NOT EXISTS "rustperms_user_groups" (
    username TEXT NOT NULL REFERENCES "rustperms_user" (username) ON DELETE CASCADE,
    groupname TEXT NOT NULL REFERENCES "rustperms_group" (groupname) ON DELETE CASCADE,
    PRIMARY KEY (username, groupname)
);
CREATE INDEX IF NOT EXISTS "rustperms_user_groups_username_idx" ON "rustperms_user_groups" (username);
CREATE INDEX IF NOT EXISTS "rustperms_user_groups_groupname_idx" ON "rustperms_user_groups" (groupname);