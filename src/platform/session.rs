pub mod standalone;
pub mod uwsm;

pub use standalone::StandaloneSession;
pub use uwsm::UwsmSession;

use std::collections::HashMap;

/// Identifies the kind of managed desktop session manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionManagerKind {
    Uwsm,
}

impl std::fmt::Display for SessionManagerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uwsm => write!(f, "UWSM"),
        }
    }
}

/// Explicit session mode of the running shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    Standalone,
    Managed(SessionManagerKind),
}

/// Explicit capability flags exposed by the active session backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionCapabilities {
    pub lifecycle_notifications: bool,
    pub managed_application_launch: bool,
}

/// Typed specification of an application launch request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationLaunch {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

/// Handle to a launched child process or task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildHandle {
    pub id: u32,
}

/// Errors originating from session operations or lifecycle notifications.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SessionError {
    #[error("session notification failed: {0}")]
    NotificationFailed(String),
    #[error("application launch not supported: {0}")]
    LaunchNotSupported(String),
    #[error("session error: {0}")]
    Other(String),
}

/// Trait defining the narrow session lifecycle and delegation boundary.
pub trait SessionIntegration: std::fmt::Debug {
    fn mode(&self) -> SessionMode;
    fn capabilities(&self) -> SessionCapabilities;
    fn notify_ready(&mut self) -> Result<(), SessionError>;
    fn begin_shutdown(&mut self) -> Result<(), SessionError>;
    fn launch(&mut self, request: ApplicationLaunch) -> Result<ChildHandle, SessionError>;
}

/// Trait for probing session management availability in a testable/injectable manner.
pub trait SessionDetector: Send + Sync {
    fn probe(&self) -> Result<Option<SessionManagerKind>, String>;
}

/// Default system session detector.
///
/// Automatic UWSM session detection is intentionally deferred until a stable,
/// documented detection contract is validated. Production therefore safely defaults
/// to `StandaloneSession`. Injected detectors remain available for testing and
/// backend architecture validation.
pub struct SystemSessionDetector;

impl SessionDetector for SystemSessionDetector {
    fn probe(&self) -> Result<Option<SessionManagerKind>, String> {
        Ok(None)
    }
}

/// Detects and constructs the appropriate session backend using the provided detector.
pub fn detect_session_with(detector: &dyn SessionDetector) -> Box<dyn SessionIntegration> {
    match detector.probe() {
        Ok(Some(SessionManagerKind::Uwsm)) => {
            tracing::info!("detected UWSM managed desktop session; using UWSM backend");
            Box::new(UwsmSession::new())
        }
        Ok(None) => {
            tracing::debug!("no managed session detected; using Standalone backend");
            Box::new(StandaloneSession::new())
        }
        Err(err) => {
            tracing::info!(%err, "session detector probe failed; falling back to Standalone backend");
            Box::new(StandaloneSession::new())
        }
    }
}

/// Detects and constructs the default session backend for the current environment.
pub fn detect_session() -> Box<dyn SessionIntegration> {
    detect_session_with(&SystemSessionDetector)
}
