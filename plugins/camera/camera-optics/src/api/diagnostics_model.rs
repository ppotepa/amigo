use amigo_plugin_api::CandidateTrace;

pub fn candidate_trace(
    reason: impl Into<Option<String>>,
    targets: Vec<amigo_plugin_api::TargetId>,
    status: amigo_plugin_api::CandidateStatus,
) -> CandidateTrace {
    CandidateTrace {
        domain: amigo_plugin_api::DomainId("camera.optics".to_string()),
        status,
        reason: reason.into(),
        targets,
    }
}
