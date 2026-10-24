mod app_home;
mod cache;

pub use app_home::*;
pub use cache::*;

pub const APP_HOME_ENV_VAR: &str = "TEAMY_DOMAINS_HOME";
pub const APP_HOME_DIR_NAME: &str = "teamy-domains";
pub const APP_CACHE_ENV_VAR: &str = "TEAMY_DOMAINS_CACHE";
pub const APP_CACHE_DIR_NAME: &str = "teamy-domains";
