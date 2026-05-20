use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraExposureMode2d {
    Auto,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraExposure2d {
    pub mode: CameraExposureMode2d,
    pub iso: f32,
    pub compensation: f32,
    pub white_balance: f32,
    pub nd_stops: f32,
    pub target_luma: f32,
    pub adaptation_speed: f32,
    pub min_iso: f32,
    pub max_iso: f32,
    pub opacity: f32,
}

impl Default for CameraExposure2d {
    fn default() -> Self {
        Self {
            mode: CameraExposureMode2d::Manual,
            iso: 400.0,
            compensation: 0.0,
            white_balance: 5600.0,
            nd_stops: 0.0,
            target_luma: 0.42,
            adaptation_speed: 0.8,
            min_iso: 100.0,
            max_iso: 3200.0,
            opacity: 1.0,
        }
    }
}

impl CameraExposure2d {
    pub fn normalized(mut self) -> Self {
        self.iso = finite_or(self.iso, 400.0).clamp(25.0, 25600.0);
        self.compensation = finite_or(self.compensation, 0.0).clamp(-8.0, 8.0);
        self.white_balance = finite_or(self.white_balance, 5600.0).clamp(1800.0, 12000.0);
        self.nd_stops = finite_or(self.nd_stops, 0.0).clamp(0.0, 16.0);
        self.target_luma = finite_or(self.target_luma, 0.42).clamp(0.01, 1.0);
        self.adaptation_speed = finite_or(self.adaptation_speed, 0.8).clamp(0.0, 20.0);
        self.min_iso = finite_or(self.min_iso, 100.0).clamp(25.0, 25600.0);
        self.max_iso = finite_or(self.max_iso, 3200.0).clamp(self.min_iso, 25600.0);
        self.opacity = finite_or(self.opacity, 1.0).clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.opacity > 0.0
    }
}
