use std::error::Error;
use std::fmt;

#[cfg(feature = "shell-timeout")]
use process_control::ExitStatus;
#[cfg(not(feature = "shell-timeout"))]
use std::process::ExitStatus;

#[cfg(feature = "shell-timeout")]
type ExitCode = i64;
#[cfg(not(feature = "shell-timeout"))]
type ExitCode = i32;

/// Describes the result of a process after it has failed
///
/// Produced by the [`.exit_ok`](ExitStatusExt::exit_ok) method on [`ExitStatus`].
///
/// This is implementation of the `exit_status_error` feature for stable Rust
/// or [process_control] (when the `shell-timeout` feature is enabled).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitStatusError(ExitStatus);

impl ExitStatusError {
    /// Reports the exit code, if applicable, from an `ExitStatusError`.
    ///
    /// See [std::process::ExitStatusError]
    pub fn code(&self) -> Option<ExitCode> {
        self.0.code()
    }

    /// Converts an `ExitStatusError` (back) to an `ExitStatus`.
    pub fn into_status(self) -> ExitStatus {
        self.0
    }
}

impl Error for ExitStatusError {}

impl fmt::Display for ExitStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "process exited unsuccessfully: {}", self.into_status())
    }
}

pub(crate) trait ExitStatusExt {
    /// Was termination successful?  Returns a `Result`.
    fn exit_ok(self) -> Result<(), ExitStatusError>;
}

impl ExitStatusExt for ExitStatus {
    fn exit_ok(self) -> Result<(), ExitStatusError> {
        if self.success() {
            Ok(())
        } else {
            Err(ExitStatusError(self))
        }
    }
}
