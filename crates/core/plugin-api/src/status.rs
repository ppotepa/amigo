#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContributionStatus {
    Active,
    Inactive,
    NotDeclared,
    Unsupported,
    Noop,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CandidateStatus {
    Active,
    Inactive,
    NotDeclared,
    Unsupported,
    Noop,
    Error,
}
