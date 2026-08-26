#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReplayDigest(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayFrameDigest {
    pub frame_index: u64,
    pub input: ReplayDigest,
    pub world: ReplayDigest,
    pub render: ReplayDigest,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeterministicReplay {
    frames: Vec<ReplayFrameDigest>,
}

impl DeterministicReplay {
    pub fn record(
        &mut self,
        frame_index: u64,
        input_bytes: &[u8],
        world_bytes: &[u8],
        render_bytes: &[u8],
    ) {
        self.frames.push(ReplayFrameDigest {
            frame_index,
            input: stable_digest(input_bytes),
            world: stable_digest(world_bytes),
            render: stable_digest(render_bytes),
        });
    }

    pub fn frames(&self) -> &[ReplayFrameDigest] { &self.frames }

    pub fn digest(&self) -> ReplayDigest {
        let mut hash = FNV_OFFSET;
        for frame in &self.frames {
            hash_u64(&mut hash, frame.frame_index);
            hash_u64(&mut hash, frame.input.0);
            hash_u64(&mut hash, frame.world.0);
            hash_u64(&mut hash, frame.render.0);
        }
        ReplayDigest(hash)
    }

    pub fn first_divergence(&self, other: &Self) -> Option<u64> {
        let common = self.frames.len().min(other.frames.len());
        for index in 0..common {
            if self.frames[index] != other.frames[index] {
                return Some(self.frames[index].frame_index.min(other.frames[index].frame_index));
            }
        }
        if self.frames.len() == other.frames.len() {
            None
        } else {
            self.frames.get(common).or_else(|| other.frames.get(common)).map(|frame| frame.frame_index)
        }
    }
}

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub fn stable_digest(bytes: &[u8]) -> ReplayDigest {
    let mut hash = FNV_OFFSET;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    ReplayDigest(hash)
}

fn hash_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scripted_run(seed: u64) -> DeterministicReplay {
        let mut replay = DeterministicReplay::default();
        let mut state = seed;
        for frame in 0..240u64 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let input = [((frame % 7) as u8), ((frame % 3) as u8)];
            let world = state.to_le_bytes();
            let render = state.rotate_left(17).to_le_bytes();
            replay.record(frame, &input, &world, &render);
        }
        replay
    }

    #[test]
    fn identical_seed_and_inputs_have_identical_replay_digest() {
        assert_eq!(scripted_run(42), scripted_run(42));
        assert_eq!(scripted_run(42).digest(), scripted_run(42).digest());
    }

    #[test]
    fn divergent_state_reports_first_frame() {
        let a = scripted_run(42);
        let b = scripted_run(43);
        assert_eq!(a.first_divergence(&b), Some(0));
    }
}
