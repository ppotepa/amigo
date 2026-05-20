use std::sync::Arc;

use amigo_runtime::Runtime;

pub(super) fn optional<T>(runtime: &Runtime) -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    runtime.resolve::<T>()
}
