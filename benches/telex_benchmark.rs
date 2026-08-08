#![allow(deprecated, unused)]
use bamboo_core::{Engine, InputMethod, Mode};
use skey_engine::{SkeyEngine, Method};
use std::time::Instant;

const WARMUP_ITERS: usize = 10_000;
const BENCH_ITERS: usize = 500_000;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn bench_trio<F1, F2, F3>(
    label: &str,
    mut bamboo_per_key: F1,
    mut bamboo_batch: F2,
    mut skey_batch: F3,
) where
    F1: FnMut(),
    F2: FnMut(),
    F3: FnMut(),
{
    for _ in 0..WARMUP_ITERS {
        bamboo_per_key();
        bamboo_batch();
        skey_batch();
    }

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bamboo_per_key();
    }
    let bamboo_pk_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bamboo_batch();
    }
    let bamboo_batch_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        skey_batch();
    }
    let skey_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    let ratio_pk = if skey_ns > 0.0 { bamboo_pk_ns / skey_ns } else { 0.0 };
    let ratio_batch = if skey_ns > 0.0 { bamboo_batch_ns / skey_ns } else { 0.0 };

    println!(
        "{:<45} bamboo/key: {:>8.1} ns | bamboo/batch: {:>8.1} ns | skey: {:>8.1} ns | pk: {:.2}x | batch: {:.2}x",
        label, bamboo_pk_ns, bamboo_batch_ns, skey_ns, ratio_pk, ratio_batch
    );
}

fn bench_pair<F1, F2>(label: &str, mut bamboo_fn: F1, mut skey_fn: F2)
where
    F1: FnMut(),
    F2: FnMut(),
{
    for _ in 0..WARMUP_ITERS {
        bamboo_fn();
        skey_fn();
    }

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bamboo_fn();
    }
    let bamboo_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        skey_fn();
    }
    let skey_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    let ratio = if skey_ns > 0.0 { bamboo_ns / skey_ns } else { 0.0 };
    println!(
        "{:<45} bamboo: {:>8.1} ns | skey: {:>8.1} ns | ratio: {:.2}x",
        label, bamboo_ns, skey_ns, ratio
    );
}

/// Create two pre-warmed bamboo engines (per-key and batch) + one skey engine.
macro_rules! setup_engines {
    () => {{
        let mut bp = Engine::new(InputMethod::telex());
        bp.warm_up();
        let mut bb = Engine::new(InputMethod::telex());
        bb.warm_up();
        let s = SkeyEngine::new(Method::Telex);
        (bp, bb, s)
    }};
}

// ===========================================================================
// Category 1: Single syllable — basic tones
// ===========================================================================

fn bench_single_tones() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("vieetj", "việt"),
        ("viẹtj", "việt"),
        ("tieengs", "tiếng"),
        ("hoof", "hồ"),
        ("nguwowif", "người"),
        ("hoax", "hoã"),
        ("chooj", "chọ"),
        ("laaj", "lạ"),
        ("muaws", "muấ"),
        ("duowif", "dứơi"),
    ];

    println!("\n=== Category 1: Single syllable — basic Telex tones ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("tone: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 2: Single syllable — marks
// ===========================================================================

fn bench_single_marks() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("aa", "â"), ("aw", "ă"), ("oo", "ô"), ("ow", "ơ"), ("uw", "ư"),
        ("ee", "ê"), ("dd", "đ"), ("AA", "Â"), ("AW", "Ă"), ("OO", "Ô"),
        ("OW", "Ơ"), ("UW", "Ư"), ("DD", "Đ"), ("aws", "ấ"), ("owf", "ờ"),
        ("uwr", "ử"), ("aax", "ẵ"), ("eej", "ệ"),
    ];

    println!("\n=== Category 2: Single syllable — Telex marks ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("mark: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 3: Common Vietnamese words
// ===========================================================================

fn bench_common_words() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("tieengs", "tiếng"), ("vietj", "việt"), ("huowng", "hương"),
        ("quoocs", "quốc"), ("nguwowif", "người"), ("namf", "nàm"),
        ("chuyeenn", "chuyện"), ("thuyeet", "thuyết"), ("truwowjt", "trước"),
        ("nghieengs", "nghiếng"), ("hoas", "hóa"), ("khongf", "không"),
        ("duowcj", "được"), ("nhuwngf", "nhưng"), ("moiw", "mới"),
        ("laaj", "lạ"), ("tuoif", "tuổi"), ("troiwf", "trời"),
        ("doocs", "độc"), ("muaws", "muấ"), ("phuj", "phụ"),
        ("quyf", "quỳ"), ("ngux", "ngũ"), ("hoax", "hoã"),
        ("buwows", "bướ"), ("ngoif", "ngồi"), ("lamf", "làm"),
        ("anhf", "ành"), ("buocj", "bước"), ("duownjg", "đường"),
    ];

    println!("\n=== Category 3: Common Vietnamese words (Telex) ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("word: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 4: Compound marks (mark + tone)
// ===========================================================================

fn bench_compound_marks() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("aws", "ấ"), ("owf", "ờ"), ("uwr", "ử"), ("aax", "ẵ"),
        ("ooj", "ộ"), ("eef", "ề"), ("ees", "ế"), ("owr", "ở"),
        ("uws", "ứ"), ("uwf", "ừ"), ("uwx", "ữ"), ("uwj", "ự"),
        ("aas", "ấ"), ("aaf", "ầ"), ("aar", "ẩ"), ("aaj", "ậ"),
        ("oos", "ố"), ("oof", "ồ"), ("oor", "ổ"), ("ooj", "ộ"),
    ];

    println!("\n=== Category 4: Compound marks (mark + tone) ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("compound: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 5: Tone replacement
// ===========================================================================

fn bench_tone_replacement() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("vieetjs", "viếts"),
        ("hooff", "hồf"),
        ("tieengss", "tiếngs"),
    ];

    println!("\n=== Category 5: Tone replacement scenarios ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("replace: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 6: Double-vowel toggle (triple-press undo)
// ===========================================================================

fn bench_double_vowel_toggle() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("oo", "ô"), ("ooo", "oo"), ("oooo", "ôô"),
        ("ee", "ê"), ("eee", "ee"),
        ("aa", "â"), ("aaa", "aa"),
        ("dd", "đ"), ("ddd", "dd"),
        ("vayaj", "vầy"),
    ];

    println!("\n=== Category 6: Double-vowel toggle ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("toggle: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 7: w key (horn/breve modifier)
// ===========================================================================

fn bench_w_key() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("aw", "ă"), ("ow", "ơ"), ("uw", "ư"),
        ("toiw", "tơi"), ("muaw", "muă"),
        ("huowng", "hương"), ("buwocj", "bước"),
        ("suowng", "sương"), ("duowng", "đường"),
        ("muaf", "mùa"), ("tuoi", "tuoi"),
    ];

    println!("\n=== Category 7: w key (horn/breve) ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("w-key: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 8: Capitalization
// ===========================================================================

fn bench_capitalization() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("Vietj", "Việt"), ("VIETJ", "VIỆT"), ("vIeTj", "vIệT"),
        ("HOAX", "HOÃ"), ("Tieengs", "Tiếng"), ("NGUOWIF", "NGƯỜI"),
        ("Chuyeenn", "Chuyện"), ("DD", "Đ"), ("Dd", "Đ"),
    ];

    println!("\n=== Category 8: Capitalization preservation ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("caps: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 9: qu / gi digraphs
// ===========================================================================

fn bench_digraphs() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("quoocs", "quốc"), ("quaws", "quấ"),
        ("gif", "gì"), ("gij", "gị"),
        ("quyens", "quyến"), ("quyef", "quyề"),
        ("gias", "giá"), ("giaf", "già"),
    ];

    println!("\n=== Category 9: qu/gi digraphs ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("digraph: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 10: Sentence-level
// ===========================================================================

fn bench_sentences() {
    let (mut bp, mut bb, s) = setup_engines!();

    let sentences: &[(&str, &str)] = &[
        ("hom nay troij depf qua", "hôm nay trời đẹp quá"),
        ("toi dang hoc lap trinhr", "tôi đang học lập trình"),
        ("tiengf vietj ratj depf", "tiếng việt rất đẹp"),
        ("chao mowf banf ddenf vieetj namf", "chào mừng bạn đến việt nam"),
        ("tuoif treo hom nayf hocj ratj chams", "tuổi trẻ hôm nay học rất chăm"),
    ];

    println!("\n=== Category 10: Sentence-level (multi-word) ===");
    for &(input, expected) in sentences {
        let inp = input.to_string();
        bench_trio(
            &format!("sentence ({} chars)", input.len()),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 11: Long paragraph
// ===========================================================================

fn bench_long_paragraph() {
    let (mut bp, mut bb, s) = setup_engines!();

    let paragraph = "hom nay toi di hocj, troijf ratj depf. toi thifcj thuwf \
                     vieecj hocj lapj trinhr, ddao nay dang hoc ngon nguwr Rust. \
                     ngon nguwr nay ratj hayf va huuw ichj, tuy nhifeng banj ddaauf \
                     hoi kho hocc. nhungf sauj khi hocj duowcfj motf thoif gianf, \
                     toi thayf no thuwcfj suwfj ratj tuyetj vowif.";

    println!("\n=== Category 11: Long paragraph ({} chars) ===", paragraph.len());
    bench_trio(
        &format!("paragraph ({} chars)", paragraph.len()),
        || { bp.reset(); for c in paragraph.chars() { bp.process_key(c, Mode::Vietnamese); } },
        || { bb.reset(); bb.process_str(paragraph, Mode::Vietnamese); },
        || { s.transform(paragraph); },
    );
}

// ===========================================================================
// Category 12: Cold start (fresh instance per call)
// ===========================================================================

fn bench_cold_start() {
    let skey = SkeyEngine::new(Method::Telex);
    let input = "tieengs vietj huowng";

    println!("\n=== Category 12: Cold start (fresh instance per call) ===");
    bench_pair(
        &format!("cold start ({} chars)", input.len()),
        || {
            let mut e = Engine::new(InputMethod::telex());
            for c in input.chars() { e.process_key(c, Mode::Vietnamese); }
        },
        || { skey.transform(input); },
    );
}

// ===========================================================================
// Category 13: Backspace (bamboo-core only)
// ===========================================================================

fn bench_backspace_only() {
    let mut bp = Engine::new(InputMethod::telex());
    bp.warm_up();
    let input = "tieengs";

    println!("\n=== Category 13: Backspace (bamboo-core only) ===");

    for _ in 0..WARMUP_ITERS {
        bp.process_str(input, Mode::Vietnamese);
        bp.remove_last_char(true);
        bp.reset();
    }
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bp.process_str(input, Mode::Vietnamese);
        bp.remove_last_char(true);
        bp.reset();
    }
    let ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;
    println!("{:<45} bamboo: {:>8.1} ns | (skey: N/A)", "backspace x1 (tieengs)", ns);

    for _ in 0..WARMUP_ITERS {
        bp.process_str(input, Mode::Vietnamese);
        for _ in 0..7 { bp.remove_last_char(true); }
        bp.reset();
    }
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bp.process_str(input, Mode::Vietnamese);
        for _ in 0..7 { bp.remove_last_char(true); }
        bp.reset();
    }
    let ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;
    println!("{:<45} bamboo: {:>8.1} ns | (skey: N/A)", "backspace x7 clear (tieengs)", ns);
}

// ===========================================================================
// Category 14: Delta API (bamboo-core only)
// ===========================================================================

fn bench_delta_api() {
    let mut bp = Engine::new(InputMethod::telex());
    bp.warm_up();
    let keys: Vec<char> = "tieengs".chars().collect();

    println!("\n=== Category 14: Delta API (bamboo-core only) ===");

    for _ in 0..WARMUP_ITERS {
        bp.reset();
        for &k in &keys { let _ = bp.process_key_delta(k, Mode::Vietnamese); }
    }
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bp.reset();
        for &k in &keys { let _ = bp.process_key_delta(k, Mode::Vietnamese); }
    }
    let ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;
    println!("{:<45} bamboo: {:>8.1} ns | (skey: N/A)", "delta API (tieengs, 7 keys)", ns);
}

// ===========================================================================
// Category 15: Scalability — increasing input length
// ===========================================================================

fn bench_scalability() {
    let (mut bp, mut bb, s) = setup_engines!();

    println!("\n=== Category 15: Scalability (input length) ===");

    let words = [
        "tieengs", "vietj", "huowng", "quoocs", "nguwowif",
        "namf", "chuyeenn", "thuyeet", "truwowjt", "nghieengs",
        "hoas", "khongf", "duowcj", "nhuwngf", "moiw",
    ];

    for &word_count in &[1usize, 3, 5, 10, 15] {
        let mut sentence = String::new();
        for i in 0..word_count {
            if i > 0 { sentence.push(' '); }
            sentence.push_str(words[i % words.len()]);
        }
        let label = format!("{} words ({} chars)", word_count, sentence.len());
        let inp = sentence.clone();

        bench_trio(
            &label,
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 16: English passthrough
// ===========================================================================

fn bench_english_passthrough() {
    let (mut bp, mut bb, s) = setup_engines!();

    let inputs = [
        "hello world",
        "fn main() { println!(\"hi\"); }",
        "let mut count = 0;",
        "very_long_variable_name_that_keeps_going",
        "const MAX_BUFFER_SIZE: usize = 4096;",
    ];

    println!("\n=== Category 16: English passthrough ===");
    for input in &inputs {
        let inp = input.to_string();
        bench_pair(
            &format!("english ({} chars)", input.len()),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Category 17: Stress test — repeated patterns
// ===========================================================================

fn bench_stress_patterns() {
    let (mut bp, mut bb, s) = setup_engines!();

    println!("\n=== Category 17: Stress test — repeated patterns ===");

    let p1 = "tieengs ".repeat(20);
    let inp1 = p1.clone();
    bench_trio(
        &format!("repeat 'tieengs ' x20 ({} chars)", p1.len()),
        || { bp.reset(); for c in inp1.chars() { bp.process_key(c, Mode::Vietnamese); } },
        || { bb.reset(); bb.process_str(&inp1, Mode::Vietnamese); },
        || { s.transform(&inp1); },
    );

    let words = ["tieengs", "vietj", "huowng", "quoocs", "nguwowif"];
    let mut p2 = String::new();
    for i in 0..50 {
        if i > 0 { p2.push(' '); }
        p2.push_str(words[i % words.len()]);
    }
    let inp2 = p2.clone();
    bench_trio(
        &format!("alternating 5 words x50 ({} chars)", p2.len()),
        || { bp.reset(); for c in inp2.chars() { bp.process_key(c, Mode::Vietnamese); } },
        || { bb.reset(); bb.process_str(&inp2, Mode::Vietnamese); },
        || { s.transform(&inp2); },
    );
}

// ===========================================================================
// Category 18: Comprehensive Telex syllable coverage (100+ cases)
// ===========================================================================

fn bench_comprehensive_syllables() {
    let (mut bp, mut bb, s) = setup_engines!();

    let syllables: &[&str] = &[
        // Basic vowels with all 5 tones
        "a", "af", "as", "ar", "ax", "aj",
        "e", "ef", "es", "er", "ex", "ej",
        "i", "if", "is", "ir", "ix", "ij",
        "o", "of", "os", "or", "ox", "oj",
        "u", "uf", "us", "ur", "ux", "uj",
        "y", "yf", "ys", "yr", "yx", "yj",
        // Circumflex + tones
        "aa", "aas", "aaf", "aar", "aax", "aaj",
        "ee", "ees", "eef", "eer", "eex", "eej",
        "oo", "oos", "oof", "oor", "oox", "ooj",
        // Breve + tones
        "aw", "aws", "awf", "awr", "awx", "awj",
        // Horn + tones
        "ow", "ows", "owf", "owr", "owx", "owj",
        "uw", "uws", "uwf", "uwr", "uwx", "uwj",
        // D-stroke
        "dd", "DD",
        // Consonant + vowel combos (90+)
        "ba", "bo", "bu", "be", "bi", "by",
        "ca", "co", "cu", "ce", "ci",
        "da", "do", "du", "de", "di",
        "ga", "go", "gu", "ge", "gi",
        "ha", "ho", "hu", "he", "hi",
        "ka", "ko", "ku", "ke", "ki",
        "la", "lo", "lu", "le", "li", "ly",
        "ma", "mo", "mu", "me", "mi", "my",
        "na", "no", "nu", "ne", "ni", "ny",
        "pa", "po", "pu", "pe", "pi",
        "ra", "ro", "ru", "re", "ri",
        "sa", "so", "su", "se", "si",
        "ta", "to", "tu", "te", "ti",
        "va", "vo", "vu", "ve", "vi",
        "xa", "xo", "xu", "xe", "xi",
        // Complex combos
        "cha", "cho", "chu", "che", "chi",
        "nha", "nho", "nhu", "nhe", "nhi",
        "nga", "ngo", "ngu", "nge", "ngi",
        "pha", "pho", "phu", "phe", "phi",
        "qua", "que", "qui", "quy",
        "tha", "tho", "thu", "the", "thi",
        "tra", "tro", "tru", "tre", "tri",
        // Real words
        "tieengs", "vietj", "huowng", "quoocs", "nguwowif",
        "khongf", "duowcj", "nhuwngf", "moiw", "laaj",
        "tuoif", "troiwf", "lamf", "anhf", "buocj",
    ];

    println!("\n=== Category 18: Comprehensive syllables ({} cases) ===", syllables.len());

    let inputs: Vec<String> = syllables.iter().map(|s| s.to_string()).collect();

    // bamboo per-key
    for _ in 0..WARMUP_ITERS {
        for inp in &inputs { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } }
    }
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        for inp in &inputs { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } }
    }
    let bamboo_pk_ns = start.elapsed().as_nanos() as f64 / (BENCH_ITERS * inputs.len()) as f64;

    // bamboo batch
    for _ in 0..WARMUP_ITERS {
        for inp in &inputs { bb.reset(); bb.process_str(inp, Mode::Vietnamese); }
    }
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        for inp in &inputs { bb.reset(); bb.process_str(inp, Mode::Vietnamese); }
    }
    let bamboo_batch_ns = start.elapsed().as_nanos() as f64 / (BENCH_ITERS * inputs.len()) as f64;

    // skey batch
    for _ in 0..WARMUP_ITERS {
        for inp in &inputs { s.transform(inp); }
    }
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        for inp in &inputs { s.transform(inp); }
    }
    let skey_ns = start.elapsed().as_nanos() as f64 / (BENCH_ITERS * inputs.len()) as f64;

    println!(
        "{:<45} bamboo/key: {:>8.1} ns | bamboo/batch: {:>8.1} ns | skey: {:>8.1} ns | pk: {:.2}x | batch: {:.2}x",
        format!("{} syllables (avg)", inputs.len()),
        bamboo_pk_ns, bamboo_batch_ns, skey_ns,
        bamboo_pk_ns / skey_ns, bamboo_batch_ns / skey_ns
    );
}

// ===========================================================================
// Category 19: Diphthongs / Triphthongs
// ===========================================================================

fn bench_diphthongs() {
    let (mut bp, mut bb, s) = setup_engines!();

    let cases: &[(&str, &str)] = &[
        ("ai", "ai"), ("ao", "ao"), ("au", "au"), ("ay", "ay"),
        ("eo", "eo"), ("eu", "eu"), ("ia", "ia"), ("iu", "iu"),
        ("oa", "oa"), ("oe", "oe"), ("oi", "oi"),
        ("ua", "ua"), ("ue", "ue"), ("ui", "ui"),
        ("uo", "uo"), ("uu", "uu"), ("uy", "uy"),
        ("ieu", "iêu"), ("oai", "oai"), ("uay", "uay"),
        ("uoi", "uôi"), ("uyu", "uyu"),
        ("ngoif", "ngồi"), ("buoif", "buôi"),
        ("quoif", "quồi"), ("muoif", "muồi"),
    ];

    println!("\n=== Category 19: Diphthongs / Triphthongs ===");
    for &(input, expected) in cases {
        let inp = input.to_string();
        bench_trio(
            &format!("diphthong: {} → {}", input, expected),
            || { bp.reset(); for c in inp.chars() { bp.process_key(c, Mode::Vietnamese); } },
            || { bb.reset(); bb.process_str(&inp, Mode::Vietnamese); },
            || { s.transform(&inp); },
        );
    }
}

// ===========================================================================
// Main
// ===========================================================================

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════════╗");
    println!("║          Bamboo-core vs Skey-engine — Telex Benchmark Suite                  ║");
    println!("╠════════════════════════════════════════════════════════════════════════════════╣");
    println!("║  bamboo/key   = per-key stateful (simulates real typing)                     ║");
    println!("║  bamboo/batch = process_str (batch string processing)                        ║");
    println!("║  skey         = stateless batch transform                                    ║");
    println!("║  ratio > 1.0 = bamboo slower; < 1.0 = bamboo faster                         ║");
    println!("╠════════════════════════════════════════════════════════════════════════════════╣");
    println!("║  Warmup: {:>7} iters | Benchmark: {:>7} iters                                ║", WARMUP_ITERS, BENCH_ITERS);
    println!("╚════════════════════════════════════════════════════════════════════════════════╝");

    bench_single_tones();
    bench_single_marks();
    bench_common_words();
    bench_compound_marks();
    bench_tone_replacement();
    bench_double_vowel_toggle();
    bench_w_key();
    bench_capitalization();
    bench_digraphs();
    bench_sentences();
    bench_long_paragraph();
    bench_cold_start();
    bench_backspace_only();
    bench_delta_api();
    bench_scalability();
    bench_english_passthrough();
    bench_stress_patterns();
    bench_comprehensive_syllables();
    bench_diphthongs();

    println!("\n╔════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                              Benchmark Complete                              ║");
    println!("╚════════════════════════════════════════════════════════════════════════════════╝");
}
