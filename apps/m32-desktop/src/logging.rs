use std::{env, error::Error};

use tracing::Level;

pub const LOG_LEVEL_ENV: &str = "M32_LOG";
pub const DEFAULT_LOG_LEVEL: Level = Level::INFO;

pub fn init() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let raw_level = env::var(LOG_LEVEL_ENV).ok();
    let (level, used_fallback) = resolve_level(raw_level.as_deref());

    if used_fallback {
        eprintln!("M32 logging: invalid {LOG_LEVEL_ENV} value; falling back to INFO.");
    }

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_ansi(false)
        .compact()
        .try_init()
}

fn resolve_level(raw: Option<&str>) -> (Level, bool) {
    let Some(raw) = raw else {
        return (DEFAULT_LOG_LEVEL, false);
    };

    let normalized = raw.trim().to_ascii_lowercase();

    match normalized.as_str() {
        "trace" => (Level::TRACE, false),
        "debug" => (Level::DEBUG, false),
        "info" => (Level::INFO, false),
        "warn" => (Level::WARN, false),
        "error" => (Level::ERROR, false),
        _ => (DEFAULT_LOG_LEVEL, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_level_is_info() {
        assert_eq!(resolve_level(None), (Level::INFO, false));
    }

    #[test]
    fn accepted_levels_are_case_insensitive() {
        assert_eq!(resolve_level(Some("TRACE")), (Level::TRACE, false));
        assert_eq!(resolve_level(Some(" debug ")), (Level::DEBUG, false));
        assert_eq!(resolve_level(Some("Info")), (Level::INFO, false));
        assert_eq!(resolve_level(Some("WARN")), (Level::WARN, false));
        assert_eq!(resolve_level(Some("error")), (Level::ERROR, false));
    }

    #[test]
    fn invalid_level_falls_back_to_info() {
        assert_eq!(resolve_level(Some("verbose")), (Level::INFO, true));
    }
}
