# Rust Performance & Optimization Guide (Bamboo Core)

> **Mục tiêu**: Hướng dẫn toàn diện về tối ưu hóa hiệu năng cực đại trong Rust và tài liệu kiến trúc **Zero-Allocation Hot Path** của `bamboo_core`.

---

## 1. Kiến Trúc Hiệu Năng Cao của `bamboo_core`

Trong một bộ gõ tiếng Việt (IME), mỗi lần người dùng nhấn một phím, engine phải xử lý và phản hồi trong thời gian thực:
- **Ngân sách độ trễ (Latency Budget)**: `< 1 µs / keystroke` (nhanh hơn tốc độ mắt người nhận biết ~16ms hàng ngàn lần).
- **Ngân sách bộ nhớ (Allocation Budget)**: **0 Heap Allocations** trên luồng gõ phím (`hot path`).

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      HOT PATH: 0 HEAP ALLOCATIONS                       │
│                                                                         │
│   Keystroke 's' ──► [ DFA Fast-Path Cache (O(1)) ] ────────────────┐    │
│                            │ (DFA Miss)                            │    │
│                            ▼                                       ▼    │
│                     [ Stack Scratch ] ──► [ Bitmask CVC ] ──► Output    │
│                     [Transformation; 16]   u32 / u64 Bitops   [char; 16]│
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Các Mẫu Thiết Kế Tối Ưu (Patterns & Techniques)

### 2.1. Zero-Allocation Stack Structures
Không sử dụng `Vec<T>` hoặc `String` trong luồng xử lý phím bấm liên tục:

```rust
// ❌ BAD: Cấp phát Heap trên mỗi phím bấm
fn process_key(&mut self, ch: char) -> String {
    let mut transforms: Vec<Transformation> = Vec::new();
    // ...
    transforms.iter().collect()
}

//  GOOD: Fixed-capacity Stack Storage (Inlined Array)
pub const MAX_ACTIVE_TRANS: usize = 16;

#[derive(Clone, Copy, Default)]
pub struct TransformationStack {
    data: [Transformation; MAX_ACTIVE_TRANS],
    len: usize,
}
```

### 2.2. DFA State Machine với Flat Arena Storage
Lưu trữ trạng thái DFA trong mảng phẳng (`Vec<State>` được khởi tạo sẵn hoặc mảng tĩnh), tra cứu chuyển trạng thái theo mã ASCII trong $O(1)$:

```rust
pub struct State {
    // 256 transitions ánh xạ trực tiếp từ ASCII byte
    pub transitions: [u32; 256],
    pub composition_offset: u32,
    pub composition_len: u8,
}

#[inline(always)]
pub fn get_transition(&self, key: u8) -> u32 {
    self.transitions[key as usize]
}
```

### 2.3. Dynamic Case Preservation (Bảo Toàn Hoa/Thường Không Làm Ô Nhiễm DFA)
Thay vì lưu riêng trạng thái chữ hoa trong DFA (làm bùng nổ không gian trạng thái $2^N$), DFA chỉ lưu trạng thái chữ thường chuẩn (`canonical lowercase`). Trạng thái chữ hoa được ánh xạ động trên stack bằng mảng byte `prev_upper`:

```rust
let mut prev_upper = [false; MAX_ACTIVE_TRANS];
for (dst, src) in prev_upper.iter_mut().zip(&self.active_buffer[..prev_len]) {
    *dst = src.is_upper_case;
}

// Tra cứu DFA (lowercase)
self.current_state_id = next_state_id;
self.active_buffer[..self.active_len].copy_from_slice(comp);

// Ánh xạ ngược lại thuộc tính hoa/thường chỉ trong vài lệnh CPU
for (dst, &is_up) in self.active_buffer[..prev_len.min(self.active_len)]
    .iter_mut()
    .zip(&prev_upper)
{
    dst.is_upper_case = is_up;
}
```

### 2.4. Ma Trận Ngữ Âm Bitmask $O(1)$ (`spelling.rs`)
Toàn bộ quy tắc hợp lệ ngữ âm tiếng Việt (CVC - Initial Consonant, Vowel, Final Consonant) được mã hóa thành các hằng số bitmask `u32` / `u64`:

```rust
// Tra cứu tính hợp lệ phụ âm đầu + vần chỉ bằng phép AND bitwise:
#[inline(always)]
pub fn is_valid_cvc_fast(onset_idx: usize, rhyme_idx: usize) -> bool {
    (VALID_ONSET_MATRIX[onset_idx] & (1u64 << rhyme_idx)) != 0
}
```

---

## 3. Tối Ưu Hóa Bộ Nhớ & CPU Cache (Memory Layout)

### 3.1. Struct Alignment & Padding
Sắp xếp các trường trong `struct` từ lớn nhất đến nhỏ nhất để tránh padding lãng phí bộ nhớ:

```rust
// ❌ BAD: 24 bytes do struct padding
struct BadTransformation {
    target: Option<u8>, // 2 bytes + 6 pad
    rule: Rule,         // 8 bytes
    is_upper: bool,     // 1 byte + 7 pad
}

//  GOOD: 16 bytes — Field ordering & Compact Types
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Transformation {
    pub rule: Rule,             // 8 bytes
    pub target: Option<u8>,     // 2 bytes
    pub is_upper_case: bool,    // 1 byte
    pub _pad: [u8; 5],          // Explicit alignment to 16 bytes
}

// Khẳng định kích thước struct tại compile-time:
const _: () = assert!(std::mem::size_of::<Transformation>() == 16);
```

### 3.2. Không Sử Dụng Bounds-Checking Không Cần Thiết
Sử dụng Iterator và `zip()` thay vì `for i in 0..len` với chỉ mục mảng `arr[i]`:

```rust
// ❌ BAD: Trình biên dịch phải sinh mã kiểm tra biên (bounds checking) ở mỗi vòng lặp
for i in 0..len {
    dest[i] = src[i];
}

//  GOOD: Không bounds checking, SIMD vectorizable
for (dst, src) in dest[..len].iter_mut().zip(&src[..len]) {
    *dst = *src;
}
```

---

## 4. Cấu Hình Release Profile Tối Ưu (`Cargo.toml`)

Để đạt tốc độ thực thi tối đa, cấu hình `[profile.release]` và `[profile.bench]`:

```toml
[profile.release]
opt-level = 3
lto = "fat"            # Link-Time Optimization liên crate
codegen-units = 1      # Tối đa hóa inline và tối ưu toàn cục
panic = "abort"        # Loại bỏ unwinding landing pads
strip = "symbols"      # Giảm dung lượng nhị phân
overflow-checks = false

[profile.bench]
opt-level = 3
lto = "fat"
codegen-units = 1
debug = true           # Giữ debuginfo cho profiling/flamegraph
```

---

## 5. Thước Đo Benchmark Đối Soát (2026 Metrics)

Chạy thực nghiệm benchmark đối sánh trên `benches/vi_bench.rs` (100.000 iterations):

| Kịch Bản Gõ Phím | Bamboo Core (`v0.3.19`) | Skey-Engine (`v0.1.4`) | Uvie (`v2.1.1`) | Vi (`v0.8.0`) | Tốc Độ Vượt Trội |
|---|---|---|---|---|---|
| Gõ từ đơn `tieengs` $\to$ `tiếng` | **87.8 ns** | 292.2 ns | 789.0 ns | 2580.0 ns | **Nhanh gấp 3.3x Skey, 9.0x Uvie, 29x Vi** |
| Gõ từ ngắn `vietj` $\to$ `việt` | **58.7 ns** | 175.5 ns | 682.1 ns | 1547.3 ns | **Nhanh gấp 3.0x Skey, 11.6x Uvie, 26x Vi** |
| Gõ từ ghép `nguwowif` $\to$ `người` | **106.6 ns** | 329.7 ns | 883.5 ns | 3265.3 ns | **Nhanh gấp 3.1x Skey, 8.3x Uvie, 30x Vi** |
| Xóa lùi tương tác `tieengs` + 2 Backspaces | **212.8 ns** | 380.7 ns | 1100.5 ns | 1588.5 ns | **Nhanh gấp 1.8x Skey, 5.2x Uvie, 7.5x Vi** |
| Chuỗi câu đầy đủ (27 ký tự) | **778.8 ns** | 2848.9 ns | 2076.9 ns | 8904.2 ns | **Nhanh gấp 3.7x Skey, 2.7x Uvie, 11.4x Vi** |
| Đoạn văn liên tục (62 ký tự) | **1766.7 ns** | 13115.1 ns | 5256.0 ns | 22077.0 ns | **Nhanh gấp 7.4x Skey, 3.0x Uvie, 12.5x Vi** |
| Mã nguồn / Tiếng Anh (31 ký tự) | **1176.7 ns** | 3756.6 ns | 3613.6 ns | 9467.7 ns | **Nhanh gấp 3.2x Skey, 3.1x Uvie, 8.0x Vi** |

---

## 6. Quy Trình Kiểm Thử & Đo Lường Hiệu Năng (Checklist)

1. **Kiểm tra linter không lỗi cảnh báo**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```
2. **Kiểm tra định dạng code chuẩn**:
   ```bash
   cargo fmt --check
   ```
3. **Chạy toàn bộ test suites**:
   ```bash
   cargo test --all-targets
   ```
4. **Đo hiệu năng và phát hiện regression**:
   ```bash
   cargo bench --bench engine_bench
   cargo bench --bench vi_bench
   ```
5. **Profiling với Flamegraph khi cần điều tra điểm nghẽn**:
   ```bash
   cargo flamegraph --bench engine_bench
   ```
