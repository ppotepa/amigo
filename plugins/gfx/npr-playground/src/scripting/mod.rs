//! Rhai receives `npr_playground_metadata()` and `npr_playground_dispatch(intent)`
//! from the neutral playground provider adapter. Both clients use the same typed
//! `NprPlaygroundService`; scripts never execute UI code. Domain events are
//! published to ScriptEventQueue by the companion lifecycle, including when the
//! companion window is closed.
