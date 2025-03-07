
pub const REFRESH_TOKEN_LIFETIME: u64 = 60 * 60 * 24 * 60; // 60 days
pub const ACCESS_TOKEN_LIFETIME: u64 = 60 * 30; // 30 mins
pub const CRFS_TOKEN_LIFETIME: u64 = 60 * 2; // 2 mins
pub const TEMPORARY_USERDATA_TOKEN_LIFETIME: u64 = 60 * 2; // 2 mins

pub const MIN_LOGIN_LENGTH: usize = 4;
pub const MAX_LOGIN_LENGTH: usize = 32;
pub const MIN_PASSWORD_LENGTH: usize = 6;
pub const MAX_PASSWORD_LENGTH: usize = 64;

pub const MAX_LIVE_SESSIONS: usize = 5;
