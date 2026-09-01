use super::{
    ApplicationLaunch, ChildHandle, SessionCapabilities, SessionError, SessionIntegration,
    SessionManagerKind, SessionMode,
};

/// Optional UWSM managed session backend boundary.
///
/// Establishes the managed-session backend architecture boundary. Concrete UWSM
/// readiness and shutdown notification mechanics remain intentionally deferred.
#[derive(Debug, Default, Clone)]
pub struct UwsmSession;

impl UwsmSession {
    pub fn new() -> Self {
        Self
    }
}

impl SessionIntegration for UwsmSession {
    fn mode(&self) -> SessionMode {
        SessionMode::Managed(SessionManagerKind::Uwsm)
    }

    fn capabilities(&self) -> SessionCapabilities {
        SessionCapabilities {
            lifecycle_notifications: false,
            managed_application_launch: false,
        }
    }

    fn notify_ready(&mut self) -> Result<(), SessionError> {
        // Lifecycle readiness hook boundary. Concrete external notification is intentionally deferred.
        tracing::info!("notified UWSM managed session readiness hook");
        Ok(())
    }

    fn begin_shutdown(&mut self) -> Result<(), SessionError> {
        // Lifecycle shutdown hook boundary. Concrete external notification is intentionally deferred.
        tracing::info!("notified UWSM managed session shutdown hook");
        Ok(())
    }

    fn launch(&mut self, _request: ApplicationLaunch) -> Result<ChildHandle, SessionError> {
        Err(SessionError::LaunchNotSupported(
            "UWSM application launch delegation is not yet implemented in Stage 2".into(),
        ))
    }
}
