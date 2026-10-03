/// Invalid, ambiguous, or unexpected compiler unit observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitSelectionError {
    InvalidContext,
    AmbiguousUnit,
    UnexpectedUnit,
}

impl std::fmt::Display for UnitSelectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidContext => "invalid compiler architecture unit context",
            Self::AmbiguousUnit => "ambiguous compiler architecture unit observation key",
            Self::UnexpectedUnit => "unexpected compiler configuration for a selected architecture package",
        })
    }
}

impl std::error::Error for UnitSelectionError {}
