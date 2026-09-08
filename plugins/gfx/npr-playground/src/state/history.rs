use super::Settings;
use std::time::{Duration, Instant};

#[derive(Default)]
pub(super) struct History {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
}
struct Entry {
    key: String,
    before: Settings,
    after: Settings,
    time: Instant,
}
impl History {
    pub fn record(&mut self, key: &str, before: &Settings, after: &Settings) {
        if before == after {
            return;
        }
        self.redo.clear();
        if let Some(last) = self
            .undo
            .last_mut()
            .filter(|e| e.key == key && e.time.elapsed() < Duration::from_millis(500))
        {
            // Preserve only edited fields across animation ticks between edits.
            let mut merged = serde_yaml::to_value(&last.after).unwrap();
            patch(
                &mut merged,
                &serde_yaml::to_value(before).unwrap(),
                &serde_yaml::to_value(after).unwrap(),
            );
            last.after = serde_yaml::from_value(merged).unwrap();
            last.time = Instant::now();
        } else {
            if self.undo.len() == 128 {
                self.undo.remove(0);
            }
            self.undo.push(Entry {
                key: key.into(),
                before: before.clone(),
                after: after.clone(),
                time: Instant::now(),
            });
        }
    }
    pub fn available(&self) -> (bool, bool) {
        (!self.undo.is_empty(), !self.redo.is_empty())
    }
    pub fn restore(&mut self, settings: &mut Settings, redo: bool) -> Result<(), String> {
        let source = if redo { &mut self.redo } else { &mut self.undo };
        let Some(entry) = source.last() else {
            return Ok(());
        };
        let mut candidate = serde_yaml::to_value(&*settings).unwrap();
        let (from, to) = if redo {
            (&entry.before, &entry.after)
        } else {
            (&entry.after, &entry.before)
        };
        patch(
            &mut candidate,
            &serde_yaml::to_value(from).unwrap(),
            &serde_yaml::to_value(to).unwrap(),
        );
        let next: Settings = serde_yaml::from_value(candidate).map_err(|e| e.to_string())?;
        next.validate()?;
        *settings = next;
        let mut entry = source.pop().unwrap();
        // A new edit after undo/redo must never merge into an old gesture.
        entry.time = Instant::now() - Duration::from_secs(1);
        if redo {
            self.undo.push(entry);
        } else {
            self.redo.push(entry);
        }
        Ok(())
    }
}
/// Apply only fields changed by a user edit; leave unrelated live animation intact.
fn patch(current: &mut serde_yaml::Value, from: &serde_yaml::Value, to: &serde_yaml::Value) {
    if from == to {
        return;
    }
    if let (Some(current), Some(from), Some(to)) =
        (current.as_mapping_mut(), from.as_mapping(), to.as_mapping())
    {
        for (key, value) in to {
            if let (Some(current), Some(before)) = (current.get_mut(key), from.get(key)) {
                patch(current, before, value);
            }
        }
    } else {
        *current = to.clone();
    }
}
