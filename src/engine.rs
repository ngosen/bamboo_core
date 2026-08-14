//! The core engine that processes keypresses and maintains the IME state.

use crate::config::Config;
use crate::input_method::{EffectType, InputMethod, Rule};
use crate::mode::{Mode, OutputOptions};
use crate::utils::{is_upper, lower};

/// Maximum number of active transformations in a single syllable.
pub const MAX_ACTIVE_TRANS: usize = 16;

/// A lightweight snapshot of the engine state before a keystroke, used for O(1) backspace.
#[derive(Clone, Copy)]
struct Snapshot {
    active_buffer: [Transformation; MAX_ACTIVE_TRANS],
    active_len: usize,
    current_state_id: u32,
}

/// Represents a single keypress or a transformation derived from it (e.g., adding a mark or tone).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub struct Transformation {
    /// The rule that was applied to create this transformation.
    pub rule: Rule,
    /// The index of the transformation in the composition that this transformation targets (if any).
    /// For example, a tone mark transformation targets an earlier vowel.
    /// Uses u8 since MAX_ACTIVE_TRANS = 16, saving 14 bytes vs `Option<usize>`.
    pub target: Option<u8>,
    /// Whether the resulting character should be rendered as uppercase.
    pub is_upper_case: bool,
}

const _: () = assert!(std::mem::size_of::<Transformation>() <= 28);

/// A stack-allocated buffer for transformations to avoid heap allocations in the hot path.
///
/// This structure uses a fixed-size array and is extremely fast for frequent updates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub struct TransformationStack {
    data: [Transformation; MAX_ACTIVE_TRANS],
    len: usize,
}

impl TransformationStack {
    /// Creates a new, empty transformation stack.
    pub fn new() -> Self {
        Self { data: [Transformation::default(); MAX_ACTIVE_TRANS], len: 0 }
    }

    /// Pushes a new transformation onto the stack.
    /// Does nothing if the stack is full.
    pub fn push(&mut self, t: Transformation) {
        debug_assert!(
            self.len < MAX_ACTIVE_TRANS,
            "TransformationStack overflow: max {MAX_ACTIVE_TRANS} reached"
        );
        if self.len < MAX_ACTIVE_TRANS {
            self.data[self.len] = t;
            self.len += 1;
        }
    }

    /// Removes and returns the last transformation from the stack.
    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<Transformation> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.data[self.len])
        } else {
            None
        }
    }

    /// Clears all transformations from the stack.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Returns the number of transformations currently in the stack.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the stack contains no transformations.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a slice containing all transformations in the stack.
    pub fn as_slice(&self) -> &[Transformation] {
        &self.data[..self.len]
    }

    /// Returns a mutable slice containing all transformations in the stack.
    pub fn as_mut_slice(&mut self) -> &mut [Transformation] {
        &mut self.data[..self.len]
    }

    /// Appends a slice of transformations to the stack.
    pub fn extend_from_slice(&mut self, other: &[Transformation]) {
        let to_copy = other.len().min(MAX_ACTIVE_TRANS - self.len);
        if to_copy > 0 {
            self.data[self.len..self.len + to_copy].copy_from_slice(&other[..to_copy]);
            self.len += to_copy;
        }
    }

    /// Drains transformations from a starting index into another stack.
    pub fn drain_to(&mut self, start: usize, target: &mut TransformationStack) {
        target.clear();
        if start < self.len {
            target.extend_from_slice(&self.data[start..self.len]);
            self.len = start;
        }
    }
}

#[inline]
fn uoh_tail_match(s: &str) -> bool {
    ["uơ", "ưo"].iter().any(|pat| {
        s.find(pat).is_some_and(|idx| {
            s[idx + pat.len()..].chars().next().is_some_and(|c| c.is_alphabetic())
        })
    })
}

/// The main stateful processor of the Vietnamese Input Method Engine.
///
/// It maintains an internal buffer of transformations and produces the correctly marked Vietnamese text.
/// The engine uses a hybrid approach combining a Rule Engine with a Lazy JIT DFA for peak performance.
pub struct Engine {
    committed_text: String,
    /// Stack-allocated buffer for the active composition to avoid heap allocations.
    active_buffer: [Transformation; MAX_ACTIVE_TRANS],
    active_len: usize,

    input_method: InputMethod,
    all_rules: Box<[Rule]>,
    ascii_rule_indices: [(u16, u16); 128],
    non_ascii_rule_indices: Box<[(char, (u16, u16))]>,
    ascii_effect_keys: [bool; 128],
    non_ascii_effect_keys: Vec<char>,
    config: Config,

    // Stack buffers to avoid per-keystroke heap allocations.
    work_comp: TransformationStack,
    scratch_comp: TransformationStack,

    prev_preedit: String,
    delta_buf: String,

    dfa: crate::dfa::Dfa,
    current_state_id: u32,

    // Snapshot stack for O(1) backspace — all stack-allocated, zero heap.
    snapshots: [Snapshot; MAX_ACTIVE_TRANS],
    snapshot_len: usize,

    // Lazily-initialized scratch engine for restore_last_word to avoid repeated with_config.
    scratch_engine: Option<Box<Engine>>,
}

impl Engine {
    /// Creates a new engine with the specified input method and default configuration.
    pub fn new(input_method: InputMethod) -> Self {
        Self::with_config(input_method, Config::default())
    }

    /// Creates a new engine with a specific input method and configuration.
    pub fn with_config(input_method: InputMethod, config: Config) -> Self {
        let mut rules_by_key: std::collections::BTreeMap<char, Vec<Rule>> =
            std::collections::BTreeMap::new();
        for rule in &input_method.rules {
            let key = lower(rule.key);
            rules_by_key.entry(key).or_default().push(*rule);
        }

        let total_rules: usize = rules_by_key.values().map(|v| v.len()).sum();
        let mut all_rules_vec = Vec::with_capacity(total_rules);
        let mut ascii_rule_indices = [(0u16, 0u16); 128];
        let mut non_ascii_indices_vec = Vec::new();

        for (key, rules) in rules_by_key {
            let start = all_rules_vec.len() as u16;
            all_rules_vec.extend(rules);
            let end = all_rules_vec.len() as u16;
            if key.is_ascii() {
                ascii_rule_indices[key as usize] = (start, end);
            } else {
                non_ascii_indices_vec.push((key, (start, end)));
            }
        }

        let mut ascii_effect_keys = [false; 128];
        let mut non_ascii_effect_keys: Vec<char> = Vec::new();
        for key in &input_method.keys {
            if key.is_ascii() {
                ascii_effect_keys[*key as usize] = true;
            } else {
                non_ascii_effect_keys.push(*key);
            }
        }
        non_ascii_effect_keys.sort_unstable();
        non_ascii_effect_keys.dedup();

        Self {
            committed_text: String::with_capacity(128),
            active_buffer: [Transformation::default(); MAX_ACTIVE_TRANS],
            active_len: 0,
            input_method,
            all_rules: all_rules_vec.into_boxed_slice(),
            ascii_rule_indices,
            non_ascii_rule_indices: non_ascii_indices_vec.into_boxed_slice(),
            ascii_effect_keys,
            non_ascii_effect_keys,
            config,

            work_comp: TransformationStack::new(),
            scratch_comp: TransformationStack::new(),

            prev_preedit: String::with_capacity(32),
            delta_buf: String::with_capacity(32),
            dfa: crate::dfa::Dfa::new(),
            current_state_id: 0,

            snapshots: [Snapshot {
                active_buffer: [Transformation::default(); MAX_ACTIVE_TRANS],
                active_len: 0,
                current_state_id: 0,
            }; MAX_ACTIVE_TRANS],
            snapshot_len: 0,
            scratch_engine: None,
        }
    }

    #[inline]
    pub(crate) fn active_slice(&self) -> &[Transformation] {
        &self.active_buffer[..self.active_len]
    }

    fn take_active_into(&mut self, out: &mut TransformationStack) {
        out.clear();
        out.extend_from_slice(self.active_slice());
        self.active_len = 0;
    }

    fn set_active_from_stack(&mut self, src: &mut TransformationStack) {
        self.active_len = src.len().min(MAX_ACTIVE_TRANS);
        self.active_buffer[..self.active_len].copy_from_slice(src.as_slice());
        src.clear();
    }

    #[inline]
    fn push_snapshot(&mut self) {
        if self.snapshot_len < MAX_ACTIVE_TRANS {
            let snap = &mut self.snapshots[self.snapshot_len];
            snap.active_buffer[..self.active_len]
                .copy_from_slice(&self.active_buffer[..self.active_len]);
            snap.active_len = self.active_len;
            snap.current_state_id = self.current_state_id;
            self.snapshot_len += 1;
        }
    }

    #[inline]
    fn pop_snapshot(&mut self) -> Option<()> {
        if self.snapshot_len == 0 {
            return None;
        }
        self.snapshot_len -= 1;
        let snap = &self.snapshots[self.snapshot_len];
        self.active_len = snap.active_len;
        self.active_buffer[..self.active_len]
            .copy_from_slice(&snap.active_buffer[..self.active_len]);
        self.current_state_id = snap.current_state_id;
        Some(())
    }

    /// Returns the current configuration of the engine.
    pub fn config(&self) -> Config {
        self.config
    }

    /// Updates the engine configuration.
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Returns a reference to the current input method.
    pub fn input_method(&self) -> &InputMethod {
        &self.input_method
    }

    /// Warms up the DFA by pre-compiling common Vietnamese syllables.
    ///
    /// This API is intentionally unstable and currently uses a Telex-biased
    /// heuristic corpus. It can help long-lived Telex sessions, but may hurt
    /// cold-start latency or non-Telex/custom input methods.
    ///
    /// Prefer relying on the default lazy JIT behavior unless you have benchmark
    /// data for your production workload.
    #[deprecated(
        since = "0.3.4",
        note = "Engine::warm_up() is unstable and may be removed. It uses a Telex-biased heuristic and may regress cold-start or non-Telex workloads."
    )]
    pub fn warm_up(&mut self) {
        let mut compiler = crate::dfa::DfaCompiler::new(&self.input_method, self.config.to_flags());
        compiler.compile_common();
        self.dfa = compiler.dfa;
        self.current_state_id = 0;
    }

    fn get_applicable_rules(&self, key: char) -> &[Rule] {
        let key = lower(key);
        if key.is_ascii() {
            let (start, end) = self.ascii_rule_indices[key as usize];
            &self.all_rules[start as usize..end as usize]
        } else {
            self.non_ascii_rule_indices
                .binary_search_by_key(&key, |(k, _)| *k)
                .map(|idx| {
                    let (start, end) = self.non_ascii_rule_indices[idx].1;
                    &self.all_rules[start as usize..end as usize]
                })
                .unwrap_or(&[])
        }
    }

    fn can_process_key_raw(&self, lower_key: char) -> bool {
        if crate::utils::is_alpha(lower_key)
            || (lower_key.is_ascii() && self.ascii_effect_keys[lower_key as usize])
            || self.non_ascii_effect_keys.binary_search(&lower_key).is_ok()
        {
            return true;
        }
        if crate::utils::is_word_break_symbol(lower_key) {
            return false;
        }
        crate::utils::is_vietnamese_rune(lower_key)
    }

    fn generate_transformations(
        &self,
        composition: &mut TransformationStack,
        key: char,
        is_upper_case: bool,
    ) {
        let lower_key = lower(key);
        let mut trans_buf = TransformationStack::new();

        crate::bamboo_util::generate_transformations(
            composition.as_slice(),
            self.get_applicable_rules(lower_key),
            self.config.to_flags(),
            lower_key,
            is_upper_case,
            &mut trans_buf,
        );

        if trans_buf.is_empty() {
            crate::bamboo_util::generate_fallback_transformations(
                self.get_applicable_rules(lower_key),
                lower_key,
                is_upper_case,
                &mut trans_buf,
            );

            // Temporary combined data to avoid full struct copy
            let combined_len = composition.len() + trans_buf.len();
            if combined_len <= MAX_ACTIVE_TRANS {
                let mut tmp_data = [Transformation::default(); MAX_ACTIVE_TRANS];
                tmp_data[..composition.len()].copy_from_slice(composition.as_slice());
                tmp_data[composition.len()..combined_len].copy_from_slice(trans_buf.as_slice());

                if !self.input_method.super_keys.is_empty() {
                    let current_str = crate::flattener::flatten_slice(
                        &tmp_data[..combined_len],
                        OutputOptions::TONE_LESS | OutputOptions::LOWER_CASE,
                    );
                    if uoh_tail_match(&current_str) {
                        let (target, rule) = crate::bamboo_util::find_target(
                            &tmp_data[..combined_len],
                            self.get_applicable_rules(self.input_method.super_keys[0]),
                            self.config.to_flags(),
                        );
                        if let (Some(target), Some(mut rule)) = (target, rule) {
                            rule.key = '\0';
                            trans_buf.push(Transformation {
                                rule,
                                target: Some(target),
                                is_upper_case: false,
                            });
                        }
                    }
                }
            }
        }
        composition.extend_from_slice(trans_buf.as_slice());
        if self.config.to_flags() & crate::bamboo_util::EFREE_TONE_MARKING != 0
            && self.is_valid_internal(composition.as_slice(), false)
        {
            let mut extra = TransformationStack::new();
            crate::bamboo_util::refresh_last_tone_target_into(
                composition.as_mut_slice(),
                self.config.to_flags() & crate::bamboo_util::ESTD_TONE_STYLE != 0,
                &mut extra,
            );
            composition.extend_from_slice(extra.as_slice());
        }
    }

    fn last_syllable_start(composition: &[Transformation]) -> usize {
        let mut idx = composition.len();
        let mut last_is_vowel = false;
        let mut found_vowel = false;

        while idx > 0 {
            let tmp = &composition[idx - 1];
            if tmp.target.is_none() {
                let is_v = crate::utils::is_vowel(tmp.rule.result);
                if found_vowel && !is_v && !last_is_vowel {
                    break;
                }
                if is_v {
                    found_vowel = true;
                }
                last_is_vowel = is_v;
            }
            idx -= 1;
        }

        idx
    }

    fn new_composition_in_place(
        &self,
        composition: &mut TransformationStack,
        scratch: &mut TransformationStack,
        key: char,
        is_upper_case: bool,
    ) {
        let syllable_abs_start = Self::last_syllable_start(composition.as_slice());

        composition.drain_to(syllable_abs_start, scratch);

        let offset = syllable_abs_start;
        if offset != 0 {
            for t in scratch.as_mut_slice().iter_mut() {
                if let Some(target) = t.target {
                    t.target = Some(target.saturating_sub(offset as u8));
                }
            }
        }

        self.generate_transformations(scratch, key, is_upper_case);

        if offset != 0 {
            for t in scratch.as_mut_slice().iter_mut() {
                if let Some(target) = t.target {
                    t.target = Some(target + offset as u8);
                }
            }
        }

        composition.extend_from_slice(scratch.as_slice());
    }

    /// Processes a string of characters and returns the resulting active word.
    ///
    /// This is a convenience wrapper around [`Self::process_str`] and [`Self::output`].
    pub fn process(&mut self, s: &str, mode: Mode) -> String {
        self.process_str(s, mode).output()
    }

    /// Processes a string of characters and returns a reference to the engine.
    pub fn process_str(&mut self, s: &str, mode: Mode) -> &Self {
        for key in s.chars() {
            self.process_key(key, mode);
        }
        self
    }

    fn lcp_chars_and_bytes(a: &str, b: &str) -> (usize, usize) {
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();
        let min_len = a_bytes.len().min(b_bytes.len());
        let mut lcp_bytes = 0;

        // 8-byte chunk comparison
        while lcp_bytes + 8 <= min_len {
            let chunk_a = u64::from_ne_bytes(a_bytes[lcp_bytes..lcp_bytes + 8].try_into().unwrap());
            let chunk_b = u64::from_ne_bytes(b_bytes[lcp_bytes..lcp_bytes + 8].try_into().unwrap());
            let diff = chunk_a ^ chunk_b;
            if diff != 0 {
                #[cfg(target_endian = "little")]
                let mismatch_byte = (diff.trailing_zeros() / 8) as usize;
                #[cfg(target_endian = "big")]
                let mismatch_byte = (diff.leading_zeros() / 8) as usize;
                lcp_bytes += mismatch_byte;
                break;
            }
            lcp_bytes += 8;
        }

        while lcp_bytes < min_len && a_bytes[lcp_bytes] == b_bytes[lcp_bytes] {
            lcp_bytes += 1;
        }

        let prefix = &a[..lcp_bytes];
        let lcp_chars = if prefix.is_ascii() { lcp_bytes } else { prefix.chars().count() };
        (lcp_chars, lcp_bytes)
    }

    /// Processes a single key and returns a **3-way diff** for efficient text editor updates.
    ///
    /// Instead of rewriting the entire preedit, the frontend only needs to apply:
    /// 1. Keep the common prefix unchanged.
    /// 2. Delete `backspace_count` characters from the end of the previous preedit.
    /// 3. Append `inserted_suffix`.
    ///
    /// ```text
    /// previous_preedit = [common_prefix] + [backspace_count chars to delete]
    /// new_preedit      = [common_prefix] + [inserted_suffix]
    /// ```
    ///
    /// The common prefix length is implicit: `previous_preedit.len() - backspace_count`
    /// (in characters). The frontend does not need to compute LCP/LCS — the engine does it.
    ///
    /// # Returns
    ///
    /// `(backspace_count, backspaces_bytes, inserted_suffix)`:
    /// - `backspace_count`: Number of **characters** to delete from the end of the previous preedit.
    /// - `backspaces_bytes`: Number of **UTF-8 bytes** to delete (for byte-oriented editors).
    /// - `inserted_suffix`: The new string to append after deletion.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bamboo_core::{Engine, Mode, InputMethod};
    ///
    /// let mut engine = Engine::new(InputMethod::telex());
    ///
    /// let (bs, _, ins) = engine.process_key_delta('a', Mode::Vietnamese);
    /// assert_eq!(bs, 0);
    /// assert_eq!(ins, "a");
    ///
    /// let (bs, _, ins) = engine.process_key_delta('s', Mode::Vietnamese);
    /// // previous = "a", new = "á"
    /// // keep prefix = 1 - 1 = 0, delete = 1 ("a"), insert = "á"
    /// assert_eq!(bs, 1);
    /// assert_eq!(ins, "á");
    /// ```
    pub fn process_key_delta(&mut self, key: char, mode: Mode) -> (usize, usize, &str) {
        self.process_key(key, mode);

        let active_len = self.active_len;
        let active = &self.active_buffer[..active_len];
        crate::flattener::flatten_slice_into(active, OutputOptions::NONE, &mut self.delta_buf);

        let (_prefix_len, lcp_bytes) =
            Self::lcp_chars_and_bytes(&self.prev_preedit, &self.delta_buf);

        let prev_bytes = self.prev_preedit.len();

        // Count only the suffix chars after the common prefix — O(suffix_len) instead of O(total).
        let backspace_count = self.prev_preedit[lcp_bytes..].chars().count();
        let backspaces_bytes = prev_bytes.saturating_sub(lcp_bytes);

        std::mem::swap(&mut self.prev_preedit, &mut self.delta_buf);
        let inserted_suffix = &self.prev_preedit[lcp_bytes..];
        (backspace_count, backspaces_bytes, inserted_suffix)
    }

    /// Similar to [`Self::process_key_delta`], but writes the inserted string into a provided buffer.
    ///
    /// # Returns
    /// `backspace_count` — number of characters to delete from the end of the previous preedit.
    pub fn process_key_delta_into(
        &mut self,
        key: char,
        mode: Mode,
        inserted: &mut String,
    ) -> usize {
        let (backspace_count, _backspaces_bytes, ins) = self.process_key_delta(key, mode);
        inserted.clear();
        inserted.push_str(ins);
        backspace_count
    }

    /// Processes a single character.
    ///
    /// The `mode` determines whether to apply Vietnamese transformation rules.
    pub fn process_key(&mut self, key: char, mode: Mode) {
        let lower_key = lower(key);
        let is_upper_case = is_upper(key);

        // English mode: skip all Vietnamese processing.
        // Direct buffer append — no DFA lookup, no snapshot.
        if mode == Mode::English {
            if crate::utils::is_word_break_symbol(lower_key) && self.active_len > 0 {
                self.commit();
            }
            if self.active_len >= MAX_ACTIVE_TRANS {
                self.commit();
            }
            self.active_buffer[self.active_len] =
                crate::bamboo_util::new_appending_trans(lower_key, is_upper_case);
            self.active_len += 1;
            if crate::utils::is_word_break_symbol(lower_key) {
                self.commit();
            }
            self.current_state_id = 0;
            return;
        }

        // DFA Fast Path: if DFA has a cached transition, key is valid.
        // Skip can_process_key_raw entirely.
        // Uses lowercase key for DFA lookup — uppercase shares the same DFA cache.
        if lower_key.is_ascii() {
            let next_state_id =
                self.dfa.get_state(self.current_state_id).get_transition(lower_key as u8);
            if next_state_id != 0 {
                // Snapshot before overwriting active buffer (skip if empty — nothing to restore).
                if self.active_len > 0 {
                    self.push_snapshot();
                }
                self.current_state_id = next_state_id;
                let comp = self.dfa.get_composition(next_state_id);
                self.active_len = comp.len().min(MAX_ACTIVE_TRANS);
                self.active_buffer[..self.active_len].copy_from_slice(comp);
                // Restore uppercase flag on first transformation if original key was uppercase.
                // DFA stores lowercase compositions; we adjust case at output time.
                if is_upper_case && self.active_len > 0 {
                    self.active_buffer[0].is_upper_case = true;
                }
                return;
            }
        }

        // Slow path: validate key and handle word breaks
        if !self.can_process_key_raw(lower_key) {
            if crate::utils::is_word_break_symbol(lower_key) {
                self.commit();
            }
            // Snapshot before push_active so backspace can restore previous state.
            // Word breaks trigger commit() which clears snapshots — that's correct
            // (committed text can't be undone via backspace).
            if self.active_len > 0 {
                self.push_snapshot();
            }
            let trans = crate::bamboo_util::new_appending_trans(lower_key, is_upper_case);
            self.push_active(trans);
            if crate::utils::is_word_break_symbol(lower_key) {
                self.commit();
            }
            self.current_state_id = 0;
            return;
        }

        // Snapshot only needed before slow-path mutations (new_composition_in_place).
        self.push_snapshot();

        let mut work = self.work_comp;
        let mut scratch = self.scratch_comp;

        self.take_active_into(&mut work);
        self.new_composition_in_place(&mut work, &mut scratch, lower_key, is_upper_case);

        // Try to update DFA (Lazy JIT).
        // Always cache using lowercase key so uppercase keys reuse the same DFA transitions.
        if lower_key.is_ascii() && work.len() <= MAX_ACTIVE_TRANS {
            // For uppercase: create a lowercase copy of the composition for DFA caching.
            // This ensures both 'a' and 'A' share the same DFA transition from the same state.
            let cache_comp: &[Transformation] = if is_upper_case {
                self.scratch_comp.clear();
                self.scratch_comp.extend_from_slice(work.as_slice());
                if !self.scratch_comp.is_empty() {
                    self.scratch_comp.as_mut_slice()[0].is_upper_case = false;
                }
                self.scratch_comp.as_slice()
            } else {
                work.as_slice()
            };
            let next_id = self.dfa.add_state(cache_comp);
            self.dfa.states[self.current_state_id as usize]
                .set_transition(lower_key as u8, next_id);
            self.current_state_id = next_id;
        } else {
            self.current_state_id = self.dfa.find_state(work.as_slice()).unwrap_or(0);
        }

        self.set_active_from_stack(&mut work);

        self.work_comp = work;
        self.scratch_comp = scratch;
    }

    fn push_active(&mut self, trans: Transformation) {
        if self.active_len >= MAX_ACTIVE_TRANS {
            // Buffer full, auto-commit to make room
            self.commit();
        }
        self.active_buffer[self.active_len] = trans;
        self.active_len += 1;
        self.current_state_id = self.dfa.find_state(self.active_slice()).unwrap_or(0);
    }

    /// Clears the active syllable buffer and appends it to the committed text.
    pub fn commit(&mut self) {
        if self.active_len == 0 {
            return;
        }
        // Copy active transformations to stack buffer to avoid borrow conflict.
        // This is a fixed 448-byte memcpy (16 × 28 bytes) — no heap allocation.
        // Still better than the old approach which allocated a String via output().
        let mut comp = [Transformation::default(); MAX_ACTIVE_TRANS];
        comp[..self.active_len].copy_from_slice(&self.active_buffer[..self.active_len]);
        let len = self.active_len;
        self.active_len = 0;
        crate::flattener::append_flatten_slice(
            &comp[..len],
            OutputOptions::NONE,
            &mut self.committed_text,
        );
        self.current_state_id = 0;
        self.snapshot_len = 0;
    }

    /// Returns the currently active syllable as a string.
    pub fn output(&self) -> String {
        crate::flattener::flatten_slice(self.active_slice(), OutputOptions::NONE)
    }

    /// Returns the processed string according to the specified options.
    ///
    /// This can be used to get the full text (committed + active) or variations like toneless text.
    pub fn get_processed_str(&self, options: OutputOptions) -> String {
        let active = self.active_slice();
        if options.contains(OutputOptions::FULL_TEXT) {
            if active.is_empty() {
                return self.committed_text.clone();
            }
            let mut result = String::with_capacity(self.committed_text.len() + active.len() * 4);
            result.push_str(&self.committed_text);
            crate::flattener::append_flatten_slice(active, options, &mut result);
            return result;
        }
        if options.contains(OutputOptions::PUNCTUATION_MODE) {
            if active.is_empty() {
                return String::new();
            }
            let (_, tail) = crate::bamboo_util::extract_last_word_with_punctuation_marks(
                active,
                &self.input_method.keys,
            );
            return crate::flattener::flatten_slice(tail, OutputOptions::NONE);
        }
        crate::flattener::flatten_slice(active, options)
    }

    /// Checks if the current composition forms a valid Vietnamese syllable.
    pub fn is_valid(&self, input_is_full_complete: bool) -> bool {
        self.is_valid_internal(self.active_slice(), input_is_full_complete)
    }

    fn is_valid_internal(
        &self,
        composition: &[Transformation],
        input_is_full_complete: bool,
    ) -> bool {
        crate::bamboo_util::is_valid(composition, input_is_full_complete)
    }

    /// Restores the last word in the composition to its un-transformed state.
    ///
    /// If `to_vietnamese` is true, it attempts to re-apply Vietnamese transformations.
    pub fn restore_last_word(&mut self, to_vietnamese: bool) {
        let mut work = self.work_comp;

        self.take_active_into(&mut work);
        if work.is_empty() {
            self.set_active_from_stack(&mut work);
            self.current_state_id = 0;
            return;
        }

        let (prev_slice, last) =
            crate::bamboo_util::extract_last_word(work.as_slice(), Some(&self.input_method.keys));

        let mut previous = TransformationStack::new();
        previous.extend_from_slice(prev_slice);

        if last.is_empty() {
            self.set_active_from_stack(&mut work);
            self.current_state_id = 0;
            return;
        }
        if !to_vietnamese {
            previous.extend_from_slice(&crate::bamboo_util::break_composition_slice(last));
            self.set_active_from_stack(&mut previous);
            self.current_state_id = 0;
            return;
        }

        let mut new_comp = TransformationStack::new();
        if self.scratch_engine.is_none() {
            self.scratch_engine =
                Some(Box::new(Self::with_config(self.input_method.clone(), self.config)));
        }
        let temp_engine = self.scratch_engine.as_mut().unwrap();
        temp_engine.reset();

        for t in last {
            if t.rule.key == '\0' {
                continue;
            }
            temp_engine.process_key(t.rule.key, Mode::Vietnamese);
        }
        new_comp.extend_from_slice(temp_engine.active_slice());

        previous.extend_from_slice(new_comp.as_slice());

        self.set_active_from_stack(&mut previous);
        self.current_state_id = 0;
    }

    /// Removes the last character from the active composition.
    pub fn remove_last_char(&mut self, refresh_last_tone_target: bool) {
        if self.pop_snapshot().is_none() {
            return;
        }

        if refresh_last_tone_target && self.active_len > 0 {
            let mut extra = TransformationStack::new();
            crate::bamboo_util::refresh_last_tone_target_into(
                &mut self.active_buffer[..self.active_len],
                self.config.to_flags() & crate::bamboo_util::ESTD_TONE_STYLE != 0,
                &mut extra,
            );
            let available = MAX_ACTIVE_TRANS - self.active_len;
            let to_copy = extra.len().min(available);
            if to_copy > 0 {
                self.active_buffer[self.active_len..self.active_len + to_copy]
                    .copy_from_slice(&extra.as_slice()[..to_copy]);
                self.active_len += to_copy;
            }
        }
    }

    /// Removes the last output character (grapheme) from the active composition,
    /// keeping mark/tone transformations on earlier characters intact.
    ///
    /// No-op if the composition is empty. Invalidates the keystroke snapshot stack
    /// and the DFA fast-path state.
    pub fn remove_last_output_char(&mut self) {
        let last = self
            .active_slice()
            .iter()
            .enumerate()
            .rev()
            .find(|(_, t)| t.rule.effect_type == EffectType::Appending && t.rule.key != '\0');
        let Some((l, _)) = last else { return };

        // Compact the buffer, dropping the grapheme at `l` together with every
        // transformation targeting it. Transformations targeting earlier graphemes
        // keep their positions (and targets) — they are what survives the delete.
        let mut write = 0;
        for read in 0..self.active_len {
            let t = self.active_buffer[read];
            if read == l || t.target == Some(l as u8) {
                continue;
            }
            if write != read {
                self.active_buffer[write] = t;
            }
            write += 1;
        }
        self.active_len = write;

        // Keystroke snapshots are stale after compaction; the DFA fast path no longer matches.
        self.snapshot_len = 0;
        self.current_state_id = 0;

        if self.active_len > 0 {
            let mut extra = TransformationStack::new();
            crate::bamboo_util::refresh_last_tone_target_into(
                &mut self.active_buffer[..self.active_len],
                self.config.to_flags() & crate::bamboo_util::ESTD_TONE_STYLE != 0,
                &mut extra,
            );
            let available = MAX_ACTIVE_TRANS - self.active_len;
            let to_copy = extra.len().min(available);
            if to_copy > 0 {
                self.active_buffer[self.active_len..self.active_len + to_copy]
                    .copy_from_slice(&extra.as_slice()[..to_copy]);
                self.active_len += to_copy;
            }
        }
    }

    /// Resets the engine state, clearing committed and active text.
    pub fn reset(&mut self) {
        self.committed_text.clear();
        self.active_len = 0;
        self.prev_preedit.clear();
        self.delta_buf.clear();
        self.current_state_id = 0;
        self.snapshot_len = 0;
    }

    /// Returns the number of DFA states currently cached.
    pub fn dfa_state_count(&self) -> usize {
        self.dfa.states.len()
    }

    /// Returns the number of Transformations stored in the DFA arena.
    pub fn dfa_arena_len(&self) -> usize {
        self.dfa.arena.len()
    }

    /// Returns the number of entries in the DFA composition-to-state map.
    pub fn dfa_composition_count(&self) -> usize {
        self.dfa.hash_to_state.len()
    }

    /// Returns the capacity (in bytes) of the committed_text buffer.
    pub fn committed_text_capacity(&self) -> usize {
        self.committed_text.capacity()
    }

    /// Returns the number of active transformations in the current syllable.
    pub fn active_len(&self) -> usize {
        self.active_len
    }

    /// Returns the number of snapshots stored for backspace.
    pub fn snapshot_len(&self) -> usize {
        self.snapshot_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_backspaces_and_inserted() {
        let telex = InputMethod::telex();
        let mut e = Engine::new(telex);

        let (bs1, _bb1, ins1) = e.process_key_delta('a', Mode::Vietnamese);
        assert_eq!(bs1, 0, "First 'a' should have 0 backspaces");
        assert_eq!(ins1, "a");

        let (bs2, _bb2, ins2) = e.process_key_delta('s', Mode::Vietnamese);
        assert_eq!(bs2, 1, "Adding 's' to 'a' should have 1 backspace for 'á'");
        assert_eq!(ins2, "á");

        let (bs3, _bb3, ins3) = e.process_key_delta(' ', Mode::Vietnamese);
        assert_eq!(bs3, 1, "Space should clear the preedit 'á'");
        assert_eq!(ins3, "");
    }

    #[test]
    fn remove_last_output_char_telex() {
        let mut e = Engine::new(InputMethod::telex());

        // `tiếng` -> drops `g`, keeps the `s` tone on `ê`.
        e.process_str("tieesng", Mode::Vietnamese);
        assert_eq!(e.output(), "tiếng");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiến");

        // Same shape with plain `e`: `tiéng` -> `tién`.
        e.reset();
        e.process_str("tiengs", Mode::Vietnamese);
        assert_eq!(e.output(), "tiéng");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tién");

        // `việt` -> drops `t`, keeps ê mark and nặng tone.
        e.reset();
        e.process_str("vietej", Mode::Vietnamese);
        assert_eq!(e.output(), "việt");
        e.remove_last_output_char();
        assert_eq!(e.output(), "việ");

        // Tone targets the last grapheme: both are dropped.
        e.reset();
        e.process_str("baf", Mode::Vietnamese);
        assert_eq!(e.output(), "bà");
        e.remove_last_output_char();
        assert_eq!(e.output(), "b");

        // Doubled letters: `â` -> `""`.
        e.reset();
        e.process_str("aa", Mode::Vietnamese);
        assert_eq!(e.output(), "â");
        e.remove_last_output_char();
        assert_eq!(e.output(), "");

        // `đ` -> `""`.
        e.reset();
        e.process_str("dd", Mode::Vietnamese);
        assert_eq!(e.output(), "đ");
        e.remove_last_output_char();
        assert_eq!(e.output(), "");

        // Tone typed early: still drops `g`, keeps tone.
        e.reset();
        e.process_str("tieesng", Mode::Vietnamese);
        assert_eq!(e.output(), "tiếng");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiến");

        // Empty composition: no-op.
        e.reset();
        e.remove_last_output_char();
        assert_eq!(e.output(), "");
    }

    #[test]
    fn remove_last_output_char_vni() {
        let mut e = Engine::new(InputMethod::vni());

        // `việt` -> `việ`.
        e.process_str("viet65", Mode::Vietnamese);
        assert_eq!(e.output(), "việt");
        e.remove_last_output_char();
        assert_eq!(e.output(), "việ");

        // `bà` -> `b`.
        e.reset();
        e.process_str("ba2", Mode::Vietnamese);
        assert_eq!(e.output(), "bà");
        e.remove_last_output_char();
        assert_eq!(e.output(), "b");
    }

    #[test]
    fn remove_last_output_char_invalidates_snapshots() {
        let mut e = Engine::new(InputMethod::telex());

        e.process_str("tiếng", Mode::Vietnamese);
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiến");

        // remove_last_char must not "undo" past the grapheme delete to a stale snapshot.
        e.remove_last_char(true);
        assert_eq!(e.output(), "tiến");
    }

    #[test]
    fn compose_after_remove_last_output_char() {
        let mut e = Engine::new(InputMethod::telex());

        // `toàn`: the huyền tone targets the vowel `a`, not the final consonant `n`,
        // so deleting `n` keeps the tone: `toàn` -> `tòa`.
        e.process_str("toanf", Mode::Vietnamese);
        assert_eq!(e.output(), "toàn");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tòa");

        // The engine must accept keystrokes normally after compaction:
        // re-typing the deleted consonant and the tone key recomposes the word.
        e.process_key('n', Mode::Vietnamese);
        e.process_key('f', Mode::Vietnamese);
        assert_eq!(e.output(), "toàn");

        // And keep composing fresh words after a commit.
        e.process_key(' ', Mode::Vietnamese);
        e.process_key('a', Mode::Vietnamese);
        e.process_key('n', Mode::Vietnamese);
        e.process_key('h', Mode::Vietnamese);
        assert_eq!(e.output(), "anh");
    }

    #[test]
    fn remove_last_output_char_double_delete() {
        let mut e = Engine::new(InputMethod::telex());

        // Stress-test compaction on an already-compacted buffer.
        e.process_str("tieesng", Mode::Vietnamese);
        assert_eq!(e.output(), "tiếng");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiến");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiế");

        // Deleting past the end is a no-op.
        e.reset();
        e.process_str("baf", Mode::Vietnamese);
        assert_eq!(e.output(), "bà");
        e.remove_last_output_char();
        assert_eq!(e.output(), "b");
        e.remove_last_output_char();
        assert_eq!(e.output(), "");
        e.remove_last_output_char();
        assert_eq!(e.output(), "");
    }
}
