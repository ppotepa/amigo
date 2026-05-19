use super::{FocusDepthCoverage2d, FocusDepthResponse2d};

#[derive(Clone, Debug, PartialEq)]
pub struct FocusDepthSource2d {
    pub owner: String,
    pub declared: bool,
    pub coverage: FocusDepthCoverage2d,
    pub response: FocusDepthResponse2d,
}
