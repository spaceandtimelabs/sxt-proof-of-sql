use alloy_sol_types::{private::primitives::Bytes, SolValue};
use snafu::Snafu;
use std::{ffi::OsStr, io, process::Command};

/// Error type returned by [`ForgeScript`] functions.
#[derive(Debug, Snafu)]
pub enum ForgeScriptError<'a> {
    /// The script failed to run. This is usually because `forge` is not installed.
    #[snafu(transparent)]
    ExecutionFailed { source: io::Error },
    #[snafu(display("Script threw and error. Underlying command: {underlying_command:?}."))]
    /// The script threw an error. This could be expected behavior.
    SolidityError { underlying_command: &'a Command },
}

/// [`ForgeScript`] enables running solidity from within rust. Ultimately this type calls `forge script`.
/// As a result, `forge` must be installed.
/// See <https://book.getfoundry.sh/getting-started/installation> for instructions.
pub struct ForgeScript {
    command: Command,
}

impl ForgeScript {
    /// Constructs a new `ForgeScript` for running a solidity function in `path` file, where the function is named `signature`.
    pub fn new(path: impl AsRef<OsStr>, signature: impl AsRef<OsStr>) -> Self {
        let mut command = Command::new("forge");
        command.arg("script").arg(path).arg("--sig").arg(signature);
        Self { command }
    }
    /// Adds an argument to pass to the script. Only one argument can be passed per use.
    pub fn arg(&mut self, arg: impl SolValue) -> &mut Self {
        self.command.arg(Bytes::from(arg.abi_encode()).to_string());
        self
    }
    /// Executes the script as a child process, waiting for it to finish and collecting its status.
    pub fn execute(&mut self) -> Result<(), ForgeScriptError> {
        self.command
            .status()?
            .success()
            .then_some(())
            .ok_or(ForgeScriptError::SolidityError {
                underlying_command: &self.command,
            })
    }
}
