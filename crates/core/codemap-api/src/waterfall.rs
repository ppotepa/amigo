use crate::node::CodeMapNodeId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WaterfallTrace {
    pub source: Option<CodeMapNodeId>,
    pub contribution: Option<CodeMapNodeId>,
    pub candidate: Option<CodeMapNodeId>,
    pub target: Option<CodeMapNodeId>,
    pub consumer: Option<CodeMapNodeId>,
    pub diagnostic: Option<CodeMapNodeId>,
    pub test: Option<CodeMapNodeId>,
}

impl WaterfallTrace {
    pub fn is_complete(&self) -> bool {
        self.source.is_some()
            && self.contribution.is_some()
            && self.candidate.is_some()
            && self.target.is_some()
            && self.consumer.is_some()
            && self.diagnostic.is_some()
            && self.test.is_some()
    }

    pub fn missing_stages(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();

        if self.source.is_none() {
            missing.push("source");
        }
        if self.contribution.is_none() {
            missing.push("contribution");
        }
        if self.candidate.is_none() {
            missing.push("candidate");
        }
        if self.target.is_none() {
            missing.push("target");
        }
        if self.consumer.is_none() {
            missing.push("consumer");
        }
        if self.diagnostic.is_none() {
            missing.push("diagnostic");
        }
        if self.test.is_none() {
            missing.push("test");
        }

        missing
    }
}
