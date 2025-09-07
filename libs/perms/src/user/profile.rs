use rustperms::{api::util::sharded_group_update_perms, prelude::*};
use uuid::Uuid;
use crate::{groups::*, rule, user::IntoKey};





rule!(PROFILE_EDIT_PERM, "user.profile.edit");
rule!(MINIPROFILE_EDIT_PERM, "user.miniprofile.edit");



rule!(PROFILE_VIEW_PERM, "user.profile.view");


