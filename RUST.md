# RUST.md

> "Don't fight the compiler. Make invalid states unrepresentable."

---

# Part 1 - Rust Philosophy & Core Principles

Đây không phải coding style thông thường.

Đây là những nguyên tắc bắt buộc khi viết Rust.

Mục tiêu là viết code theo triết lý của Rust thay vì mang tư duy C/C++ sang.

---

# Core Philosophy

Rust ưu tiên

- Correctness
- Type Safety
- Memory Safety
- Zero-cost Abstractions
- Explicitness
- Composition
- Predictability

Compiler là đồng đội.

Đừng cố đánh lừa borrow checker.

Nếu compiler báo lỗi, hãy tìm nguyên nhân thật sự thay vì sửa cho compiler "im miệng".

---

# Golden Rules

Luôn suy nghĩ theo thứ tự sau.

```
Correctness
    ↓
Safety
    ↓
Readability
    ↓
Maintainability
    ↓
Performance
```

Đừng tối ưu trước khi code đúng.

---

# Rust is NOT C++

Không mang tư duy C/C++ vào Rust.

Sai:

- viết mọi thứ bằng struct + impl
- dùng getter/setter khắp nơi
- mutable mặc định
- clone khắp project
- index mọi collection
- vòng for kiểu C
- enum chỉ để chứa integer
- bool flag thay state machine

Rust có cách làm riêng.

Hãy dùng cách của Rust.

---

# The Type System is Your First Line of Defense

Nếu compiler có thể kiểm tra thì đừng để runtime kiểm tra.

Sai

```rust
pub const STATUS_CONNECTING: u8 = 0;
pub const STATUS_CONNECTED: u8 = 1;
pub const STATUS_CLOSED: u8 = 2;
```

Đúng

```rust
enum ConnectionState {
    Connecting,
    Connected,
    Closed,
}
```

---

Sai

```rust
fn login(force: bool)
```

Đúng

```rust
enum LoginMode {
    Normal,
    Force,
}

fn login(mode: LoginMode)
```

Compiler sẽ ngăn truyền giá trị sai.

---

# Make Invalid States Impossible

Không dùng primitive để biểu diễn domain.

Sai

```rust
struct User {
    id: String,
}
```

Đúng

```rust
struct UserId(Uuid);

struct User {
    id: UserId,
}
```

Sai

```rust
status: u8
```

Đúng

```rust
status: TransferState
```

---

# Newtype Pattern

Không truyền

```
String
usize
u64
bool
```

khắp project.

Tạo kiểu riêng.

Ví dụ

```rust
struct UserId(Uuid);

struct DeviceId(Uuid);

struct TransferId(Uuid);

struct MessageId(Uuid);
```

Thậm chí

```rust
struct Username(String);

struct Password(String);
```

cũng tốt hơn

```rust
(String, String)
```

---

# Primitive Obsession

Primitive chỉ nên tồn tại ở rìa hệ thống.

Business logic nên làm việc với type.

Sai

```rust
fn send(
    from: String,
    to: String,
    size: usize,
)
```

Đúng

```rust
fn send(
    from: UserId,
    to: UserId,
    size: Bytes,
)
```

---

# Prefer Enums

Đừng dùng

```
bool

u8

u32

String
```

để biểu diễn trạng thái.

Sai

```rust
struct Task {
    done: bool,
}
```

Đúng

```rust
enum TaskState {
    Pending,
    Running,
    Completed,
    Failed,
}
```

---

# Prefer Exhaustive Match

Sai

```rust
if status == 1 {
}
else if status == 2 {
}
```

Đúng

```rust
match status {
    ConnectionState::Connecting => {}
    ConnectionState::Connected => {}
    ConnectionState::Closed => {}
}
```

Compiler sẽ nhắc nếu enum thêm variant mới.

---

# Reference First

Đây là nguyên tắc QUAN TRỌNG NHẤT.

Luôn nghĩ

```
&
```

trước

```
clone()
```

Ownership chỉ nên chuyển khi thực sự cần.

---

## Borrow > Move > Clone

Luôn ưu tiên

```
&T

↓

&mut T

↓

T

↓

Clone
```

Clone là lựa chọn cuối cùng.

---

Sai

```rust
foo(name.clone());
```

Đúng

```rust
foo(&name);
```

---

Sai

```rust
let username = user.name.clone();
```

Đúng

```rust
let username = &user.name;
```

---

Sai

```rust
let path = pathbuf.clone();

read(path);
```

Đúng

```rust
read(&pathbuf);
```

---

# Clone Policy

Mỗi lần viết

```rust
.clone()
```

hãy tự hỏi:

- Clone để làm gì?
- Có borrow được không?
- Có move ownership được không?
- Có dùng Cow được không?
- Clone này có allocation không?
- Clone này O(1) hay O(n)?

Nếu không trả lời được

→ Không clone.

---

# Clone is NOT a Fix

Không bao giờ thêm clone chỉ vì compiler báo lỗi.

Sai

```rust
foo(data.clone());
bar(data.clone());
baz(data.clone());
```

Đúng

Tìm lại ownership.

---

Nếu AI thêm clone để sửa borrow checker

=> AI đang viết code sai.

---

# Prefer References

Input của API nên nhận

```
&str

&Path

&[u8]

&[T]

&T
```

thay vì

```
String

Vec<u8>

PathBuf

Vec<T>
```

---

Sai

```rust
fn parse(json: String)
```

Đúng

```rust
fn parse(json: &str)
```

---

Sai

```rust
fn hash(data: Vec<u8>)
```

Đúng

```rust
fn hash(data: &[u8])
```

---

Sai

```rust
fn open(path: PathBuf)
```

Đúng

```rust
fn open(path: &Path)
```

---

# Accept Wide, Return Narrow

Input nên linh hoạt.

Output nên cụ thể.

Ví dụ

```rust
fn load(path: impl AsRef<Path>)
```

thay vì

```rust
fn load(path: PathBuf)
```

---

Input nên dùng

```
AsRef

IntoIterator

Borrow

Into
```

khi hợp lý.

---

# Avoid Unnecessary Allocation

Sai

```rust
String::from("hello")
```

Đúng

```rust
"hello"
```

---

Sai

```rust
format!("{}", value)
```

nếu chỉ cần Display.

---

Sai

```rust
let bytes = text.as_bytes().to_vec();
```

Đúng

```rust
let bytes = text.as_bytes();
```

---

# Zero-Copy Mindset

Luôn tự hỏi

"Có thể tránh copy không?"

Ưu tiên

```
&str

&[u8]

Cow<'_, str>

Bytes

Arc<[u8]>
```

hơn

```
String

Vec<u8>
```

nếu ownership không cần.

---

# Immutability First

Mặc định

```
let
```

Thay vì

```
let mut
```

Mutable chỉ khi thật sự cần.

---

# Keep Functions Pure

Ưu tiên function

- không mutate global state
- không side effect
- dễ test
- deterministic

---

# Prefer Composition

Thay vì

```text
Manager
Controller
Helper
Util
```

hãy chia thành các thành phần nhỏ có trách nhiệm rõ ràng.

Rust mạnh ở composition.

Không phải inheritance.

---

# Rust Mindset Checklist

Trước khi viết code hãy tự hỏi

□ Có cần ownership không?

□ Có borrow được không?

□ Có cần clone không?

□ Có enum thay bool không?

□ Có newtype thay String không?

□ Có thể để compiler kiểm tra không?

□ Có thể bỏ allocation không?

□ Có thể tránh copy không?

□ Có thể dùng std thay vì tự viết không?

□ Có thể biểu diễn bằng type không?

Nếu câu trả lời là "Có"

=> Hãy sửa thiết kế trước khi viết code.

---

# End of Part 1

Nếu compiler có thể chứng minh code đúng

→ Hãy để compiler làm.

Nếu type system làm được

→ Đừng để runtime làm.

Nếu borrow được

→ Đừng clone.

Nếu enum làm được

→ Đừng dùng integer.

Nếu newtype làm được

→ Đừng truyền String.

Viết Rust theo cách của Rust.

---

# Part 2 - Idiomatic Rust & Standard Library First

Rust không chỉ là ownership.

Điểm mạnh lớn nhất của Rust là Standard Library và hệ sinh thái iterator.

Nếu bạn đang viết nhiều vòng `for`, nhiều `if`, nhiều biến mutable, rất có thể bạn đang viết C bằng cú pháp Rust.

---

# Standard Library First

Trước khi tự viết một thuật toán, hãy tự hỏi:

- std đã có chưa?
- Iterator đã có chưa?
- Slice đã có chưa?
- Option có method không?
- Result có method không?

Không viết lại những gì std đã làm tốt.

---

# Iterator First

Đừng nghĩ

```text
for
```

Hãy nghĩ

```text
Iterator
```

Rust được thiết kế xoay quanh Iterator.

---

## Prefer Iterator over Index

Sai

```rust
for i in 0..values.len() {
    println!("{}", values[i]);
}
```

Đúng

```rust
for value in &values {
    println!("{value}");
}
```

---

Sai

```rust
for i in 0..users.len() {
    process(&users[i]);
}
```

Đúng

```rust
for user in &users {
    process(user);
}
```

---

Nếu cần index

Dùng

```rust
for (index, value) in values.iter().enumerate() {

}
```

---

# Avoid Manual Loops

Sai

```rust
let mut sum = 0;

for x in values {
    sum += x;
}
```

Đúng

```rust
let sum: i32 = values.iter().sum();
```

---

Sai

```rust
let mut count = 0;

for user in users {
    if user.online {
        count += 1;
    }
}
```

Đúng

```rust
let count = users
    .iter()
    .filter(|u| u.online)
    .count();
```

---

# Prefer map()

Sai

```rust
let mut result = Vec::new();

for user in users {
    result.push(user.id);
}
```

Đúng

```rust
let result = users
    .iter()
    .map(|u| u.id)
    .collect::<Vec<_>>();
```

---

# Prefer filter()

Sai

```rust
let mut active = Vec::new();

for user in users {
    if user.active {
        active.push(user);
    }
}
```

Đúng

```rust
let active = users
    .iter()
    .filter(|u| u.active)
    .collect::<Vec<_>>();
```

---

# Prefer filter_map()

Sai

```rust
let mut ids = Vec::new();

for value in values {
    if let Some(id) = parse(value) {
        ids.push(id);
    }
}
```

Đúng

```rust
let ids = values
    .iter()
    .filter_map(parse)
    .collect::<Vec<_>>();
```

---

# Prefer find()

Sai

```rust
let mut result = None;

for user in users {
    if user.id == id {
        result = Some(user);
        break;
    }
}
```

Đúng

```rust
let result = users
    .iter()
    .find(|u| u.id == id);
```

---

# Prefer any()

Sai

```rust
let mut found = false;

for user in users {
    if user.admin {
        found = true;
        break;
    }
}
```

Đúng

```rust
let found = users
    .iter()
    .any(|u| u.admin);
```

---

# Prefer all()

Sai

```rust
let mut ok = true;

for x in values {
    if !valid(x) {
        ok = false;
        break;
    }
}
```

Đúng

```rust
let ok = values
    .iter()
    .all(valid);
```

---

# Prefer fold()

Nếu đang cộng dồn giá trị

Hãy nghĩ tới

```rust
.fold()
```

Ví dụ

```rust
let total = values
    .iter()
    .fold(0, |acc, value| acc + value);
```

---

# Prefer reduce()

Nếu accumulator chính là phần tử đầu tiên.

```rust
values.iter().copied().reduce(i32::max)
```

---

# Prefer try_fold()

Nếu có Result.

Đừng viết

```rust
for ...

match ...

return Err(...)
```

Hãy dùng

```rust
.try_fold(...)
```

---

# Prefer collect()

Đừng push từng phần tử.

Hãy collect.

---

# Prefer extend()

Nếu chỉ nối collection.

Không

```rust
for item in other {
    vec.push(item);
}
```

Đúng

```rust
vec.extend(other);
```

---

# Prefer flatten()

Sai

```rust
Vec<Vec<T>>
```

rồi tự loop.

Đúng

```rust
flatten()
```

---

# Prefer flat_map()

Nếu map trả Iterator.

---

# Prefer zip()

Sai

```rust
for i in 0..a.len() {

}
```

Đúng

```rust
for (a, b) in left.iter().zip(right.iter()) {

}
```

---

# Prefer chunks()

Đừng tự chia block.

```rust
slice.chunks(4096)
```

---

# Prefer windows()

Đừng index

```rust
i

i+1
```

Dùng

```rust
slice.windows(2)
```

---

# Prefer split()

Đừng parser bằng index.

---

# Prefer partition()

Nếu cần chia thành 2 nhóm.

```rust
let (ok, failed) =
items.into_iter().partition(Result::is_ok);
```

---

# Prefer inspect()

Debug pipeline.

Không

```rust
println!()
```

giữa pipeline.

---

# Iterator Chain

Một iterator chain dài vẫn tốt hơn nhiều mutable variable.

Ví dụ

```rust
users
    .iter()
    .filter(...)
    .map(...)
    .filter(...)
    .collect()
```

đọc dễ hơn

```rust
let mut ...

for ...

if ...

push...
```

---

# Prefer Pattern Matching

Không viết

```rust
if ...

else if ...

else if ...
```

liên tục.

Rust sinh ra để

```rust
match
```

---

# if let

Sai

```rust
match option {
    Some(v) => process(v),
    None => {}
}
```

Đúng

```rust
if let Some(v) = option {
    process(v);
}
```

---

# let else

Sai

```rust
let value = match option {
    Some(v) => v,
    None => return,
};
```

Đúng

```rust
let Some(value) = option else {
    return;
};
```

---

# matches!

Sai

```rust
if let State::Running = state {
    ...
}
```

Nếu chỉ cần bool.

Đúng

```rust
if matches!(state, State::Running) {

}
```

---

# Option API First

Đừng unwrap.

Hãy nhớ Option có rất nhiều API.

Ưu tiên

- map
- and_then
- or
- or_else
- unwrap_or
- unwrap_or_else
- is_some_and
- filter
- zip
- transpose
- flatten

---

# Result API First

Đừng match mọi Result.

Hãy dùng

- map
- map_err
- inspect
- inspect_err
- and_then
- or_else
- ok
- err

---

# Prefer ? Operator

Sai

```rust
match read(path) {
    Ok(v) => v,
    Err(e) => return Err(e),
}
```

Đúng

```rust
let value = read(path)?;
```

---

# Avoid Boolean Flags

Sai

```rust
fn create(force: bool)
```

Đúng

```rust
enum CreateMode {
    Normal,
    Force,
}
```

---

# Avoid Magic Numbers

Sai

```rust
timeout(30)
```

Đúng

```rust
const DEFAULT_TIMEOUT: Duration =
    Duration::from_secs(30);
```

---

# Avoid Magic Strings

Sai

```rust
if role == "admin"
```

Đúng

```rust
enum Role {
    Admin,
    User,
}
```

---

# Prefer std APIs

Trước khi viết code hãy kiểm tra xem std đã có chưa.

Một số API rất hay bị bỏ quên:

- take()
- replace()
- mem::swap()
- mem::take()
- Option::take()
- std::iter::once()
- repeat_with()
- from_fn()
- std::array::from_fn()
- std::cmp::{min,max,clamp}
- std::mem
- std::borrow::Cow

---

# Don't Write C in Rust

Nếu code của bạn có nhiều:

- mutable variable
- index
- push
- break
- continue
- flag
- counter

Hãy dừng lại.

Có khả năng Iterator sẽ giúp code ngắn hơn, an toàn hơn và idiomatic hơn.

---

# Part 2 Summary

Khi bắt đầu viết một đoạn code, hãy tự hỏi theo thứ tự:

□ Có method của `Option` không?

□ Có method của `Result` không?

□ Có method của `Iterator` không?

□ Có API của `slice` không?

□ Có API trong `std` không?

□ Có thể dùng `map()` không?

□ Có thể dùng `filter()` không?

□ Có thể dùng `find()` không?

□ Có thể dùng `collect()` không?

□ Có thể dùng `?` không?

Nếu câu trả lời là **Có**, hãy dùng API của Rust thay vì viết theo phong cách C/C++.

---

# Part 3 - API Design, Traits, Lifetimes & Error Handling

Đây là phần quyết định chất lượng của một thư viện Rust.

Code có thể chạy đúng nhưng API tệ sẽ khiến người dùng phải clone nhiều hơn, ownership phức tạp hơn và rất khó mở rộng.

Thiết kế API tốt là khiến người dùng **khó dùng sai**.

---

# Design APIs Around Borrowing

API nên ưu tiên nhận reference.

Sai

```rust
fn parse(text: String)
```

Đúng

```rust
fn parse(text: &str)
```

---

Sai

```rust
fn hash(data: Vec<u8>)
```

Đúng

```rust
fn hash(data: &[u8])
```

---

Sai

```rust
fn open(path: PathBuf)
```

Đúng

```rust
fn open(path: &Path)
```

---

# Accept Broad, Return Concrete

Input càng linh hoạt càng tốt.

Output càng rõ ràng càng tốt.

Ví dụ

```rust
fn load(path: impl AsRef<Path>)
```

thay vì

```rust
fn load(path: PathBuf)
```

---

Có thể truyền

- Path
- PathBuf
- &Path
- String
- &String
- &str

mà không cần clone.

---

# AsRef

Nếu chỉ đọc dữ liệu

→ dùng

```rust
AsRef<T>
```

Ví dụ

```rust
fn open<P: AsRef<Path>>(path: P)
```

---

# Into

Nếu cần ownership

→ dùng

```rust
Into<T>
```

Ví dụ

```rust
fn new(name: impl Into<String>)
```

thay vì

```rust
fn new(name: String)
```

---

# IntoIterator

Nếu API nhận collection

Đừng viết

```rust
Vec<T>
```

Hãy viết

```rust
impl IntoIterator<Item = T>
```

---

# Avoid Concrete Types

Sai

```rust
fn process(users: Vec<User>)
```

Đúng

```rust
fn process(
    users: impl IntoIterator<Item = User>
)
```

---

# Return Slices

Sai

```rust
fn users(&self) -> &Vec<User>
```

Đúng

```rust
fn users(&self) -> &[User]
```

Caller không cần biết bên trong dùng Vec.

---

# Avoid Returning HashMap

Đừng expose implementation.

Sai

```rust
fn cache(&self) -> &HashMap<K, V>
```

Nếu có thể

Trả

```rust
Iterator

Slice

View

```

---

# Constructors

Không tạo

```
new()

new2()

new3()

new_with_flag()

new_from_other()
```

Hãy dùng Builder.

---

# Builder Pattern

Sai

```rust
Config::new(
    host,
    port,
    timeout,
    retry,
    ssl,
    compression,
)
```

Đúng

```rust
Config::builder()
    .host(...)
    .timeout(...)
    .ssl(...)
    .build()
```

---

# Prefer Default

Nếu struct có giá trị mặc định.

Implement

```rust
Default
```

---

# Implement Standard Traits

Nếu phù hợp

Ưu tiên implement

```
Debug

Display

Default

Clone

Copy

Hash

Eq

PartialEq

Ord

PartialOrd

From

Into

TryFrom

AsRef

Borrow
```

Không tự tạo

```
to_string_custom()

convert()

```

---

# From > Constructor

Sai

```rust
UserId::from_uuid(uuid)
```

Đúng

```rust
impl From<Uuid> for UserId
```

---

# TryFrom

Nếu conversion có thể fail.

Dùng

```rust
TryFrom
```

---

# Extension Traits

Nếu muốn thêm method

Đừng viết

```rust
utils::parse_user(...)
```

Hãy viết

```rust
trait UserExt {
    fn parse(...);
}
```

---

# Traits Describe Capability

Trait không phải interface Java.

Trait mô tả khả năng.

Ví dụ

```
Read

Write

Serialize

Display
```

Không phải

```
UserManager

FileManager

DatabaseManager
```

---

# Don't Create Traits Too Early

Nếu chỉ có một implementation

→ chưa cần trait.

Trait chỉ khi có abstraction thực sự.

---

# Generic vs dyn Trait

Ưu tiên Generic.

```rust
fn process<T: Read>(...)
```

Chỉ dùng

```rust
dyn Trait
```

khi runtime polymorphism là yêu cầu.

---

# Object Safety

Nếu trait không cần dynamic dispatch

Đừng ép object safe.

---

# Sealed Trait

Nếu không muốn crate khác implement.

Dùng sealed trait pattern.

Không expose trait nếu không cần.

---

# Avoid Over-Generic APIs

Sai

```rust
T

U

V

K

P

Q
```

khắp project.

Generic phải giải quyết vấn đề thật.

Không phải "phòng hờ".

---

# Lifetime Philosophy

AI rất thích thêm

```rust
<'a>
```

mọi nơi.

Đó thường là dấu hiệu thiết kế chưa tốt.

---

# Lifetime Elision

Đừng annotate nếu compiler tự suy luận được.

Sai

```rust
fn name<'a>(
    value: &'a str,
) -> &'a str
```

Đúng

```rust
fn name(
    value: &str,
) -> &str
```

---

# Don't Fight Lifetimes

Nếu lifetime quá phức tạp

Đừng cố thêm

```
'a

'b

'c

'static
```

Hãy xem lại ownership.

---

# Prefer Ownership Change

Thay vì

```
5 lifetime

8 generic

```

Có thể

```
Arc

Cow

Bytes

```

đơn giản hơn.

---

# Never Add 'static to Silence Compiler

Sai

```rust
&'static str
```

chỉ để compiler hết lỗi.

'static là contract.

Không phải thuốc chữa.

---

# Lifetime Checklist

Nếu compiler báo lỗi

Đừng thêm lifetime trước.

Hãy tự hỏi

- borrow sai?
- ownership sai?
- clone được không?
- move được không?
- Arc hợp lý không?

---

# Error Philosophy

Library

→ thiserror

Application

→ anyhow

Không ngược lại.

---

# Never Panic in Library

Không

```rust
unwrap()

expect()

panic!()
```

trong library.

Ngoại lệ

- test
- unreachable invariant

---

# Prefer Result

Sai

```rust
Option<T>
```

nếu cần biết nguyên nhân.

Đúng

```rust
Result<T, Error>
```

---

# Rich Errors

Sai

```rust
Err(String)
```

Đúng

```rust
enum Error {

}
```

---

# thiserror

Ưu tiên

```rust
#[derive(thiserror::Error)]
```

Không tự implement Display.

---

# Error Variants

Error phải mô tả domain.

Sai

```
Unknown

Generic

Internal
```

Đúng

```
InvalidToken

PeerOffline

Timeout

PermissionDenied

```

---

# Error Context

Application

Có thể dùng

```rust
context()

with_context()
```

---

# Don't Lose Errors

Sai

```rust
.map_err(|_| Error)
```

Nếu làm mất nguyên nhân.

---

# ? First

Đừng

```rust
match
```

nếu chỉ propagate.

Dùng

```rust
?
```

---

# Result API

Ưu tiên

```
map()

map_err()

inspect()

inspect_err()

and_then()

or_else()
```

---

# Logging

Library

Không

```
println!
```

Không log lung tung.

Để application quyết định.

---

# Documentation

Public API

Phải có

Rustdoc.

Mọi public function nên giải thích

- làm gì
- input
- output
- lỗi nào

---

# Naming

Tên API phải là động từ.

Ví dụ

```
load()

save()

connect()

send()

receive()

encode()

decode()

```

Không

```
do()

run()

process()

execute()

```

nếu quá chung chung.

---

# Avoid Utils

Không tạo

```
utils.rs

helpers.rs

common.rs

misc.rs
```

Nếu một module không có domain rõ ràng

→ Thiết kế đang có vấn đề.

---

# Public API Checklist

Trước khi public một API

□ Có nhận reference thay ownership không?

□ Có dùng AsRef nếu phù hợp không?

□ Có cần Into không?

□ Có đang expose Vec thay Slice không?

□ Có expose HashMap không?

□ Có implement trait chuẩn chưa?

□ Có thể dùng Builder không?

□ Có rustdoc chưa?

□ Có Error cụ thể chưa?

□ Có panic không?

□ Có clone không cần thiết không?

---

# Library Design Rules

Một thư viện Rust tốt nên khiến người dùng

- clone ít nhất
- không cần nghĩ về lifetime
- khó dùng sai
- khó tạo invalid state
- compiler hướng dẫn cách dùng đúng

Nếu người dùng phải thêm

```rust
.clone()
```

hay

```rust
Arc<Mutex<_>>
```

chỉ để dùng API của bạn

→ API đó cần được thiết kế lại.

---

# End of Part 3

API tốt không phải API nhiều tính năng.

API tốt là API:

- Khó dùng sai.
- Dễ đọc.
- Ít clone.
- Ít lifetime.
- Ít generic dư thừa.
- Dựa vào type system thay vì comment.
- Để compiler bảo vệ người dùng.

---

# Part 4 - Async Rust & Concurrency

Async Rust không phải là "thêm async vào mọi function".

Async là công cụ để quản lý IO concurrency.

Không phải để tăng tốc CPU.

Nếu không có IO

→ Đừng dùng async.

---

# Async Philosophy

Mỗi async task phải có

- owner
- lifecycle
- shutdown
- cancellation

Không được có task "mồ côi".

---

# Async is Viral

Một khi function là async

Caller cũng phải async.

Đừng async hóa toàn bộ project.

---

# Ask Before Using async

Trước khi thêm

```rust
async fn
```

hãy tự hỏi

□ Có IO không?

□ Có cần concurrency không?

□ Blocking có đủ không?

Nếu không

→ Viết synchronous.

---

# Never Spawn Everything

Sai

```rust
tokio::spawn(...);

tokio::spawn(...);

tokio::spawn(...);

tokio::spawn(...);
```

mọi nơi.

Spawn không miễn phí.

---

# Every Spawn Needs an Owner

Mỗi

```rust
tokio::spawn()
```

phải có

- JoinHandle
- shutdown
- cancellation

Không spawn rồi bỏ.

---

# Detached Tasks

Sai

```rust
tokio::spawn(async {

});
```

không giữ JoinHandle.

Task sẽ chạy mãi.

---

# Structured Concurrency

Ưu tiên

```
JoinSet

JoinHandle

scope()

CancellationToken
```

thay vì

```
spawn khắp project
```

---

# Cancellation

Task dài phải hủy được.

Ví dụ

```
CancellationToken
```

hoặc

```
select!
```

---

Không nên

```rust
loop {

}
```

không có đường thoát.

---

# select!

Ưu tiên

```rust
tokio::select!
```

để

- timeout
- shutdown
- cancellation
- channel

---

# Timeouts

Network

Disk

RPC

Database

đều nên có timeout.

Không await vô hạn.

---

# Don't Hold Mutex Across Await

Đây là luật quan trọng nhất.

Sai

```rust
let mut state = mutex.lock().await;

socket.read().await;

state.counter += 1;
```

Mutex bị giữ suốt thời gian network.

---

Đúng

```rust
{
    let mut state = mutex.lock().await;

    state.counter += 1;
}

socket.read().await;
```

Release lock càng sớm càng tốt.

---

# Lock Small

Mutex chỉ nên bảo vệ

- vài field

- vài assignment

Không bảo vệ business logic.

---

Sai

```rust
lock

↓

parse

↓

encrypt

↓

network

↓

database

↓

unlock
```

---

Đúng

```rust
lock

↓

copy data

↓

unlock

↓

xử lý
```

---

# Never Sleep While Holding Mutex

Sai

```rust
let lock = mutex.lock().await;

tokio::time::sleep(...).await;
```

---

# Never Await While Holding Mutex

Không

```
network

database

sleep

channel.recv

file IO

```

khi đang giữ mutex.

---

# Prefer Message Passing

Nếu nhiều task cần giao tiếp

Ưu tiên

```
mpsc

broadcast

watch

oneshot
```

hơn

```
Arc<Mutex<_>>
```

---

# Shared State is the Last Choice

Thứ tự ưu tiên

```
Ownership

↓

Channel

↓

Arc

↓

Arc<Mutex>

↓

Arc<RwLock>
```

---

# Actor Model

Nếu state phức tạp

Một owner.

Nhiều message.

Thay vì

```
10 task

↓

1 mutex
```

---

# Choose the Right Channel

mpsc

→ queue

---

oneshot

→ request/response

---

watch

→ latest state

---

broadcast

→ event

---

Đừng dùng

```
mpsc
```

cho mọi thứ.

---

# Prefer Bounded Channels

Sai

```rust
unbounded_channel()
```

mọi nơi.

Có thể ăn hết RAM.

---

Đúng

```rust
mpsc::channel(1024)
```

---

# Backpressure

Producer không được nhanh vô hạn.

Nếu consumer chậm

Producer phải

- block

- retry

- drop

- backoff

---

# Never Ignore Backpressure

Nếu queue luôn tăng

Đó là bug.

Không phải feature.

---

# Avoid Global Runtime

Không tạo runtime mới.

Không nested runtime.

---

Sai

```rust
Runtime::new()

Runtime::new()
```

---

# CPU Work

CPU intensive

Không chạy trên async executor.

Dùng

```rust
spawn_blocking()
```

---

# spawn_blocking

Ví dụ

```
compression

hashing

image

pdf

zip

```

---

Không block Tokio worker.

---

# Async Drop

Drop không async.

Không phụ thuộc Drop để đóng kết nối.

Hãy có

```
shutdown()

close()

stop()
```

---

# Futures Should Be Cancel Safe

Nếu future bị cancel

Không để

- leak

- inconsistent state

- half-written data

---

# Fairness

Loop dài

Nên yield.

Ví dụ

```rust
tokio::task::yield_now().await;
```

---

# Arc Philosophy

Arc chỉ giải quyết ownership.

Không giải quyết synchronization.

Đừng dùng Arc như thuốc chữa.

---

# Mutex Philosophy

Mutex chỉ bảo vệ dữ liệu.

Không bảo vệ workflow.

---

# RwLock

Chỉ dùng nếu

Reader

>>

Writer

Nếu write nhiều

Mutex thường nhanh hơn.

---

# Notify

Notify

→ signal

Không truyền data.

---

# Atomic

Nếu chỉ

- counter

- bool

- flag

Đừng dùng Mutex.

Dùng Atomic.

---

# OnceLock

Khởi tạo một lần.

Thread-safe.

Không cần Mutex.

---

# LazyLock

Global lazy initialization.

Ưu tiên hơn static mut.

---

# DashMap

Concurrent HashMap.

Chỉ dùng khi thực sự cần concurrent access.

Không thay HashMap mặc định.

---

# Async Error Handling

Không

```rust
.unwrap()
```

trong task.

Task chết âm thầm rất khó debug.

---

# JoinHandle

Luôn kiểm tra

```rust
.await
```

để biết task có panic không.

---

# Async Testing

Ưu tiên

```rust
#[tokio::test]
```

Không tạo runtime bằng tay.

---

# Logging

Dùng

```
tracing

instrument

span

```

Không

```
println!
```

---

# Async Code Smells

Nếu thấy

```
Arc<Mutex<Arc<Mutex<T>>>>

```

→ Thiết kế sai.

---

Nếu thấy

```
spawn()

spawn()

spawn()

spawn()

```

→ Thiết kế sai.

---

Nếu thấy

```
clone()

clone()

clone()

```

để task chạy được

→ Ownership sai.

---

Nếu thấy

```
Mutex

↓

await

↓

unlock
```

→ Sai.

---

Nếu thấy

```
unbounded channel
```

khắp nơi

→ Sai.

---

# Concurrency Decision Tree

Trước khi thêm synchronization

Hãy hỏi

□ Ownership có giải quyết được không?

↓

□ Message passing có giải quyết được không?

↓

□ Arc có đủ không?

↓

□ Atomic có đủ không?

↓

□ Mutex có thật sự cần không?

↓

□ RwLock có tốt hơn Mutex không?

↓

□ Actor model có đơn giản hơn không?

Chỉ dùng Mutex khi tất cả lựa chọn phía trên không phù hợp.

---

# Async Checklist

Trước khi merge code

□ Có task nào không shutdown không?

□ Có JoinHandle bị bỏ không?

□ Có Mutex giữ qua await không?

□ Có unbounded channel không?

□ Có blocking IO trên runtime không?

□ Có CPU work trong async không?

□ Có timeout không?

□ Có cancellation không?

□ Có backpressure không?

□ Có clone chỉ để spawn không?

□ Có Arc<Mutex<_>> lồng nhau không?

---

# End of Part 4

Async Rust không phải về `async/await`.

Mà là về:

- Ownership.
- Scheduling.
- Cancellation.
- Backpressure.
- Message Passing.
- Structured Concurrency.

Nếu thấy nhiều `Arc<Mutex<_>>`, nhiều `clone()`, nhiều `spawn()`, hãy xem lại thiết kế trước khi sửa code.

---

# Part 5 - Performance, Memory & Data Structures

Rust rất nhanh.

Nhưng Rust **không tự động nhanh**.

Hiệu năng đến từ:

- đúng data structure
- ít allocation
- ít clone
- cache locality
- zero-copy
- đúng ownership

Không phải từ việc thêm `unsafe`.

---

# Performance Philosophy

Thứ tự ưu tiên

```
Correctness

↓

Safety

↓

Profiling

↓

Optimization
```

Không tối ưu trước khi benchmark.

---

# Measure First

Không đoán.

Không "cảm thấy".

Benchmark trước.

Profile sau.

Tối ưu cuối cùng.

---

# Don't Fear Allocation

Allocation không xấu.

Allocation không cần thiết mới xấu.

---

# Every Allocation Matters

Mỗi lần

```
String::new()

Vec::new()

format!()

to_string()

clone()

collect()

```

đều có thể tạo allocation.

Hãy biết mình đang allocate ở đâu.

---

# Reference First

Nếu chỉ đọc dữ liệu

Đừng tạo

```
String

Vec

```

Hãy dùng

```
&str

&[u8]

&[T]

```

---

# Clone is Allocation

Hầu hết

```
String::clone()

Vec::clone()

HashMap::clone()

```

đều allocate.

Clone không miễn phí.

---

# Prefer Borrow

Sai

```rust
let name = user.name.clone();
```

Đúng

```rust
let name = &user.name;
```

---

# Zero Copy

Mục tiêu

Không copy dữ liệu nếu ownership không yêu cầu.

Ưu tiên

```
&str

&[u8]

Cow

Bytes

Arc<[u8]>

```

---

# Cow

Nếu dữ liệu

- thường borrow
- đôi khi own

Dùng

```rust
Cow<'a, str>
```

Thay vì

```
String
```

---

# Bytes

Networking

File

Protocol

Buffer

Ưu tiên

```
bytes::Bytes
```

Thay vì clone Vec<u8>.

Bytes clone chỉ tăng reference count.

---

# Arc<[T]>

Nếu dữ liệu immutable

chia sẻ nhiều nơi

→ dùng

```
Arc<[T]>
```

---

# Box<[T]>

Nếu

- immutable
- fixed size

→ Box<[T]>

nhỏ hơn Vec.

---

# SmallVec

Nếu

95%

trường hợp chỉ có vài phần tử.

Dùng

```
SmallVec
```

để tránh heap allocation.

Ví dụ

```
4

8

16

```

phần tử.

---

# ArrayVec

Capacity cố định.

Không heap.

Rất tốt cho

- parser

- protocol

- embedded

---

# Vec

Collection mặc định.

Nếu không biết dùng gì

→ Vec.

---

# VecDeque

Nếu cần

```
push_front()

pop_front()

queue
```

Không dùng Vec.

---

# LinkedList

Không dùng.

Hầu như luôn chậm hơn Vec.

Cache locality rất kém.

---

# HashMap

Lookup nhanh.

Không giữ thứ tự.

---

# BTreeMap

Nếu cần

- sorted

- range

- iteration theo key

---

# IndexMap

Nếu cần

```
HashMap

+

preserve insertion order
```

---

# HashSet

Membership.

Không dùng Vec nếu chỉ cần contains.

---

# BinaryHeap

Priority queue.

Đừng tự implement heap.

---

# Vec Capacity

Nếu biết kích thước.

Dùng

```rust
Vec::with_capacity()
```

---

Sai

```rust
Vec::new()

push

push

push
```

---

# Reserve

Nếu sắp thêm nhiều phần tử.

```
reserve()

reserve_exact()
```

---

# Shrink

Sau khi dùng xong.

Có thể

```
shrink_to_fit()
```

nếu cần.

---

# Avoid Temporary String

Sai

```rust
format!("{}", value)
```

chỉ để compare.

---

Sai

```rust
to_string()
```

rồi lại

```
&str
```

---

# Avoid Repeated Allocation

Sai

```rust
loop {

String::new()

}
```

---

Đúng

Reuse buffer.

---

# String Reuse

Nếu buffer dùng nhiều lần.

```
clear()
```

thay vì tạo mới.

---

# mem::take()

Rất hữu ích.

Thay vì

```rust
clone()

clear()
```

Có thể

```
mem::take()
```

---

# mem::replace()

Thay giá trị

không cần clone.

---

# swap()

Đừng clone.

Có thể

```
mem::swap()
```

---

# take()

Option

Vec

String

đều có

```
take()
```

---

# Cache Locality

Vec nhanh hơn LinkedList

không phải vì thuật toán.

Mà vì cache CPU.

Luôn ưu tiên contiguous memory.

---

# Flat Data

Ưu tiên

```
Vec<T>
```

Thay vì

```
Vec<Box<T>>
```

nếu không cần.

---

# Avoid Pointer Chasing

Ít

```
Box

Rc

Arc

```

hơn

→ cache tốt hơn.

---

# Copy vs Clone

Copy

```
u32

usize

bool

char

```

rất rẻ.

Clone

```
String

Vec

HashMap

```

thường đắt.

---

# Inline Small Types

Struct nhỏ

Copy

thường tốt hơn Arc.

---

# Prefer Slice

Sai

```rust
Vec<T>
```

Đúng

```rust
&[T]
```

---

# Prefer Arrays

Nếu kích thước compile-time.

```
[T; N]
```

hơn

```
Vec<T>
```

---

# Avoid Boxing Too Early

Không

```
Box<T>
```

chỉ vì compiler gợi ý.

---

# Enum Memory

Enum lớn

Có thể tốn RAM.

Nếu một variant rất lớn.

Cân nhắc

```
Box
```

variant đó.

---

# Lazy Allocation

Không allocate trước khi cần.

---

# Avoid Double Buffer

Sai

```
read

↓

Vec

↓

clone

↓

Vec

```

---

# Streaming

Nếu dữ liệu lớn

Đừng load toàn bộ.

Hãy stream.

---

# Prefer Iterators

Iterator thường

Zero-cost.

Compiler tối ưu rất tốt.

---

# Avoid Intermediate Collection

Sai

```rust
collect()

↓

map()

↓

collect()
```

---

Nối iterator.

---

# String Formatting

Nếu append nhiều lần.

Đừng

```
format!

+

format!

+

format!
```

---

Dùng

```
write!

push_str()

```

---

# Hashing

Đừng hash nhiều lần.

Cache nếu hợp lý.

---

# Profiling

Ưu tiên

```
criterion

cargo bench

cargo flamegraph

perf

heaptrack

valgrind

```

Không tối ưu bằng cảm giác.

---

# SIMD

Đừng tự viết SIMD.

Ưu tiên

```
memchr

portable_simd

```

hoặc crate mature.

---

# Unsafe

Không dùng unsafe để tối ưu

nếu chưa benchmark.

---

# Performance Myths

Sai

```
unsafe

↓

nhanh hơn
```

Không chắc.

---

Sai

```
clone

↓

rẻ
```

Không.

---

Sai

```
async

↓

nhanh hơn
```

Không.

---

Sai

```
HashMap

↓

luôn nhanh nhất
```

Không.

---

Sai

```
BTreeMap

↓

luôn chậm
```

Không.

---

# Allocation Checklist

Trước khi merge

□ Có clone không?

□ Có format! không cần thiết?

□ Có String::new trong loop?

□ Có Vec::new trong loop?

□ Có collect dư thừa?

□ Có allocation lặp lại?

□ Có thể borrow không?

□ Có thể Cow không?

□ Có thể Bytes không?

□ Có thể reuse buffer không?

---

# Collection Decision Tree

```
Mặc định?

↓

Vec

↓

Queue?

↓

VecDeque

↓

Lookup?

↓

HashMap

↓

Ordered?

↓

BTreeMap

↓

Ordered HashMap?

↓

IndexMap

↓

Few elements?

↓

SmallVec

↓

Fixed capacity?

↓

ArrayVec

↓

Shared immutable?

↓

Arc<[T]>

↓

Read-only slice?

↓

&[T]
```

---

# Performance Rules

Không clone nếu borrow được.

Không allocate nếu reuse được.

Không copy nếu share được.

Không benchmark bằng cảm giác.

Không optimize trước khi profile.

Không dùng unsafe nếu std đã đủ.

---

# AI Performance Rules

Nếu AI sinh code

- Không `.clone()` liên tục.
- Không `collect()` rồi `iter()` lại ngay.
- Không `Vec<String>` nếu `&str` đủ.
- Không `String` nếu `Cow<'_, str>` phù hợp.
- Không `Vec<u8>` nếu `Bytes` phù hợp.
- Không `HashMap` nếu chỉ có vài key cố định (`phf` hoặc `match` có thể tốt hơn).
- Không tạo nhiều buffer tạm trong pipeline.
- Không thêm `Box`, `Arc` hoặc `Rc` chỉ để compiler hết lỗi.

---

# End of Part 5

Hiệu năng trong Rust không đến từ `unsafe`.

Nó đến từ:

- Ownership đúng.
- Borrow đúng.
- Data structure đúng.
- Cache locality tốt.
- Ít allocation.
- Zero-copy khi hợp lý.
- Benchmark và profiling trước khi tối ưu.

---

# Part 6 - Project Architecture, Modules & Dependency Management

Code có thể rất "Rusty" nhưng project vẫn có thể rất tệ.

Rust không chỉ là viết function đẹp.

Quan trọng hơn là:

- Module Organization
- Dependency Direction
- Domain Separation
- Public API Design
- Crate Boundaries

Kiến trúc tốt giúp project mở rộng mà không trở thành "God Project".

---

# Architecture Philosophy

Một project tốt nên có

- module nhỏ
- dependency rõ ràng
- public API nhỏ
- coupling thấp
- cohesion cao

---

# Organize by Domain

KHÔNG tổ chức project theo loại file.

Sai

```
models/

helpers/

utils/

controllers/

interfaces/

```

Đúng

```
auth/

chat/

network/

storage/

media/

crypto/

protocol/

transport/
```

Module nên phản ánh business domain.

Không phản ánh ngôn ngữ.

---

# One Module = One Responsibility

Một module chỉ nên có một lý do để thay đổi.

Sai

```
network/

↓

socket

↓

config

↓

parser

↓

crypto

↓

database
```

---

Đúng

```
transport/

protocol/

crypto/

storage/

```

---

# Avoid Utils

Nếu project xuất hiện

```
utils.rs

helpers.rs

common.rs

misc.rs

shared.rs
```

Hãy dừng lại.

Thông thường

→ Architecture đang sai.

---

Utilities thường là nơi mọi thứ bị nhét vào.

Cuối cùng không ai biết file đó dùng để làm gì.

---

# Prefer Domain Names

Tên module nên là

```
transfer

peer

channel

identity

presence

```

Không phải

```
manager

processor

helper

handler

```

---

# Avoid God Modules

Một module

1000+

dòng

↓

chia nhỏ.

---

Một module không nên biết mọi thứ.

---

# Module Hierarchy

Module nên tạo thành cây.

Không phải mạng nhện.

Ví dụ

```
protocol

↓

frame

↓

packet

↓

codec
```

Không

```
frame

↓

packet

↓

frame

↓

packet
```

---

# Keep Modules Independent

Module càng ít phụ thuộc nhau càng tốt.

Nếu

```
A

↓

B

↓

C

↓

A
```

Có dependency vòng.

Thiết kế sai.

---

# Layering

Ví dụ

```
Application

↓

Domain

↓

Infrastructure

↓

OS
```

Không gọi ngược lên trên.

---

# Dependency Direction

Dependency chỉ nên đi xuống.

Không ngược lên.

Ví dụ

```
UI

↓

Service

↓

Repository

↓

Storage
```

Storage không được gọi UI.

---

# Public API First

Module nên expose ít nhất có thể.

Mặc định

```
private
```

---

# pub is Expensive

Mỗi

```
pub
```

là một contract.

Không public nếu không cần.

---

Sai

```rust
pub struct Foo

pub struct Bar

pub struct Baz

pub fn ...

pub fn ...
```

mọi nơi.

---

# Prefer pub(crate)

Nếu chỉ dùng nội bộ crate.

Dùng

```rust
pub(crate)
```

---

# Keep API Small

Expose

5 function

tốt hơn

50 function.

---

# Re-export Carefully

Nếu cần API đẹp

Dùng

```rust
pub use
```

Đừng expose toàn bộ cấu trúc thư mục.

---

# Avoid Deep Module Trees

Sai

```
a

↓

b

↓

c

↓

d

↓

e

↓

f
```

---

Ưu tiên

2–4 level.

---

# Crate Philosophy

Một crate

=

một responsibility.

---

Đừng tạo

```
common

core

base

shared
```

chứa mọi thứ.

---

# Split by Domain

Ví dụ

```
vchat-crypto

vchat-protocol

vchat-storage

vchat-transport

vchat-media

```

---

Không

```
vchat-utils

vchat-common

vchat-base

```

---

# Crate Dependencies

Dependency nên tạo DAG.

Không dependency vòng.

---

# Feature Flags

Dùng Cargo Feature.

Không

```
cfg

cfg

cfg

```

khắp nơi.

---

Feature phải độc lập.

Không phụ thuộc lẫn nhau.

---

# Minimize Dependencies

Mỗi dependency mới

là

- compile time
- security risk
- maintenance
- MSRV risk
- license risk

---

# Before Adding a Crate

Tự hỏi

□ Std có làm được không?

↓

□ Đã có crate trong project chưa?

↓

□ Crate mature không?

↓

□ Maintenance tốt không?

↓

□ License phù hợp không?

↓

□ Có thực sự cần không?

---

# Prefer Mature Crates

Ưu tiên crate

- nhiều người dùng
- nhiều maintainer
- tài liệu tốt
- test đầy đủ
- update thường xuyên

---

Đừng chọn crate chỉ vì

```
100 dòng code ít hơn
```

---

# MSRV

Đặt

Minimum Supported Rust Version.

Không vô tình dùng API mới.

---

# Workspace

Project lớn

Ưu tiên Cargo Workspace.

---

Workspace nên chia

```
library

application

tools

examples

benchmarks
```

---

# Internal APIs

Không public crate nội bộ nếu không cần.

---

# Examples

Crate nên có

```
examples/
```

nếu API phức tạp.

---

# Tests

Không để

```
main.rs
```

2000 dòng.

Test cũng vậy.

---

# Integration Tests

Ưu tiên

```
tests/
```

thay vì chỉ unit test.

---

# Benches

Benchmark

```
benches/
```

Không để trong src.

---

# Fuzz

Nếu parser

protocol

decoder

↓

nên có fuzzing.

---

# Build Scripts

Tránh

```
build.rs
```

nếu không thật sự cần.

---

# Macros

Macro chỉ khi

- giảm lặp
- tăng type safety

Không thay function.

---

# Avoid Copy-Paste

Nếu copy lần thứ ba

↓

Refactor.

---

# Keep Files Reasonable

Không có giới hạn cứng.

Nhưng

1000+

dòng

↓

xem xét chia.

---

# Documentation

Mỗi module nên có

- mục đích
- responsibility
- public API

---

# README

Workspace nên có

README giải thích

- architecture
- crate graph
- build
- testing

---

# Logging

Đừng log ở mọi layer.

Log gần boundary.

Ví dụ

```
API

Network

Database

Filesystem
```

Không log trong mọi helper.

---

# Configuration

Không hardcode.

Ưu tiên

```
serde

toml

yaml

```

---

# Constants

Nếu là domain

↓

newtype

enum

Nếu là config

↓

const

---

# Naming

Tên module nên là danh từ.

Tên function nên là động từ.

Tên type nên mô tả domain.

---

# Avoid Manager Syndrome

Không tạo

```
UserManager

SocketManager

TransferManager

ConfigManager

```

Nếu có thể.

Tên

"Manager"

thường che giấu trách nhiệm thực sự.

---

# Prefer Explicit Types

Không

```
Data

Info

Object

Item

Manager

Helper

```

---

Đặt tên

```
TransferQueue

PeerDirectory

ChunkVerifier

FrameDecoder

```

---

# Circular Dependencies

Không

```
auth

↓

storage

↓

network

↓

auth
```

---

Dependency phải một chiều.

---

# Architecture Checklist

Trước khi merge

□ Module có một responsibility không?

□ Có utils/common/helper không?

□ Có dependency vòng không?

□ Có quá nhiều pub không?

□ Có expose implementation không?

□ Có crate nào responsibility không rõ không?

□ Có dependency mới không cần thiết không?

□ Có thể tách module nhỏ hơn không?

□ API có tối giản không?

□ Có README hoặc rustdoc cho module lớn không?

---

# AI Architecture Rules

Nếu AI sinh code

Không được tạo

```
utils.rs

helpers.rs

common.rs

misc.rs

manager.rs

```

trừ khi người dùng yêu cầu rõ ràng.

---

Không tạo

```
Arc<Mutex<_>>
```

làm trung tâm của toàn bộ project.

---

Không tạo dependency vòng giữa module.

---

Không expose toàn bộ struct bằng

```
pub
```

---

Không tạo crate

```
shared

common

base

core

```

chỉ để chứa mọi thứ.

---

Luôn tổ chức theo domain trước.

---

# End of Part 6

Một project Rust tốt không chỉ có code đẹp.

Nó còn phải:

- Dễ điều hướng.
- Dễ mở rộng.
- Dễ kiểm thử.
- Dễ thay thế implementation.
- Có dependency rõ ràng.
- Có public API nhỏ.
- Tổ chức theo domain thay vì theo kỹ thuật.

---

# Part 7 - Unsafe Rust, FFI & Low-level Programming

Unsafe không phải là "Rust không an toàn".

Unsafe là nơi **bạn** chịu trách nhiệm thay compiler chứng minh tính đúng đắn.

Nguyên tắc:

> **Unsafe should be isolated, justified, documented, tested, and minimized.**

---

# Unsafe Philosophy

Unsafe không tắt borrow checker.

Unsafe chỉ cho phép compiler tin tưởng bạn.

Bạn phải đảm bảo:

- Không UB
- Không data race
- Không dangling pointer
- Không invalid reference
- Không aliasing sai
- Không vi phạm lifetime

---

# Unsafe Is Not Faster

Sai

```
unsafe

↓

faster
```

Không có gì đảm bảo.

Nhiều đoạn unsafe còn chậm hơn std.

---

# Safe First

Luôn theo thứ tự

```
Safe std

↓

Battle-tested crate

↓

unsafe
```

---

# Before Writing unsafe

Tự hỏi

□ Std có API chưa?

□ Có crate mature không?

□ Có cách safe không?

□ Benchmark chưa?

□ Unsafe có thật sự cần không?

Nếu chưa

→ Không viết unsafe.

---

# Isolate Unsafe

Sai

```rust
unsafe fn parse(...) {

}
```

200 dòng.

---

Đúng

```rust
fn parse(...) {

    // safe

    unsafe {
        ...
    }

    // safe
}
```

Unsafe càng nhỏ càng tốt.

---

# One Unsafe Block

Một unsafe block

↓

Một responsibility.

Không nhét nhiều thao tác.

---

# Safety Comments

Mọi unsafe block đều phải có

```rust
// SAFETY:
```

Ví dụ

```rust
// SAFETY:
//
// Pointer được tạo từ slice hợp lệ.
// Không alias mutable.
// Lifetime được đảm bảo.
unsafe {

}
```

Không có comment

↓

Không merge.

---

# Unsafe Checklist

Trước khi merge

□ Pointer hợp lệ?

□ Alignment đúng?

□ Lifetime đúng?

□ Không dangling?

□ Không alias mutable?

□ Không out-of-bounds?

□ Không data race?

□ Có benchmark?

□ Có test?

□ Có comment SAFETY?

---

# Undefined Behavior

UB không phải panic.

UB nghĩa là

Compiler có thể làm bất cứ điều gì.

---

Ví dụ UB

- Dangling pointer
- Double free
- Use after free
- Invalid reference
- Data race
- Out of bounds pointer
- Reading uninitialized memory

---

# References

Reference luôn phải hợp lệ.

Không bao giờ tạo

```rust
&T
```

từ invalid pointer.

---

# Raw Pointer

Raw pointer

```
*const T

*mut T
```

không có guarantee.

Không dereference ngoài unsafe.

---

# Prefer References

Nếu có thể

```
&T

&mut T
```

↓

Raw pointer.

---

# Null Pointer

Rust reference

Không bao giờ null.

Nếu có thể null

Dùng

```
Option<&T>

Option<NonNull<T>>
```

---

# NonNull

Nếu pointer chắc chắn không null.

Ưu tiên

```
NonNull<T>
```

thay vì

```
*mut T
```

---

# MaybeUninit

Không dùng

```rust
mem::zeroed()
```

cho mọi type.

Ưu tiên

```
MaybeUninit<T>
```

---

# mem::zeroed

Rất nguy hiểm.

Một số type không thể zero.

---

# mem::uninitialized

Không dùng.

Đã deprecated.

---

# Drop

Không gọi

```
drop_in_place
```

nếu không hiểu ownership.

---

# Double Drop

Không được

- drop hai lần
- free hai lần

---

# ManuallyDrop

Chỉ dùng khi thật sự cần.

Nếu không hiểu

↓

Không dùng.

---

# Pin

Pin không phải optimization.

Pin dùng khi

Địa chỉ object không được thay đổi.

---

Không dùng Pin nếu chưa hiểu.

---

# Self-referential Struct

Không tự thiết kế.

Nếu cần

↓

Thiết kế lại ownership.

---

# Send

Type có thể chuyển sang thread khác.

---

# Sync

Reference có thể chia sẻ giữa nhiều thread.

---

# Không Implement Send/Sync Bằng Tay

Trừ khi cực kỳ hiểu.

Sai

```rust
unsafe impl Send for Foo {}
```

Nếu không chứng minh được.

---

# repr(C)

Chỉ dùng cho

- FFI
- binary layout

---

Không dùng repr(C) cho mọi struct.

---

# repr(transparent)

Dùng cho newtype.

Ví dụ

```rust
struct UserId(Uuid);
```

---

# repr(packed)

Tránh.

Dễ tạo UB.

---

# repr(align)

Chỉ dùng khi thực sự cần alignment.

---

# Transmute

```
mem::transmute()
```

là API nguy hiểm nhất std.

Hầu hết trường hợp

↓

Có API an toàn hơn.

---

Không transmute nếu chưa đọc Rust Nomicon.

---

# Slice

Không tự tạo slice từ pointer

nếu không chắc length đúng.

---

# String

Không tự tạo String từ bytes

nếu chưa kiểm tra UTF-8.

Ưu tiên

```
from_utf8()

from_utf8_lossy()
```

---

# FFI Philosophy

Boundary phải nhỏ.

Không để unsafe lan vào business logic.

---

# FFI Types

Không truyền

```
String

Vec

HashMap

```

qua C.

Ưu tiên

```
CString

CStr

slice

pointer

repr(C)
```

---

# CString

Rust

↓

C

---

# CStr

C

↓

Rust

---

# Ownership Across FFI

Luôn rõ

Ai allocate?

Ai free?

Ai sở hữu?

---

# Panic Across FFI

Không panic qua boundary C.

Catch trước.

---

# ABI

Không assume Rust ABI ổn định.

FFI

↓

extern "C"

---

# Unsafe Trait

Nếu trait có invariant đặc biệt.

Có thể dùng

```
unsafe trait
```

Nhưng rất hiếm.

---

# Interior Mutability

Unsafe không phải giải pháp.

Ưu tiên

```
Cell

RefCell

Mutex

RwLock

Atomic
```

---

# Atomics

Nếu chỉ cần

- bool
- counter

↓

Atomic.

Không Mutex.

---

# Lock-free

Không tự viết lock-free structure

trừ khi thật sự cần.

Rất khó đúng.

---

# Inline Assembly

```
asm!
```

Chỉ khi

- kernel
- embedded
- compiler intrinsic

---

# Miri

Unsafe code

↓

Chạy Miri.

---

# Sanitizers

Ưu tiên

- AddressSanitizer
- ThreadSanitizer
- LeakSanitizer

---

# Fuzzing

Unsafe parser

↓

Bắt buộc fuzz.

---

# Unsafe Module

Nếu project lớn.

Tạo

```
unsafe/

```

hoặc

```
ffi/

```

Không để unsafe rải rác.

---

# Audit

Unsafe phải audit định kỳ.

---

# Unsafe Review

Code review phải hỏi

- Invariant là gì?

- Có UB không?

- Lifetime đúng chưa?

- Pointer hợp lệ?

- Alignment đúng?

- Có test chưa?

- Có benchmark chưa?

- Có thể viết safe không?

---

# AI Rules

Nếu AI sinh

```
unsafe
```

AI phải giải thích

- Vì sao cần unsafe?

- Có API safe thay thế không?

- Safety invariant là gì?

- Có UB nào có thể xảy ra?

Nếu không trả lời được

↓

Không dùng unsafe.

---

# AI Must Never

Không được

- dùng transmute để sửa lỗi type
- thêm unsafe vì borrow checker
- unsafe impl Send/Sync tùy tiện
- dereference raw pointer không giải thích
- bỏ comment SAFETY
- dùng MaybeUninit sai cách
- dùng mem::zeroed cho type bất kỳ

---

# Low-level Crates

Ưu tiên

```
bytemuck

zerocopy

memchr

bytes

portable-simd

crossbeam

libc

```

thay vì tự viết.

---

# Rust Nomicon

Nếu định viết unsafe.

Đọc Rust Nomicon trước.

Không đoán.

---

# Unsafe Decision Tree

```
Có API safe?

↓

YES

↓

Dùng safe.

↓

NO

↓

Có crate mature?

↓

YES

↓

Dùng crate.

↓

NO

↓

Benchmark chứng minh cần?

↓

NO

↓

Không viết.

↓

YES

↓

Unsafe nhỏ nhất có thể.

↓

Comment SAFETY.

↓

Test.

↓

Fuzz.

↓

Miri.

↓

Review.
```

---

# Final Rules

Unsafe là công cụ cuối cùng.

Không phải công cụ đầu tiên.

Nếu có thể viết safe

↓

Viết safe.

Nếu phải viết unsafe

↓

Unsafe phải:

- nhỏ
- cô lập
- documented
- benchmarked
- tested
- fuzzed
- audited

Mọi dòng unsafe đều phải có lý do tồn tại.

---

# Part 8 - Testing, Benchmarking, Code Quality & CI

Một project Rust tốt không phải là project compile được.

Một project Rust tốt phải chứng minh được rằng code:

- đúng
- ổn định
- không UB
- không regression
- dễ refactor

Testing không phải để "tăng coverage".

Testing để tăng **niềm tin khi thay đổi code**.

---

# Quality Pyramid

Ưu tiên

```
Correctness

↓

Tests

↓

Benchmarks

↓

Documentation

↓

Optimization
```

Đừng benchmark code chưa test.

---

# Testing Philosophy

Một test tốt phải

- deterministic
- độc lập
- nhanh
- dễ đọc
- dễ debug

---

# Test Behavior

Test hành vi.

Không test implementation.

Sai

```text
Hàm này gọi function A
↓

PASS
```

Đúng

```text
Input

↓

Output

↓

PASS
```

---

# Arrange - Act - Assert

Mỗi test nên theo cấu trúc

```
Arrange

↓

Act

↓

Assert
```

Ví dụ

```rust
#[test]
fn parses_valid_uuid() {
    // Arrange

    let input = "...";

    // Act

    let result = parse(input);

    // Assert

    assert!(result.is_ok());
}
```

---

# One Assertion Purpose

Một test

↓

Một responsibility.

Sai

```text
Test login

↓

20 assert
```

---

# Naming

Tên test phải mô tả behavior.

Đúng

```
rejects_invalid_packet()

parses_empty_frame()

returns_timeout_when_peer_offline()

```

Sai

```
test1()

basic()

works()

```

---

# Unit Tests

Test từng module.

Đặt gần code.

```rust
#[cfg(test)]
mod tests {

}
```

---

# Integration Tests

Test public API.

Đặt trong

```
tests/
```

Không phụ thuộc implementation.

---

# Property Testing

Nếu có parser

codec

serializer

crypto

↓

Dùng

```
proptest
```

hoặc

```
quickcheck
```

---

Ví dụ

Encode

↓

Decode

↓

Bằng dữ liệu ngẫu nhiên.

---

# Fuzz Testing

Parser

Protocol

Network

Binary

↓

Bắt buộc fuzz.

---

Ưu tiên

```
cargo fuzz
```

---

# Snapshot Testing

Nếu output rất lớn.

Có thể dùng

```
insta
```

---

# Golden Tests

Nếu protocol ổn định.

Lưu

```
input

↓

expected output
```

để chống regression.

---

# Regression Tests

Mỗi bug đã fix

↓

Thêm test.

Không để bug quay lại.

---

# Boundary Tests

Luôn test

- empty
- one element
- maximum
- minimum
- invalid input

---

# Panic Tests

Nếu panic là expected.

Dùng

```rust
#[should_panic]
```

---

# Async Tests

Ưu tiên

```rust
#[tokio::test]
```

Không tạo runtime bằng tay.

---

# Time-sensitive Tests

Không dùng

```
sleep()
```

để chờ.

Ưu tiên

- timeout
- Notify
- Barrier
- Channel

---

# Deterministic Tests

Không dùng

```
thread::sleep()
```

để đồng bộ.

---

# Mocking

Mock IO.

Không mock business logic.

---

# Dependency Injection

Thay vì mock framework.

Truyền trait hoặc interface nhỏ.

---

# Test Data

Không hardcode lung tung.

Đặt trong

```
fixtures/
```

---

# Test Isolation

Test không được phụ thuộc

- thứ tự chạy
- file tạo bởi test khác
- network thật
- database thật

---

# Temporary Files

Dùng

```
tempfile
```

---

# Benchmark Philosophy

Benchmark

↓

Đo hiệu năng.

Không đo tính đúng.

---

# Criterion

Ưu tiên

```
criterion
```

Không

```
Instant
```

---

# Benchmark Layout

Đặt trong

```
benches/
```

Không trong

```
tests/
```

---

# Measure Hot Paths

Benchmark

- parser
- codec
- crypto
- allocation
- protocol

Không benchmark getter.

---

# Compare Before/After

Benchmark phải trả lời

Code mới

↓

Nhanh hơn?

Chậm hơn?

Bao nhiêu?

---

# Don't Guess Performance

Không

```
Tôi nghĩ nhanh hơn.
```

↓

Benchmark.

---

# Flamegraph

Nếu benchmark chậm.

Ưu tiên

```
cargo flamegraph
```

---

# Profiling

Đừng optimize khi chưa profile.

---

# Miri

Chạy

```
cargo miri test
```

cho

- unsafe
- pointer
- aliasing

---

# Sanitizers

Ưu tiên

- ASan
- TSan
- LSan

---

# Clippy

Bật

```
clippy::all

clippy::pedantic

clippy::cargo

clippy::nursery
```

---

# Restriction Lints

Chọn lọc

```
restriction
```

Không bật toàn bộ.

---

# Clippy Should Fail CI

Nếu lint quan trọng.

CI nên fail.

---

# rustfmt

Code style

↓

rustfmt.

Không tranh cãi style.

---

# Documentation Tests

Ví dụ trong rustdoc

↓

Compile được.

---

# Cargo Test

CI nên chạy

```
cargo test
```

mọi commit.

---

# Cargo Check

CI nên chạy

```
cargo check
```

---

# Cargo Clippy

CI nên chạy

```
cargo clippy
```

---

# Cargo Fmt

CI nên chạy

```
cargo fmt --check
```

---

# Cargo Doc

CI nên build

```
cargo doc
```

---

# Cargo Audit

Kiểm tra

- CVE
- vulnerable crate

---

# Cargo Deny

Kiểm tra

- license
- duplicate dependency
- advisory

---

# Cargo Nextest

Project lớn

↓

Ưu tiên

```
cargo-nextest
```

---

# MSRV Check

CI nên test

Minimum Supported Rust Version.

---

# Feature Matrix

Nếu có feature.

CI nên test

```
--all-features

--no-default-features
```

---

# Coverage

Coverage cao

≠

Code tốt.

Đừng viết test chỉ để tăng coverage.

---

# Test Speed

Test nên nhanh.

Nếu test > vài giây

↓

Xem lại.

---

# Ignore Tests

Chỉ dùng

```rust
#[ignore]
```

cho

- benchmark-like
- integration rất nặng

---

# Logging in Tests

Không

```
println!
```

khắp nơi.

Chỉ log khi debug.

---

# CI Philosophy

CI phải

- nhanh
- deterministic
- reproducible

---

# CI Pipeline

Ví dụ

```
cargo fmt --check

↓

cargo check

↓

cargo clippy

↓

cargo test

↓

cargo miri

↓

cargo fuzz (nightly schedule)

↓

cargo bench (optional)

↓

cargo audit

↓

cargo deny
```

---

# Code Review

Reviewer nên hỏi

- Test đâu?

- Benchmark đâu?

- Có regression test không?

- Có UB không?

- Có panic không?

- Có clone không?

- Có allocation dư không?

---

# AI Testing Rules

Nếu AI sinh code

AI phải sinh luôn

- unit test

- integration test (nếu cần)

- property test (nếu phù hợp)

Không chỉ sinh implementation.

---

Nếu AI sửa bug

↓

Phải thêm regression test.

---

Nếu AI tối ưu

↓

Phải benchmark.

---

Nếu AI thêm unsafe

↓

Phải có

- Miri

- fuzz

- test

---

# Testing Checklist

Trước khi merge

□ Unit test?

□ Integration test?

□ Property test?

□ Regression test?

□ Boundary test?

□ Async test?

□ Benchmark?

□ Clippy sạch?

□ rustfmt?

□ cargo check?

□ cargo doc?

□ Miri (nếu unsafe)?

□ Fuzz (nếu parser)?

□ CI pass?

---

# Quality Rules

Không merge

nếu

- chưa test

- chưa lint

- chưa format

- benchmark giảm mạnh

- thêm panic vô lý

- thêm unsafe không giải thích

---

# End of Part 8

Code đúng hôm nay chưa đủ.

Code phải:

- đúng ngày mai.
- đúng sau refactor.
- đúng sau tối ưu.
- đúng sau nhiều năm.

Testing không phải để chứng minh code hoạt động.

Testing để chứng minh code **không dễ bị phá hỏng**.


---

# Part 9 - Rust Ecosystem & Crate Selection Guide

Một trong những sai lầm lớn nhất khi học Rust là:

> **Viết lại thứ mà ecosystem đã giải quyết rất tốt.**

Rust có một ecosystem cực kỳ chất lượng.

Ưu tiên:

```
std

↓

battle-tested crate

↓

tự viết
```

Không làm ngược lại.

---

# Crate Selection Philosophy

Đừng chọn crate vì

- nhiều sao GitHub
- mới ra
- benchmark đẹp

Hãy chọn vì

- mature
- nhiều người dùng
- API ổn định
- tài liệu tốt
- bảo trì tích cực

---

# Before Adding a Dependency

Tự hỏi

□ std có làm được không?

↓

□ Project đã có crate tương tự chưa?

↓

□ Có đáng thêm compile time không?

↓

□ Có đáng thêm security risk không?

↓

□ Có đáng thêm maintenance không?

---

# Async Runtime

## tokio

Dùng khi

- server
- client
- QUIC
- TCP
- UDP
- filesystem async
- timer

Đây là runtime mặc định của ecosystem.

---

## async-std

Ít dùng cho project mới.

Nếu bắt đầu mới

↓

Tokio.

---

## futures

Không phải runtime.

Là utility crate.

Rất nên có.

---

## tokio-util

Dùng cho

- CancellationToken
- codec
- framed transport

Rất đáng dùng.

---

# Error Handling

## thiserror

Library

↓

Luôn ưu tiên.

---

## anyhow

Application

↓

Luôn ưu tiên.

Không dùng trong library public API.

---

## eyre / color-eyre

CLI

Tool

↓

Hiển thị lỗi đẹp.

---

# Serialization

## serde

Gần như bắt buộc.

Không tự viết serializer.

---

## serde_json

JSON.

---

## bincode

Binary serialization.

Nhanh.

Không self-describing.

---

## postcard

Embedded

Protocol

No-std

↓

Rất tốt.

---

## rmp-serde

MessagePack.

---

## ciborium

CBOR.

---

# Networking

## bytes

★★★★★

Nếu làm

- protocol
- TCP
- QUIC
- HTTP

↓

Nên dùng.

Clone gần như miễn phí.

---

## quinn

QUIC.

Lựa chọn hàng đầu hiện nay.

---

## axum

HTTP server.

API hiện đại.

---

## hyper

HTTP low-level.

---

## reqwest

HTTP client.

Không tự viết client.

---

## rustls

TLS.

Ưu tiên hơn OpenSSL nếu có thể.

---

# Crypto

## ring

Crypto production.

---

## blake3

Hash cực nhanh.

Rất nên dùng.

---

## sha2

SHA-256.

---

## zeroize

Xóa secret khỏi RAM.

---

## secrecy

Bao bọc secret.

Giảm nguy cơ log nhầm.

---

## rand

Random number.

Không tự viết RNG.

---

# Collections

## smallvec

Ít phần tử.

↓

Tránh heap allocation.

---

## arrayvec

Capacity cố định.

---

## indexmap

HashMap có thứ tự.

---

## dashmap

Concurrent HashMap.

Chỉ dùng khi thật sự cần.

---

## slotmap

ID ổn định.

Game.

ECS.

---

## slab

Object pool.

---

# Concurrency

## parking_lot

Mutex

RwLock

Condvar

Nhanh hơn std trong nhiều trường hợp.

---

## crossbeam

Channel

Atomic

Lock-free

Thread utility.

Rất mature.

---

## rayon

CPU parallelism.

Không dùng Tokio cho CPU-bound.

---

## flume

Channel API rất đẹp.

Có thể thay mpsc trong nhiều trường hợp.

---

# Parsing

## winnow

Parser combinator hiện đại.

Ưu tiên hơn nom cho project mới.

---

## nom

Rất mạnh.

Protocol.

Binary.

---

## memchr

Tìm byte cực nhanh.

Đừng tự loop.

---

## aho-corasick

Nhiều pattern.

---

## regex

Regex.

Compile trước.

Không compile trong loop.

---

# Logging

## tracing

★★★★★

Chuẩn hiện nay.

Không println!.

---

## tracing-subscriber

Subscriber cho tracing.

---

## tracing-error

Error context.

---

# CLI

## clap

CLI parser.

---

## dialoguer

Interactive CLI.

---

## indicatif

Progress bar.

---

# Configuration

## toml

Cargo.

Config.

---

## config

Merge config.

---

## envy

Environment variable.

---

# Filesystem

## tempfile

Temporary file.

---

## walkdir

Recursive directory.

---

## ignore

Gitignore-aware traversal.

---

# Date & Time

## time

Ưu tiên.

API hiện đại.

---

## chrono

Legacy.

Vẫn rất phổ biến.

---

# UUID

## uuid

Chuẩn.

Không tự generate UUID.

---

# IDs

Nếu không cần UUID.

Có thể dùng

- snowflake
- ulid
- nanoid

Tùy yêu cầu.

---

# Database

## sqlx

Compile-time checked SQL.

---

## sea-orm

ORM.

---

## rusqlite

SQLite.

---

# WebSocket

## tokio-tungstenite

WebSocket.

---

# Compression

## zstd

★★★★★

Ưu tiên.

---

## flate2

gzip.

---

# Images

## image

Chuẩn.

---

# Benchmark

## criterion

★★★★★

Chuẩn benchmark.

---

# Testing

## proptest

Property testing.

---

## insta

Snapshot testing.

---

## rstest

Parameterized tests.

---

# Fuzz

## cargo-fuzz

LLVM libFuzzer.

---

# Documentation

## cargo-doc

Sinh rustdoc.

---

# Build

## cc

Compile C.

---

## bindgen

Generate binding.

---

## cbindgen

Generate C header.

---

# Macros

## paste

Identifier concatenation.

---

## derive_more

Giảm boilerplate.

---

# Unsafe

## bytemuck

Safe cast.

---

## zerocopy

Zero-copy parsing.

---

# Lazy Initialization

## once_cell

Trước Rust 1.70.

---

## OnceLock

Std.

Ưu tiên nếu đủ.

---

## LazyLock

Std.

Ưu tiên.

---

# Memory

## bumpalo

Arena allocator.

---

## typed-arena

Arena.

---

# Embedded

## heapless

Không heap allocation.

---

# Math

## nalgebra

Linear algebra.

---

# SIMD

## portable-simd

Khi stable phù hợp.

---

# HTTP

## tower

Middleware.

---

## tower-http

HTTP middleware.

---

# Recommended Default Stack

CLI

```
clap
anyhow
tracing
```

---

Server

```
tokio
axum
tower
tracing
serde
thiserror
bytes
```

---

Library

```
thiserror
serde
```

---

Protocol

```
bytes
smallvec
thiserror
serde
blake3
```

---

Networking

```
tokio
quinn
bytes
rustls
```

---

Desktop

```
tauri
tokio
serde
tracing
```

---

# Crates to Think Twice About

Không phải cấm.

Nhưng hãy cân nhắc.

---

## lazy_static

Ưu tiên

```
OnceLock

LazyLock
```

---

## failure

Deprecated.

---

## openssl

Nếu không bắt buộc.

Ưu tiên rustls.

---

## unsafe-heavy crate

Đọc source.

Đọc issue.

---

# Anti-pattern

Không thêm crate chỉ để

```
10 dòng code
```

---

Không thêm crate

đã abandon.

---

Không thêm nhiều crate

chỉ để tiện.

---

# Dependency Quality Checklist

Trước khi thêm

□ Maintenance tốt?

□ License phù hợp?

□ MSRV?

□ Documentation?

□ Benchmark?

□ Unsafe nhiều không?

□ Có audit?

□ Có alternative trong std?

□ Có alternative đã dùng trong project?

---

# Crate Upgrade

Đừng update mù.

Đọc

CHANGELOG.

---

# Cargo.lock

Application

↓

Commit.

---

Library

↓

Thường không commit.

---

# Cargo Features

Không bật

```
full
```

nếu không cần.

Giảm compile time.

---

# Cargo Tree

Thường xuyên chạy

```bash
cargo tree
```

để xem dependency phình to.

---

# Duplicate Dependencies

Kiểm tra

```
cargo tree -d
```

---

# Security

Thường xuyên chạy

```
cargo audit
cargo deny
```

---

# AI Rules

Nếu AI thêm crate mới

AI phải giải thích

- Vì sao cần?
- Vì sao không dùng std?
- Có crate nào đã tồn tại trong project?
- Trade-off là gì?
- Có unsafe không?
- Có mature không?

Không thêm dependency chỉ vì "tiện".

---

# Golden Rule

Rust ecosystem rất mạnh.

Đừng vẽ lại bánh xe.

Nhưng cũng đừng lắp thêm 10 cái bánh xe khi std đã đủ.

---

# End of Part 9

Một Rustacean giỏi không phải là người nhớ nhiều crate nhất.

Mà là người biết:

- Khi nào dùng std.
- Khi nào dùng crate.
- Khi nào không nên thêm dependency.
- Khi nào tự triển khai vì yêu cầu đặc biệt.

---

# Part 10 - AI Coding Rules, Review Checklist & Rust Engineering Standards

Đây là bộ quy tắc dành cho AI (Claude Code, Codex, GPT...), cũng như cho mọi lập trình viên tham gia dự án Rust.

Mục tiêu không phải là **code chạy được**.

Mục tiêu là:

- Idiomatic
- Safe
- Performant
- Maintainable
- Reviewable
- Predictable

Compiler là người đồng đội.

Đừng chống lại compiler.

---

# Core Philosophy

Rust không phải C++ có borrow checker.

Rust là một ngôn ngữ khác hoàn toàn.

Nếu tư duy vẫn là

```
OOP

↓

mutable state

↓

shared state

↓

inheritance
```

thì code sẽ luôn "có mùi".

---

# Golden Rules

Luôn ưu tiên

```
Correctness

↓

Safety

↓

Readability

↓

Maintainability

↓

Performance

↓

Micro optimization
```

---

# AI Must Think Before Coding

Trước khi sinh code

AI phải tự hỏi

□ Có API std không?

□ Có iterator không?

□ Có Option API không?

□ Có Result API không?

□ Có ownership đơn giản hơn không?

□ Có cần async không?

□ Có clone không?

□ Có allocation không?

□ Có crate mature không?

---

# Ownership Rules

Ưu tiên

```
Borrow

↓

Move

↓

Arc

↓

Clone
```

Không làm ngược lại.

---

# Clone Rules

Không clone chỉ để compiler hết lỗi.

Sai

```rust
value.clone()
```

nếu borrow đủ.

---

Clone chỉ khi

- ownership thật sự chuyển
- dữ liệu nhỏ
- benchmark chứng minh hợp lý

---

# Reference First

Nếu chỉ đọc

↓

Dùng

```
&T

&str

&Path

&[u8]

&[T]
```

---

# String Rules

Input

↓

```
&str
```

Output

↓

```
String
```

nếu cần ownership.

---

# Collection Rules

Input

↓

Slice

Iterator

AsRef

Không

```
Vec
```

nếu không cần ownership.

---

# Enum First

Không

```rust
pub const ADMIN = 1;
pub const USER = 2;
pub const GUEST = 3;
```

Đúng

```rust
enum Role {
    Admin,
    User,
    Guest,
}
```

---

# Newtype First

Không

```
String

String

String
```

khắp project.

Đúng

```rust
struct UserId(Uuid);

struct PeerId(Uuid);

struct DeviceId(Uuid);
```

Type system phải phân biệt domain.

---

# Type Safety

Đừng dùng

```
u64

String

usize
```

cho mọi thứ.

Tạo type nếu có ý nghĩa domain.

---

# Match First

Nếu nhiều nhánh

↓

match.

Không

```
if

else if

else if

else if
```

---

# Iterator First

Không

```
for

push

flag

counter
```

nếu Iterator giải quyết được.

---

# Standard Library First

Thứ tự

```
std

↓

crate

↓

custom implementation
```

---

# Error Handling

Library

↓

thiserror

Application

↓

anyhow

---

Không

```
panic!

unwrap()

expect()
```

trong production.

---

# Async Rules

Không async nếu không có IO.

Không giữ Mutex qua await.

Không spawn vô tội vạ.

Không unbounded channel mặc định.

---

# Synchronization Rules

Ưu tiên

```
Ownership

↓

Channel

↓

Arc

↓

Atomic

↓

Mutex

↓

RwLock
```

---

# Shared State Rules

Shared mutable state

↓

Last resort.

---

# Performance Rules

Không optimize bằng cảm giác.

Benchmark trước.

Profile sau.

---

# Unsafe Rules

Không dùng unsafe để sửa borrow checker.

Không transmute khi chưa hiểu.

Không unsafe impl Send.

Không unsafe nếu std đủ.

---

# Crate Rules

Không thêm dependency nếu std đủ.

Không thêm dependency chỉ vì

```
10 dòng code
```

---

# Naming Rules

Struct

↓

Danh từ.

---

Function

↓

Động từ.

---

Trait

↓

Capability.

---

Module

↓

Domain.

---

Không

```
utils

helpers

common

manager

processor

misc
```

---

# API Rules

Input rộng.

Output cụ thể.

---

Nhận

```
AsRef

IntoIterator

Into
```

Trả

```
Slice

Concrete Type

Iterator
```

---

# Public API

Mặc định

```
private
```

Public là contract.

---

# Documentation Rules

Mọi public API

↓

rustdoc.

---

Unsafe

↓

SAFETY comment.

---

Ví dụ

↓

Code compile.

---

# Testing Rules

Mỗi bug

↓

Regression test.

---

Parser

↓

Property test.

---

Unsafe

↓

Miri.

---

Protocol

↓

Fuzz.

---

Performance

↓

Criterion.

---

# Logging Rules

Không

```
println!
```

Production.

Ưu tiên

```
tracing
```

---

# Review Rules

Reviewer phải hỏi

---

## Ownership

□ Clone có cần không?

□ Borrow được không?

□ Lifetime đơn giản chưa?

---

## Async

□ Mutex qua await?

□ Cancellation?

□ Timeout?

□ JoinHandle?

□ Backpressure?

---

## Performance

□ Allocation?

□ Copy?

□ Zero-copy?

□ Data structure đúng?

---

## API

□ Input quá cụ thể?

□ Public quá nhiều?

□ Có Builder không?

---

## Errors

□ Có panic?

□ Error rõ ràng?

□ Có context?

---

## Unsafe

□ Có cần unsafe?

□ Có SAFETY comment?

□ Có Miri?

---

## Testing

□ Unit test?

□ Integration test?

□ Regression test?

□ Benchmark?

---

## Architecture

□ Domain rõ?

□ Có utils?

□ Có dependency vòng?

---

# AI Must Never Generate

Không được sinh

```
unwrap()
```

để "cho nhanh".

---

Không được

```
clone()
```

liên tục.

---

Không được

```
Arc<Mutex<_>>
```

làm giải pháp mặc định.

---

Không được

```
Box::new()
```

chỉ để compiler hết lỗi.

---

Không được

```
unsafe
```

nếu không giải thích.

---

Không được

```
tokio::spawn()
```

không giữ JoinHandle.

---

Không được

```
sleep()
```

để đồng bộ task.

---

Không được

```
Vec<String>
```

nếu

```
&str
```

đủ.

---

Không được

```
HashMap
```

nếu chỉ có vài key cố định.

---

Không được

```
String
```

cho mọi ID.

---

Không được

```
pub
```

mọi field.

---

Không được

```
Manager

Helper

Processor

Common

Utils
```

làm tên type/module.

---

# AI Should Prefer

Ưu tiên

```
Enum

Newtype

Iterator

Pattern Matching

Result API

Option API

Slices

Borrowing

Composition

Builder Pattern

Structured Concurrency

Zero-copy

Battle-tested Crates
```

---

# Cargo Commands

Nên chạy trước khi commit

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
cargo doc
```

Nếu project có

```bash
cargo bench
cargo miri test
cargo fuzz run
cargo audit
cargo deny check
cargo nextest run
```

---

# Clippy Recommendation

Trong `lib.rs`

```rust
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
```

Sau đó chọn lọc thêm

```rust
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_must_use)]
#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]
```

Không bật toàn bộ `restriction`.

Chỉ chọn lint thật sự có ích.

---

# Rust Project Checklist

Trước khi release

□ rustfmt sạch

□ clippy sạch

□ test pass

□ benchmark ổn định

□ fuzz pass

□ Miri pass

□ audit pass

□ deny pass

□ docs cập nhật

□ changelog cập nhật

□ MSRV đúng

---

# Engineering Principles

Luôn viết code sao cho

- Dễ đọc hơn thông minh.
- Dễ thay đổi hơn ngắn.
- Type system diễn tả domain.
- Compiler bắt lỗi thay con người.
- Ownership rõ ràng.
- API khó dùng sai.
- Allocation tối thiểu.
- Clone là ngoại lệ.
- Unsafe là ngoại lệ.
- Async là ngoại lệ.
- Shared mutable state là ngoại lệ.

---

# The Rust Mindset

Đừng hỏi

> "Làm sao để compiler hết báo lỗi?"

Hãy hỏi

> "Tại sao compiler cho rằng thiết kế của mình chưa đúng?"

Phần lớn thời gian, compiler đúng.

---

# Final Decision Tree

```
Có std API?

↓

YES

↓

Dùng std

↓

NO

↓

Có crate mature?

↓

YES

↓

Dùng crate

↓

NO

↓

Có cần ownership?

↓

NO

↓

Borrow

↓

YES

↓

Move

↓

Clone là lựa chọn cuối

↓

Có IO?

↓

NO

↓

Sync

↓

YES

↓

Async

↓

Có shared state?

↓

NO

↓

Ownership

↓

YES

↓

Channel

↓

Atomic

↓

Mutex

↓

RwLock

↓

Có cần unsafe?

↓

NO

↓

Safe Rust

↓

YES

↓

Unsafe nhỏ nhất

↓

Document

↓

Test

↓

Fuzz

↓

Miri

↓

Review
```

---

# Final Words

Rust không thưởng cho người viết code ngắn nhất.

Rust thưởng cho người thiết kế API tốt nhất.

Một Rustacean giỏi không phải là người dùng nhiều `unsafe`, nhiều generic hay nhiều macro.

Mà là người biết tận dụng:

- Ownership.
- Borrowing.
- Type System.
- Pattern Matching.
- Iterator.
- Standard Library.
- Ecosystem.
- Compiler.

Để tạo ra phần mềm:

- An toàn.
- Dễ đọc.
- Dễ bảo trì.
- Dễ mở rộng.
- Hiệu năng cao.
- Và khó viết sai.

---

# End

> "Make illegal states unrepresentable."

Nếu type system có thể ngăn bug xảy ra,
đừng để runtime làm việc đó.

**Happy Rusting! 🦀**
