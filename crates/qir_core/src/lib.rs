pub mod analytics;
pub mod backup;
pub mod db;
pub mod demo;
pub mod domain;
pub mod error;
pub mod ingest;
pub mod metrics;
pub mod normalize;
pub mod profiles;
pub mod repo;
pub mod report;
pub mod sanitize;
pub mod validate;

#[cfg(test)]
mod tests {
    use super::error::AppError;

    #[test]
    fn app_error_is_structured() {
        let err = AppError::new("DB_TEST", "db failed").with_retryable(false);
        assert_eq!(err.code, "DB_TEST");
        assert_eq!(err.message, "db failed");
        assert_eq!(err.retryable, false);
    }
}
