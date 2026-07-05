//! Platform recharge catalog tenant resolution.
//!
//! Public recharge packages and exchange rules fall back to a platform-owned tenant
//! when the caller tenant has no scoped catalog. The tenant id is configurable via
//! `SDKWORK_ORDER_PLATFORM_CATALOG_TENANT_ID` (default `100001`).

pub const ENV_PLATFORM_CATALOG_TENANT_ID: &str = "SDKWORK_ORDER_PLATFORM_CATALOG_TENANT_ID";
pub const DEFAULT_PLATFORM_CATALOG_TENANT_ID: &str = "100001";
const PLATFORM_TENANT_PLACEHOLDER: &str = "__PLATFORM_TENANT__";

/// Resolves the platform catalog tenant id from environment with validation.
pub fn platform_catalog_tenant_id() -> String {
    std::env::var(ENV_PLATFORM_CATALOG_TENANT_ID)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| is_valid_platform_catalog_tenant_id(value))
        .unwrap_or_else(|| DEFAULT_PLATFORM_CATALOG_TENANT_ID.to_owned())
}

/// Materializes SQL that contains `__PLATFORM_TENANT__` with the resolved platform tenant.
pub fn materialize_platform_catalog_sql(template: &str) -> String {
    template.replace(PLATFORM_TENANT_PLACEHOLDER, &platform_catalog_tenant_id())
}

fn is_valid_platform_catalog_tenant_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_platform_catalog_tenant_is_stable() {
        let _guard = EnvGuard::unset(ENV_PLATFORM_CATALOG_TENANT_ID);
        assert_eq!(
            platform_catalog_tenant_id(),
            DEFAULT_PLATFORM_CATALOG_TENANT_ID
        );
    }

    #[test]
    fn invalid_platform_catalog_tenant_falls_back_to_default() {
        let _guard = EnvGuard::set(ENV_PLATFORM_CATALOG_TENANT_ID, "'; DROP TABLE--");
        assert_eq!(
            platform_catalog_tenant_id(),
            DEFAULT_PLATFORM_CATALOG_TENANT_ID
        );
    }

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn unset(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            // SAFETY: test-only serial mutation of process environment.
            unsafe { std::env::remove_var(key) };
            Self { key, previous }
        }

        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            // SAFETY: test-only serial mutation of process environment.
            unsafe { std::env::set_var(key, value) };
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => unsafe { std::env::set_var(self.key, value) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }
}
