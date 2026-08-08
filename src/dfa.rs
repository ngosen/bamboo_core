//! DFA-based engine for high-performance Vietnamese input method.

use crate::engine::Transformation;
use crate::input_method::InputMethod;
use rustc_hash::FxHashMap;

/// Maximum transitions per DFA state. Vietnamese input typically uses ~12 keys per state.
const MAX_TRANS: usize = 24;

/// A compact DFA state representing a unique syllable composition.
///
/// Transitions are stored as sorted `(key, state_id)` pairs instead of a full
/// 128-entry table. A 128-bit bitset enables O(1) "has transition?" checks
/// and fast rejection for keys that don't have transitions.
#[derive(Clone, Debug)]
pub struct State {
    /// Sorted (key, state_id) pairs. Linear scan is fine for ≤24 entries.
    pub transitions: [(u8, u32); MAX_TRANS],
    /// Number of valid transitions.
    pub trans_len: u8,
    /// 128-bit bitset: bit i = 1 means key (i) has a transition.
    /// Enables O(1) rejection for keys without transitions.
    pub bitset: [u64; 2],
    /// Precomputed hash of the composition for O(1) equality check.
    pub comp_hash: u64,
    /// Start index in the DFA arena.
    pub comp_offset: u32,
    /// Number of transformations in this state.
    pub comp_len: u8,
}

impl Default for State {
    fn default() -> Self {
        Self {
            transitions: [(0, 0); MAX_TRANS],
            trans_len: 0,
            bitset: [0; 2],
            comp_hash: 0,
            comp_offset: 0,
            comp_len: 0,
        }
    }
}

impl State {
    /// Looks up a transition by key. Uses bitset for O(1) rejection.
    #[inline]
    pub fn get_transition(&self, key: u8) -> u32 {
        // O(1) rejection: if the bit is not set, no transition exists.
        let idx = key as usize;
        if self.bitset[idx / 64] & (1u64 << (idx % 64)) == 0 {
            return 0;
        }
        // Bit is set — linear scan to find the exact transition.
        let trans = &self.transitions[..self.trans_len as usize];
        for &(k, id) in trans {
            if k == key {
                return id;
            }
        }
        0
    }

    /// Sets a transition, maintaining sorted order by key.
    #[inline]
    pub fn set_transition(&mut self, key: u8, state_id: u32) {
        let idx = key as usize;
        self.bitset[idx / 64] |= 1u64 << (idx % 64);

        let len = self.trans_len as usize;
        for entry in self.transitions[..len].iter_mut() {
            if entry.0 == key {
                entry.1 = state_id;
                return;
            }
        }
        if len < MAX_TRANS {
            self.transitions[len] = (key, state_id);
            self.trans_len += 1;
        }
    }
}

/// The DFA core that manages states and transitions with Arena Allocation.
pub struct Dfa {
    pub states: Vec<State>,
    pub arena: Vec<Transformation>,
    /// Maps composition hash → state_id for O(1) lookup (collision-free for distinct compositions).
    pub hash_to_state: FxHashMap<u64, u32>,
}

impl Clone for Dfa {
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            arena: self.arena.clone(),
            hash_to_state: self.hash_to_state.clone(),
        }
    }
}

impl Default for Dfa {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes a fast hash of a composition slice.
/// Uses FxHash (same as the HashMap) for consistency.
#[inline]
fn hash_composition(composition: &[Transformation]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    // Hash the raw bytes of the transformation slice for speed.
    let bytes = unsafe {
        std::slice::from_raw_parts(
            composition.as_ptr() as *const u8,
            std::mem::size_of_val(composition),
        )
    };
    bytes.hash(&mut hasher);
    hasher.finish()
}

impl Dfa {
    /// Creates a new DFA with an initial empty state.
    pub fn new() -> Self {
        let mut dfa = Self {
            states: Vec::with_capacity(128),
            arena: Vec::with_capacity(512),
            hash_to_state: FxHashMap::default(),
        };
        dfa.states.push(State::default());
        dfa.hash_to_state.insert(0, 0); // empty composition hash = 0
        dfa
    }

    pub fn get_state(&self, id: u32) -> &State {
        &self.states[id as usize]
    }

    pub fn get_composition(&self, state_id: u32) -> &[Transformation] {
        let state = &self.states[state_id as usize];
        let start = state.comp_offset as usize;
        let end = start + state.comp_len as usize;
        &self.arena[start..end]
    }

    pub fn add_state(&mut self, composition: &[Transformation]) -> u32 {
        let hash = hash_composition(composition);

        // Fast path: hash match → verify arena equality (no heap allocation).
        if let Some(&id) = self.hash_to_state.get(&hash) {
            let existing = self.get_composition(id);
            if existing == composition {
                return id;
            }
            // Hash collision — fall through to add new state.
        }

        let id = self.states.len() as u32;
        let comp_offset = self.arena.len() as u32;
        let comp_len = composition.len() as u8;

        self.arena.extend_from_slice(composition);
        self.states.push(State {
            comp_hash: hash,
            comp_offset,
            comp_len,
            ..State::default()
        });

        self.hash_to_state.insert(hash, id);
        id
    }

    pub fn find_state(&self, composition: &[Transformation]) -> Option<u32> {
        let hash = hash_composition(composition);
        let id = *self.hash_to_state.get(&hash)?;
        let existing = self.get_composition(id);
        if existing == composition {
            Some(id)
        } else {
            None // Hash collision with different composition.
        }
    }
}

/// A DFA compiler that supports pre-initializing common states.
pub struct DfaCompiler<'a> {
    #[allow(dead_code)]
    pub input_method: &'a InputMethod,
    #[allow(dead_code)]
    pub flags: u32,
    pub dfa: Dfa,
    engine: crate::Engine,
}

impl<'a> DfaCompiler<'a> {
    pub fn new(im: &'a InputMethod, flags: u32) -> Self {
        let engine = crate::Engine::with_config(im.clone(), crate::Config::from_flags(flags));
        Self { input_method: im, flags, dfa: Dfa::new(), engine }
    }

    /// Compiles common Vietnamese syllables into the DFA.
    pub fn compile_common(&mut self) {
        let fc = [
            "", "b", "c", "ch", "d", "dd", "g", "gh", "h", "k", "kh", "l", "m", "n", "nh", "ng",
            "ngh", "p", "ph", "q", "r", "s", "t", "th", "tr", "v", "x",
        ];
        let vowels = [
            "a", "e", "i", "o", "u", "y", "aa", "ee", "oo", "aw", "ow", "uw", "ai", "ao", "au",
            "ay", "ie", "oa", "oe", "oi", "ua", "ue", "ui", "uo", "uy",
        ];
        let tones = ["", "s", "f", "r", "x", "j"];

        // Stack-allocated buffer: max prefix "ngh" (3) + max vowel "uay" (3) + tone (1) = 7
        let mut buf = [0u8; 8];
        for &f in &fc {
            for &v in &vowels {
                for &t in &tones {
                    let mut pos = 0;
                    for &b in f.as_bytes() { buf[pos] = b; pos += 1; }
                    for &b in v.as_bytes() { buf[pos] = b; pos += 1; }
                    for &b in t.as_bytes() { buf[pos] = b; pos += 1; }
                    // SAFETY: all parts are ASCII
                    let seq = unsafe { std::str::from_utf8_unchecked(&buf[..pos]) };
                    self.simulate_str(seq);
                }
            }
        }
    }

    fn simulate_str(&mut self, s: &str) {
        self.engine.reset();

        let mut current_state = 0u32;
        for k in s.chars() {
            if !k.is_ascii() {
                continue;
            }

            let prev_state = current_state;
            self.engine.process_key(k, crate::Mode::Vietnamese);

            let comp = self.engine.active_slice();
            current_state = self.dfa.add_state(comp);

            // Link the transition
            self.dfa.states[prev_state as usize].set_transition(k as u8, current_state);
        }
    }
}
