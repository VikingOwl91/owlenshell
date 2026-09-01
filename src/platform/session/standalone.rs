use super::{
    ApplicationLaunch, ChildHandle, SessionCapabilities, SessionError, SessionIntegration,
    SessionMode,
};

/// Default standalone session implementation.
///
/// Owns its own process lifecycle with zero external session manager dependencies.
#[derive(Debug, Default, Clone)]
pub struct StandaloneSession;

impl StandaloneSession {
    pub fn new() -> Self {
        Self
    }
}

impl SessionIntegration for StandaloneSession {
    fn mode(&self) -> SessionMode {
        SessionMode::Standalone
    }

    fn capabilities(&self) -> SessionCapabilities {
        SessionCapabilities {
            lifecycle_notifications: false,
            managed_application_launch: false,
        }
    }

    fn notify_ready(&mut self) -> Result<(), SessionError> {
        // Safe no-op in standalone mode
        Ok(())
    }

    fn begin_shutdown(&mut self) -> Result<(), SessionError> {
        // Safe no-op in standalone mode
        Ok(())
    }

    fn launch(&mut self, _request: ApplicationLaunch) -> Result<ChildHandle, SessionError> {
        Err(SessionError::LaunchNotSupported(
            "standalone session does not support managed application launch".into(),
        ))
    }
}
