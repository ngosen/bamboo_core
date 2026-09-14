//! DFA-based engine for high-performance Vietnamese input method.
//!
//! This module provides the Deterministic Finite Automaton (DFA) state management,
//! bitset fast-path transition lookups, arena allocation for transformations,
//! and the JIT compiler for pre-compiling common syllable transitions.

use crate::engine::Transformation;
use crate::input_method::InputMethod;
use rustc_hash::FxHashMap;

/// Maximum transitions per DFA state. Vietnamese input typically uses ~12 keys per state.
const MAX_TRANS: usize = 24;

/// A compact DFA state representing a unique syllable composition.
///
/// Transitions are stored as sorted `(key, state_id)` pairs instead of a full
/// 128-entry table. A 128-bit bitset enables $O(1)$ "has transition?" checks
/// and fast rejection for keys that don't have transitions.
///
/// Memory layout is carefully aligned (160 bytes total) so that the hot fields
/// (bitset, keys, hash, offset, lengths) reside within the first 64-byte cache line.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct State {
    /// 128-bit bitset: bit `i = 1` means key `i` has a transition (16 bytes: offset 0..16).
    pub bitset: [u64; 2],
    /// Keys triggering transitions (24 bytes: offset 16..40).
    pub trans_keys: [u8; MAX_TRANS],
    /// Precomputed hash of the composition for $O(1)$ equality check (8 bytes: offset 40..48).
    pub comp_hash: u64,
    /// Start index in the DFA arena (4 bytes: offset 48..52).
    pub comp_offset: u32,
    /// Number of valid transitions (1 byte: offset 52).
    pub trans_len: u8,
    /// Number of transformations in this state (1 byte: offset 53).
    pub comp_len: u8,
    /// Explicit padding to ensure hot cache line boundary at 64 bytes.
    pub _pad_hot: [u8; 10],

    /// Destination state IDs for transitions (96 bytes: offset 64..160).
    pub trans_states: [u32; MAX_TRANS],
}

const _: () = assert!(std::mem::size_of::<State>() == 160);

impl Default for State {
    fn default() -> Self {
        Self {
            bitset: [0; 2],
            trans_keys: [0; MAX_TRANS],
            comp_hash: 0,
            comp_offset: 0,
            trans_len: 0,
            comp_len: 0,
            _pad_hot: [0; 10],
            trans_states: [0; MAX_TRANS],
        }
    }
}

impl State {
    /// Performs a fast transition lookup on an ASCII key using bitset check + SWAR parallel search.
    ///
    /// The algorithm runs in constant time without branches for typical keys:
    /// 1. $O(1)$ rejection via the 128-bit bitset (1 bit per ASCII code).
    /// 2. If present, SWAR (SIMD Within A Register) compares 8 keys at once using 64-bit integers.
    ///
    /// # Arguments
    /// * `key` - The ASCII byte of the key to look up (0..127).
    ///
    /// # Returns
    /// The destination state ID (non-zero), or `0` if no transition exists.
    #[inline]
    pub const fn get_transition(&self, key: u8) -> u32 {
        // O(1) rejection: if the bit is not set, no transition exists.
        let idx = key as usize;
        if self.bitset[idx / 64] & (1u64 << (idx % 64)) == 0 {
            return 0;
        }

        let broadcast = (key as u64) * 0x0101010101010101;

        // Check chunk 0 (keys 0..8) using safe byte slice loading
        let c0 = u64::from_le_bytes([
            self.trans_keys[0],
            self.trans_keys[1],
            self.trans_keys[2],
            self.trans_keys[3],
            self.trans_keys[4],
            self.trans_keys[5],
            self.trans_keys[6],
            self.trans_keys[7],
        ]);
        let v0 = c0 ^ broadcast;
        let m0 = v0.wrapping_sub(0x0101010101010101) & !v0 & 0x8080808080808080;
        if m0 != 0 {
            let offset = (m0.trailing_zeros() / 8) as usize;
            if offset < self.trans_len as usize {
                return self.trans_states[offset];
            }
        }

        // Check chunk 1 (keys 8..16)
        if self.trans_len > 8 {
            let c1 = u64::from_le_bytes([
                self.trans_keys[8],
                self.trans_keys[9],
                self.trans_keys[10],
                self.trans_keys[11],
                self.trans_keys[12],
                self.trans_keys[13],
                self.trans_keys[14],
                self.trans_keys[15],
            ]);
            let v1 = c1 ^ broadcast;
            let m1 = v1.wrapping_sub(0x0101010101010101) & !v1 & 0x8080808080808080;
            if m1 != 0 {
                let offset = 8 + (m1.trailing_zeros() / 8) as usize;
                if offset < self.trans_len as usize {
                    return self.trans_states[offset];
                }
            }
        }

        // Check chunk 2 (keys 16..24)
        if self.trans_len > 16 {
            let c2 = u64::from_le_bytes([
                self.trans_keys[16],
                self.trans_keys[17],
                self.trans_keys[18],
                self.trans_keys[19],
                self.trans_keys[20],
                self.trans_keys[21],
                self.trans_keys[22],
                self.trans_keys[23],
            ]);
            let v2 = c2 ^ broadcast;
            let m2 = v2.wrapping_sub(0x0101010101010101) & !v2 & 0x8080808080808080;
            if m2 != 0 {
                let offset = 16 + (m2.trailing_zeros() / 8) as usize;
                if offset < self.trans_len as usize {
                    return self.trans_states[offset];
                }
            }
        }

        0
    }

    /// Sets or updates a transition from this state on a given key.
    ///
    /// Updates both the 128-bit bitset and the key/state arrays.
    ///
    /// # Arguments
    /// * `key` - The ASCII key trigger.
    /// * `state_id` - The target DFA state ID.
    #[inline]
    pub fn set_transition(&mut self, key: u8, state_id: u32) {
        let idx = key as usize;
        self.bitset[idx / 64] |= 1u64 << (idx % 64);

        let len = self.trans_len as usize;
        for i in 0..len {
            if self.trans_keys[i] == key {
                self.trans_states[i] = state_id;
                return;
            }
        }
        if len < MAX_TRANS {
            self.trans_keys[len] = key;
            self.trans_states[len] = state_id;
            self.trans_len += 1;
        }
    }
}

/// The DFA core that manages states and transitions with Arena Allocation.
///
/// Stores all unique composition states and provides fast $O(1)$ lookups
/// without heap allocation during state transitions.
#[derive(Debug)]
pub struct Dfa {
    /// Array of all DFA states. State 0 is always the initial (empty) state.
    pub states: Vec<State>,
    /// Continuous arena storing transformation slices for all states.
    pub arena: Vec<Transformation>,
    /// Maps composition hash to `state_id` for $O(1)$ deduplication and lookup.
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
///
/// Uses the derived `Hash` impl, which hashes only the initialized fields and
/// never reads struct padding bytes. Hashing raw struct bytes directly would
/// read uninitialized padding (UB, flagged by Miri, and nondeterministic
/// across builds) — so the field-based hash is both correct and Miri-clean.
#[inline]
fn hash_composition(composition: &[Transformation]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    composition.hash(&mut hasher);
    hasher.finish()
}

impl Dfa {
    /// Creates a new DFA with an initial empty state (state ID 0).
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

    /// Returns a reference to the [`State`] with the given ID.
    ///
    /// # Panics
    /// Panics if `id as usize` is out of bounds of the `states` vector.
    pub fn get_state(&self, id: u32) -> &State {
        &self.states[id as usize]
    }

    /// Retrieves the slice of [`Transformation`]s belonging to the specified state.
    pub fn get_composition(&self, state_id: u32) -> &[Transformation] {
        let state = &self.states[state_id as usize];
        let start = state.comp_offset as usize;
        let end = start + state.comp_len as usize;
        &self.arena[start..end]
    }

    /// Adds a new composition state to the DFA or returns the existing state ID if already present.
    ///
    /// Uses the precomputed hash and arena verification for fast collision-free deduplication.
    pub fn add_state(&mut self, composition: &[Transformation]) -> u32 {
        let hash = hash_composition(composition);

        // Fast path: hash match -> verify arena equality (no heap allocation).
        if let Some(&id) = self.hash_to_state.get(&hash) {
            let existing = self.get_composition(id);
            if existing == composition {
                return id;
            }
            // Hash collision -> fall through to add new state.
        }

        let id = self.states.len() as u32;
        let comp_offset = self.arena.len() as u32;
        let comp_len = composition.len() as u8;

        self.arena.extend_from_slice(composition);
        self.states.push(State { comp_hash: hash, comp_offset, comp_len, ..State::default() });

        self.hash_to_state.insert(hash, id);
        id
    }

    /// Finds the state ID corresponding to an existing composition slice, if present.
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

/// A DFA compiler that pre-initializes common syllable states into a [`Dfa`].
pub struct DfaCompiler<'a> {
    /// The input method used for compiling transitions.
    #[allow(dead_code)]
    pub input_method: &'a InputMethod,
    /// Engine configuration.
    #[allow(dead_code)]
    pub config: crate::Config,
    /// The compiled DFA instance.
    pub dfa: Dfa,
    engine: crate::Engine,
}

impl<'a> DfaCompiler<'a> {
    /// Creates a new compiler instance for a given input method and configuration.
    pub fn new(im: &'a InputMethod, config: crate::Config) -> Self {
        let engine = crate::Engine::with_config(im.clone(), config);
        Self { input_method: im, config, dfa: Dfa::new(), engine }
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
                    for &b in f.as_bytes() {
                        buf[pos] = b;
                        pos += 1;
                    }
                    for &b in v.as_bytes() {
                        buf[pos] = b;
                        pos += 1;
                    }
                    for &b in t.as_bytes() {
                        buf[pos] = b;
                        pos += 1;
                    }
                    if let Ok(seq) = std::str::from_utf8(&buf[..pos]) {
                        self.simulate_str(seq);
                    }
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
