use super::{MotionShutterCoverage2d, MotionShutterResponse2d};

#[derive(Clone, Debug, PartialEq)]
pub struct MotionShutterSource2d {
    pub owner: String,
    pub declared: bool,
    pub coverage: MotionShutterCoverage2d,
    pub response: MotionShutterResponse2d,
}
