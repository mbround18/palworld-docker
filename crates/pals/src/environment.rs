pub fn name() -> String {
    gsm_shared::fetch_var("NAME", "My Pal Server")
}

#[cfg(test)]
mod tests {
    use super::name;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn name_uses_default_when_environment_is_missing() {
        let _lock = env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe {
            std::env::remove_var("NAME");
        }

        assert_eq!(name(), "My Pal Server");
    }
}
