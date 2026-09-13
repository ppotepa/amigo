use crate::playback::{ModelPlayback, PlaybackSource};
use amigo_3d_mesh::{MeshAnimationClip, MeshGeometryAsset};
use amigo_render_npr::{NprGeometry, NprPreparedSurfaceVariants};
use std::sync::Arc;

pub(super) struct ModelGeometry {
    bind: NprPreparedSurfaceVariants,
    asset: Option<Arc<MeshGeometryAsset>>,
    /// One replaceable pose, not a cache entry for every animation frame.
    pose: Option<((usize, u32), NprPreparedSurfaceVariants)>,
}
impl ModelGeometry {
    pub fn builtin(geometry: NprGeometry) -> Self {
        Self {
            bind: NprPreparedSurfaceVariants::new(geometry),
            asset: None,
            pose: None,
        }
    }
    pub fn imported(asset: MeshGeometryAsset) -> Result<Self, String> {
        Ok(Self {
            bind: NprPreparedSurfaceVariants::new(NprGeometry::from_indexed(
                &asset.positions,
                &asset.indices,
            )?),
            asset: Some(Arc::new(asset)),
            pose: None,
        })
    }
    pub fn animations(&self) -> Vec<MeshAnimationClip> {
        self.asset
            .as_ref()
            .map(|a| a.animations().to_vec())
            .unwrap_or_default()
    }
    pub fn prepared(
        &mut self,
        playback: Option<&ModelPlayback>,
    ) -> Result<&mut NprPreparedSurfaceVariants, String> {
        let Some(playback) = playback else {
            return Ok(&mut self.bind);
        };
        let PlaybackSource::Clip { index } = playback.source else {
            return Ok(&mut self.bind);
        };
        let key = (index, playback.time_seconds.to_bits());
        if self.pose.as_ref().is_none_or(|(old, _)| *old != key) {
            let frame = self
                .asset
                .as_ref()
                .ok_or("built-in model has no animation clips")?
                .sample_animation(index, playback.time_seconds)?;
            let prepared = NprPreparedSurfaceVariants::new(NprGeometry::from_indexed(
                &frame.positions,
                &frame.indices,
            )?);
            self.pose = Some((key, prepared));
        }
        Ok(&mut self.pose.as_mut().unwrap().1)
    }
}
