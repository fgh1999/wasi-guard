pub type WasiErrno = u16;

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
impl Action {
    pub const fn default() -> Action {
        Self::Kill
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::default()
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Allow, Self::Allow) => Some(std::cmp::Ordering::Equal),
            (Self::Allow, _) => Some(std::cmp::Ordering::Less),
            (_, Self::Allow) => Some(std::cmp::Ordering::Greater),
            (Self::Log, Self::Log) => Some(std::cmp::Ordering::Equal),
            (Self::Log, _) => Some(std::cmp::Ordering::Less),
            (_, Self::Log) => Some(std::cmp::Ordering::Greater),
            (Self::ReturnErrno(_), Self::ReturnErrno(_)) => None,
            (Self::ReturnErrno(_), _) => Some(std::cmp::Ordering::Less),
            (_, Self::ReturnErrno(_)) => Some(std::cmp::Ordering::Greater),
            (Self::Kill, Self::Kill) => Some(std::cmp::Ordering::Equal),
        }
    }
}

/// Returns an iterator that filters out `Action::Allow` from the given actions.
pub fn actions_to_execute(actions: &[Action]) -> impl Iterator<Item = &Action> + '_ {
    actions.iter().filter(|&&act| act != Action::Allow)
}

// TODO: impl action interface in nigredo

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn action_order() {
        assert!(Action::Allow < Action::Log);
        assert!(Action::Log < Action::ReturnErrno(0));
        assert!(Action::ReturnErrno(0) < Action::Kill);
    }
}
