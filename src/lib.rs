//! # Bamboo Core
//!
//! **Bamboo Core** is an ultra-fast, zero-heap-hotpath Vietnamese Input Method Engine (IME) core written in pure Rust.
//! Ported and evolved from the original [bamboo-core](https://github.com/BambooEngine/bamboo-core) (Go), it powers
//! modern Vietnamese input on Linux (Fcitx5, `IBus`), macOS, Windows, text editors, embedded systems, and foreign-language bindings.
//!
//! ## Key Highlights
//!
//! - **Hybrid Architecture**: Combines a declarative **Rule Engine** with a **Lazy JIT DFA** (Deterministic Finite Automaton)
//!   for $O(1)$ amortized keystroke processing.
//! - **Zero Heap Allocations on Hot Paths**: Active compositions live in a stack-allocated buffer ([`advanced::MAX_ACTIVE_TRANS`] = 16).
//!   Transitions, backspaces, and commits execute with zero per-keystroke dynamic memory allocations.
//! - **$O(1)$ Keystroke Undo & Grapheme Deletion**: Supports both snapshot-based instant keystroke backspace ([`Engine::remove_last_char`])
//!   and smart grapheme-level backspace ([`Engine::remove_last_output_char`]) which preserves diacritics on earlier letters.
//! - **3-Way Diff for Text Editors**: [`Engine::process_key_delta`] computes longest common prefixes (LCP) via 8-byte chunk scanning (SWAR)
//!   and returns `(backspaces_count, backspaces_bytes, inserted_suffix)` to update UI preedit buffers with minimal flicker.
//! - **Built-in Input Methods**: Standard [`InputMethod::telex()`], [`InputMethod::vni()`], [`InputMethod::viqr()`], [`InputMethod::microsoft_layout()`],
//!   [`InputMethod::telex_2()`], [`InputMethod::telex_w()`], and hybrid combinations.
//! - **Encoding Conversion**: Includes encoders and character tables for 16 Vietnamese charsets (Unicode, TCVN3, VNI Windows, Windows-1258, VISCII, VPS, NCR, etc.).
//! - **C-Compatible FFI**: Full `extern "C"` API in [`ffi`] for seamless integration with C/C++, Python, Swift, or native GUI toolkits.
//!
//! ## API Selection Guide
//!
//! | Use Case | Primary API | Description |
//! |---|---|---|
//! | **IME Frontend (Fcitx5 / `IBus` / Native)** | [`Engine::process_key`], [`Engine::output`] | Feeds keys one-by-one, updates internal buffer, queries current composition word |
//! | **Text Editor / Terminal / IDE** | [`Engine::process_key_delta`] | Returns a 3-way diff `(backspaces, bytes, inserted)` to replace only changed suffixes |
//! | **Keystroke-level Backspace** | [`Engine::remove_last_char`] | $O(1)$ undo of the last physical keypress using stack snapshots |
//! | **Grapheme-level Backspace** | [`Engine::remove_last_output_char`] | Deletes preceding character while retaining marks/tones on remaining vowels (e.g. `tiếng` $\rightarrow$ `tiến`) |
//! | **Word Finalization** | [`Engine::commit`] | Clears composing state and appends text to committed stream |
//! | **Batch / Convenience (Testing)** | [`Engine::process`] | Processes a full string and returns output (**convenience only, not for real-time IME**) |
//! | **Charset Conversion** | [`advanced::encode`] | Converts Unicode strings to legacy encodings (TCVN3, VNI, etc.) |
//!
//! ## Quick Start — Real-Time IME Integration
//!
//! Feed keystrokes one at a time using [`Engine::process_key`]:
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod};
//!
//! let mut engine = Engine::new(InputMethod::telex());
//!
//! engine.process_key('t', Mode::Vietnamese);
//! engine.process_key('i', Mode::Vietnamese);
//! engine.process_key('e', Mode::Vietnamese);
//! engine.process_key('e', Mode::Vietnamese);
//! engine.process_key('n', Mode::Vietnamese);
//! engine.process_key('g', Mode::Vietnamese);
//! engine.process_key('s', Mode::Vietnamese);
//!
//! assert_eq!(engine.output(), "tiếng");
//! ```
//!
//! ## Text Editor Integration — 3-Way Diff
//!
//! When integrating with text editors, computing the exact backspaces and new text to insert
//! prevents unnecessary full-string re-renders:
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod};
//!
//! let mut engine = Engine::new(InputMethod::telex());
//!
//! // Type 'a' -> insert "a", 0 backspaces
//! let (bs, _bytes, ins) = engine.process_key_delta('a', Mode::Vietnamese);
//! assert_eq!(bs, 0);
//! assert_eq!(ins, "a");
//!
//! // Type 's' -> delete 1 char ("a"), insert "á"
//! let (bs, _bytes, ins) = engine.process_key_delta('s', Mode::Vietnamese);
//! assert_eq!(bs, 1);
//! assert_eq!(ins, "á");
//! ```
//!
//! ## Smart Backspace Modes
//!
//! Bamboo Core provides two complementary backspace semantics:
//!
//! 1. **Keystroke Undo ([`Engine::remove_last_char`])**:
//!    Restores the engine to the exact state before the previous keystroke in $O(1)$ time.
//!
//!    ```rust
//!    use bamboo_core::{Engine, Mode, InputMethod};
//!
//!    let mut engine = Engine::new(InputMethod::telex());
//!    engine.process_str("chuyeenr", Mode::Vietnamese);
//!    assert_eq!(engine.output(), "chuyển");
//!
//!    // Undo 'r' (tone hook) -> returns to "chuyên"
//!    engine.remove_last_char(true);
//!    assert_eq!(engine.output(), "chuyên");
//!    ```
//!
//! 2. **Grapheme Deletion ([`Engine::remove_last_output_char`])**:
//!    Deletes the whole preceding letter before the caret while preserving diacritics and tone marks
//!    on the earlier characters.
//!
//!    ```rust
//!    use bamboo_core::{Engine, Mode, InputMethod};
//!
//!    let mut engine = Engine::new(InputMethod::telex());
//!    engine.process_str("tieesng", Mode::Vietnamese);
//!    assert_eq!(engine.output(), "tiếng");
//!
//!    // Drops 'g', keeps 'ê' and 's' tone mark -> "tiến"
//!    engine.remove_last_output_char();
//!    assert_eq!(engine.output(), "tiến");
//!    ```
//!
//! ## Tone Marking & Orthography Semantics
//!
//! - **Last Tone Key Wins**: Typing a new tone key replaces any existing tone on that syllable (`looixfsx` $\rightarrow$ `lỗi`).
//! - **Tone Undo**: Typing the exact same tone key consecutively removes the tone and outputs the raw character (`ass` $\rightarrow$ `as`).
//! - **Free Tone Marking**: Tones can be typed at any point during word composition (`hoangf` $\rightarrow$ `hoàng`).
//! - **Spelling Auto-Correction**: When [`Config::auto_correct`](crate::Config::auto_correct) is enabled, invalid syllable compositions
//!   automatically fall back to raw characters.

#![warn(
    missing_docs,
    clippy::undocumented_unsafe_blocks,
    clippy::doc_markdown,
    clippy::manual_let_else,
    clippy::semicolon_if_nothing_returned,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::perf,
    clippy::trivially_copy_pass_by_ref,
    clippy::large_types_passed_by_value,
    clippy::needless_collect,
    clippy::or_fun_call,
    clippy::format_push_string,
    clippy::unnecessary_to_owned,
    clippy::redundant_clone
)]

mod bamboo_util;
mod charset_def;
mod config;
mod dfa;
mod encoder;
mod engine;
mod flattener;
mod input_method;
mod input_method_def;
mod mode;
mod spelling;
mod utils;

pub mod ffi;
pub mod wasm;

/// Parallel batch processing utilities for bulk Vietnamese text transformations.
///
/// Available when the `parallel` feature is enabled.
#[cfg(feature = "parallel")]
pub mod parallel {
    use crate::{Engine, InputMethod, Mode};
    use rayon::prelude::*;

    /// Processes multiple input strings in parallel using Rayon work-stealing.
    ///
    /// # Example
    /// ```rust
    /// use bamboo_core::{parallel::process_batch, Mode, InputMethod};
    ///
    /// let inputs = vec!["tieengs", "vieetj", "nam"];
    /// let results = process_batch(&inputs, &InputMethod::telex(), Mode::Vietnamese);
    /// assert_eq!(results, vec!["tiếng", "việt", "nam"]);
    /// ```
    pub fn process_batch<S: AsRef<str> + Sync>(
        inputs: &[S],
        input_method: &InputMethod,
        mode: Mode,
    ) -> Vec<String> {
        inputs
            .par_iter()
            .map(|s| {
                let mut engine = Engine::new(input_method.clone());
                engine.process(s.as_ref(), mode)
            })
            .collect()
    }
}

pub use config::{Config, ConfigBuilder};
pub use engine::{Engine, RestoreMark, Transformation, TransformationStack};
pub use input_method::InputMethod;
pub use mode::{Mode, OutputOptions};

/// Advanced types for low-level interaction with the engine.
///
/// This module exposes internal structures and raw definitions
/// for users who need to build custom input methods or analyze the composition state.
pub mod advanced {
    pub use crate::engine::{MAX_ACTIVE_TRANS, Transformation, TransformationStack};
    pub use crate::input_method::{EffectType, Mark, Rule, Tone};
    pub use crate::mode::OutputOptions;

    pub use crate::charset_def::{
        CharsetDefinition, get_charset_definition, get_charset_definitions,
    };
    pub use crate::dfa::{Dfa, State};
    pub use crate::encoder::{charset_names, encode, get_charset_name, get_charset_names};
    pub use crate::input_method_def::{
        InputMethodDef, get_input_method, get_input_method_definitions,
    };
}
