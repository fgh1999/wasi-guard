pub type WasiErrno = i32;

/// Actions that can be taken by the policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Allow,
    Log,
    /// Return the WASI call with a user-defined errno.
    ReturnErrno(WasiErrno),
    /// Terminate the WASM task.
    Kill,
}

impl Default for Action {
    fn default() -> Self {
        Self::Kill
    }
}
