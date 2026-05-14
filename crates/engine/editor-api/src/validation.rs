pub trait ValidationProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn validate(&self, ctx: &ValidationContext, out: &mut Vec<ValidationDiagnostic>);
}

#[derive(Debug, Default)]
pub struct ValidationContext;

#[derive(Debug, Clone)]
pub struct ValidationDiagnostic {
    pub severity: ValidationSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}
