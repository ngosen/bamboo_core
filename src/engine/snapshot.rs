use super::state::{MAX_ACTIVE_TRANS, Transformation};

/// A lightweight snapshot of the engine state before a keystroke, used for O(1) backspace.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Snapshot {
    pub(crate) active_buffer: [Transformation; MAX_ACTIVE_TRANS],
    pub(crate) active_len: usize,
    pub(crate) current_state_id: u32,
    pub(crate) english_bypass: bool,
}
