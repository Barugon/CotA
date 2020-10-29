pub const ADVENTURER_SKILLS: &str = include_str!("res/adventurer_skills.csv");
pub const PRODUCER_SKILLS: &str = include_str!("res/producer_skills.csv");
pub const LEVEL_EXP_VALUES: [i64; 200] = include!("res/level_exp_values");
pub const SKILL_EXP_VALUES: [i64; 200] = include!("res/skill_exp_values");

// File menu.
pub const LOG_FOLDER_ID: i64 = 0;
pub const QUIT_ID: i64 = 1;

// View menu.
pub const REFRESH_ID: i64 = 0;
pub const RESISTS_ID: i64 = 1;
pub const FILTER_ID: i64 = 2;
pub const RESET_ID: i64 = 3;

// Help menu.
pub const ABOUT_ID: i64 = 0;

// Tabs.
pub const STATS_IDX: i64 = 0;
pub const PORTALS_IDX: i64 = 1;
pub const _EXPERIENCE_IDX: i64 = 2;
pub const _OFFLINE_IDX: i64 = 3;
