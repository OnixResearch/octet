/// Retain the first serialization failure until a caller's error boundary.
/// Validation functions still gather all ordinary policy issues on the success path.
#[derive(Default)]
pub(super) struct Issues {
    entries: Vec<crate::model::CompilerArchitectureIssue>,
    serialization_error: Option<String>,
}

impl Issues {
    pub(super) fn push(&mut self, candidate: Result<crate::model::CompilerArchitectureIssue, String>) {
        match candidate {
            Ok(entry) => self.entries.push(entry),
            Err(error) => {
                if self.serialization_error.is_none() {
                    self.serialization_error = Some(error);
                }
            }
        }
    }

    pub(super) fn stale_identity(&mut self, observed: &str, identity: impl FnOnce() -> Result<String, String>) -> bool {
        if !super::valid_blake3(observed) {
            return true;
        }
        match identity() {
            Ok(expected) => observed != expected,
            Err(error) => {
                if self.serialization_error.is_none() {
                    self.serialization_error = Some(error);
                }
                false
            }
        }
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len().saturating_add(usize::from(self.serialization_error.is_some()))
    }

    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.serialization_error.is_none()
    }

    pub(super) fn sort(&mut self) {
        self.entries.sort();
    }

    pub(super) fn dedup(&mut self) {
        self.entries.dedup();
    }

    pub(super) fn into_result(self) -> Result<Vec<crate::model::CompilerArchitectureIssue>, String> {
        match self.serialization_error {
            Some(error) => Err(error),
            None => Ok(self.entries),
        }
    }

    pub(super) fn into_messages(self, include_code: bool) -> Vec<String> {
        let mut messages = self
            .entries
            .into_iter()
            .map(|entry| {
                if include_code {
                    format!("{}: {}", entry.code, entry.message)
                } else {
                    entry.message
                }
            })
            .collect::<Vec<_>>();
        if let Some(error) = self.serialization_error {
            messages.push(format!("compiler architecture identity serialization failed: {error}"));
        }
        sorted_unique(messages)
    }
}

impl FromIterator<Result<crate::model::CompilerArchitectureIssue, String>> for Issues {
    fn from_iter<T: IntoIterator<Item = Result<crate::model::CompilerArchitectureIssue, String>>>(values: T) -> Self {
        let mut issues = Self::default();
        for issue in values {
            issues.push(issue);
        }
        issues
    }
}

pub(super) fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
