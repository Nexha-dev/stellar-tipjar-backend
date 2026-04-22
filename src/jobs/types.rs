//! Job system type definitions and data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a job
pub type JobId = Uuid;

/// Unique identifier for a worker
pub type WorkerId = String;

/// Database row representation of a job
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub worker_id: Option<String>,
    pub priority: Option<i32>,
}

/// Main job entity representing a background task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub job_type: JobType,
    pub payload: JobPayload,
    pub status: JobStatus,
    pub retry_count: i32,
    pub max_retries: i32,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub worker_id: Option<WorkerId>,
    pub priority: i32,
}

impl TryFrom<JobRow> for Job {
    type Error = JobError;

    fn try_from(row: JobRow) -> Result<Self, Self::Error> {
        let job_type: JobType = serde_json::from_value(serde_json::Value::String(row.job_type))
            .map_err(JobError::Serialization)?;
        let payload: JobPayload =
            serde_json::from_value(row.payload).map_err(JobError::Serialization)?;
        let status: JobStatus = serde_json::from_value(serde_json::Value::String(row.status))
            .map_err(JobError::Serialization)?;

        Ok(Job {
            id: row.id,
            job_type,
            payload,
            status,
            retry_count: row.retry_count,
            max_retries: row.max_retries,
            created_at: row.created_at,
            scheduled_at: row.scheduled_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
            error_message: row.error_message,
            worker_id: row.worker_id,
            priority: row.priority.unwrap_or(0),
        })
    }
}

/// Types of jobs that can be processed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    VerifyTransaction,
    SendNotification,
    CleanupData,
    AggregateStats,
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{:?}", self));
        write!(f, "{}", s)
    }
}

/// Current status of a job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{:?}", self));
        write!(f, "{}", s)
    }
}

/// Type-safe job payloads for different job types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum JobPayload {
    VerifyTransaction {
        tip_id: Uuid,
        transaction_hash: String,
        creator_wallet: String,
    },
    SendNotification {
        creator_id: Uuid,
        tip_id: Uuid,
        notification_type: NotificationType,
        recipient_email: String,
    },
    CleanupData {
        cleanup_type: CleanupType,
        older_than: DateTime<Utc>,
    },
    AggregateStats {
        creator_username: String,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    },
}

/// Types of notifications that can be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    TipReceived,
    TipVerified,
    TipFailed,
}

/// Types of data cleanup operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupType {
    CompletedJobs,
    FailedJobs,
    OldTipData,
}

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: i32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 60000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Calculate the delay before the next retry attempt using exponential backoff
    pub fn next_delay_secs(&self, attempt: i32) -> i64 {
        let base = self.base_delay_ms as f64;
        let delay = base * self.backoff_multiplier.powi(attempt);
        let delay = delay.min(self.max_delay_ms as f64);
        (delay / 1000.0).ceil() as i64
    }
}

/// Job system configuration
#[derive(Debug, Clone)]
pub struct JobConfig {
    pub worker_count: usize,
    pub poll_interval_ms: u64,
    pub shutdown_timeout_ms: u64,
    pub cleanup_interval_hours: u64,
    pub job_retention_days: i64,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            poll_interval_ms: 1000,
            shutdown_timeout_ms: 30000,
            cleanup_interval_hours: 24,
            job_retention_days: 7,
        }
    }
}

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub worker_count: usize,
    pub poll_interval_ms: u64,
    pub shutdown_timeout_ms: u64,
}

impl From<&JobConfig> for WorkerConfig {
    fn from(config: &JobConfig) -> Self {
        Self {
            worker_count: config.worker_count,
            poll_interval_ms: config.poll_interval_ms,
            shutdown_timeout_ms: config.shutdown_timeout_ms,
        }
    }
}

/// Context provided to job handlers during execution
#[derive(Debug, Clone)]
pub struct JobContext {
    pub job_id: JobId,
    pub worker_id: WorkerId,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
}

/// Result type for job operations
pub type JobResult<T> = Result<T, JobError>;

/// Errors that can occur during job processing
#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Job handler not found for type: {job_type}")]
    HandlerNotFound { job_type: String },

    #[error("Job execution failed: {message}")]
    ExecutionFailed { message: String },

    #[error("External service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Job timeout after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    #[error("Worker shutdown requested")]
    Shutdown,

    #[error("Invalid job state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },
}
