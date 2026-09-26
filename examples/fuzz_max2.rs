//! Deep fuzz round 2: pure-letter unique (no digits/underscores) to maximize states/key,
//! cross 65k u16 boundary, compare presets.
use bamboo_core::{Config, Engine, InputMethod, Mode};

fn fmt_b(b: usize) -> String {
    if b < 1024 {
        format!("{}B", b)
    } else if b < 1024 * 1024 {
        format!("{:.1}KB", b as f64 / 1024.0)
    } else {
        format!("{:.2}MB", b as f64 / 1024.0 / 1024.0)
    }
}
fn report(tag: &str, e: &Engine, keys: usize) {
    println!(
        "{:<34} keys={:<8} states={:<7} (cap {}) arena={:<7} (cap {}) flat={:<7} map={:<6} (cap {}) used={:<8} alloc={:<8} text={}",
        tag,
        keys,
        e.dfa_state_count(),
        e.dfa_states_capacity(),
        e.dfa_arena_len(),
        e.dfa_arena_capacity(),
        e.dfa_flat_len(),
        e.dfa_composition_count(),
        e.dfa_map_capacity(),
        fmt_b(e.dfa_memory_used()),
        fmt_b(e.dfa_memory_allocated()),
        fmt_b(e.committed_text_len()),
    );
}

// pure letters only, len 10, base-19 consonants -> every key stays on DFA path (no digits/state0)
fn pure_word(i: usize) -> String {
    const ALPH: &[u8] = b"bcdfghjklmnpqrstvwxz"; // 19
    let mut x = i;
    let mut s = String::with_capacity(10);
    for _ in 0..10 {
        s.push(ALPH[x % ALPH.len()] as char);
        x /= ALPH.len();
    }
    s
}
// tone-heavy invalid: valid onset + vowel + double tone (triggers undo/bypass path)
fn tone_spam_word(i: usize) -> String {
    const V: &[&str] = &["a", "e", "i", "o", "u"];
    format!("thr{}{}{}{}", V[i % 5], "ss", "xx", &pure_word(i)[..2])
}

fn run_pure(im: InputMethod, label: &str, cfg: Config, n: usize) {
    let mut e = Engine::with_config(im, cfg);
    let mut keys = 0;
    for i in 0..n {
        let w = pure_word(i);
        for c in w.chars() {
            e.process_key(c, Mode::Vietnamese);
            keys += 1;
        }
        e.process_key(' ', Mode::Vietnamese);
        keys += 1;
        if [10_000, 30_000, 60_000, 100_000, 130_000, 150_000].contains(&(i + 1)) {
            report(&format!("{} {}w", label, i + 1), &e, keys);
        }
    }
    if ![10_000, 30_000, 60_000, 100_000, 130_000, 150_000].contains(&n) {
        report(&format!("{} {}w FINAL", label, n), &e, keys);
    }
    println!("    rate {:.4} states/key", e.dfa_state_count() as f64 / keys as f64);
}

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(150_000);
    println!("=== FUZZ2 pure-letter N={} ===", n);

    let cfg_on = Config::default();
    let cfg_off = Config::builder().auto_correct(false).build();

    // Telex pure OFF (absolute worst)
    run_pure(InputMethod::telex(), "telex-OFF-pure", cfg_off, n);
    // Telex pure ON
    run_pure(InputMethod::telex(), "telex-ON-pure", cfg_on, n);

    // Combined preset worst OFF (widest alphabet)
    {
        let mut e = Engine::with_config(InputMethod::telex_vni_viqr(), cfg_off);
        let mut keys = 0;
        let m = (n / 3).max(30_000);
        for i in 0..m {
            // mix letters + digits (VNI tones) + viqr punct
            let w = format!("{}1{}{}", &pure_word(i)[..5], i % 10, &pure_word(i + 99999)[..3]);
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
        }
        report(&format!("telexvni-viqr-OFF-mix {}w", m), &e, keys);
    }

    // Tone-spam (undo path) ON vs OFF
    for (cfg, lbl) in [(cfg_on, "tone-spam-ON"), (cfg_off, "tone-spam-OFF")] {
        let mut e = Engine::with_config(InputMethod::telex(), cfg);
        let mut keys = 0;
        let m = 20_000;
        for i in 0..m {
            let w = tone_spam_word(i);
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
        }
        e.commit();
        report(&format!("{} {}w", lbl, m), &e, keys);
    }

    // Uppercase: does it double?
    {
        let mut e = Engine::with_config(InputMethod::telex(), cfg_off);
        let mut keys = 0;
        for i in 0..10_000 {
            let w = pure_word(i).to_uppercase();
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
        }
        report("telex-OFF-UPPER 10kw", &e, keys);
    }

    // Beyond-u16 check: transitions dropped but states keep growing?
    {
        let mut e = Engine::with_config(InputMethod::telex(), cfg_off);
        for i in 0..70_000 {
            let w = pure_word(i);
            e.process_str(&w, Mode::Vietnamese);
            e.process_key(' ', Mode::Vietnamese);
        }
        e.commit();
        println!(
            "beyond-u16 probe: states={} (>65536 means Vec grows past u16 cap)",
            e.dfa_state_count()
        );
    }
}
