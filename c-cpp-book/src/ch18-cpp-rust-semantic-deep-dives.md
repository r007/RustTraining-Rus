## Семантические различия C++ → Rust: углублённый разбор

> **Что вы узнаете:** подробное соответствие концепций C++, у которых нет очевидного аналога в Rust, — четыре именованных приведения, SFINAE и ограничения трейтами, CRTP и ассоциированные типы и другие частые трудности при переводе.

Ниже описаны концепции C++, для которых нет очевидного взаимно однозначного
соответствия в Rust. Эти различия часто мешают программистам C++ при переводе кода.

### Иерархия приведений: четыре приведения C++ → эквиваленты в Rust

В C++ есть четыре именованных приведения. Rust заменяет их другими, более явными механизмами:

```cpp
// Иерархия приведений C++
int i = static_cast<int>(3.14);            // 1. Числовое / приведение вверх
Derived* d = dynamic_cast<Derived*>(base); // 2. Приведение вниз во время выполнения
int* p = const_cast<int*>(cp);              // 3. Снятие const
auto* raw = reinterpret_cast<char*>(&obj); // 4. Переинтерпретация на уровне битов
```

| Приведение C++ | Эквивалент в Rust | Безопасность | Примечания |
|----------|----------------|--------|-------|
| `static_cast` (числовое) | Ключевое слово `as` | Безопасно, но может обрезать/переполнить | `let i = 3.14_f64 as i32;` — обрезает до 3 |
| `static_cast` (числовое, проверяемое) | `From`/`Into` | Безопасно, проверяется на этапе компиляции | `let i: i32 = 42_u8.into();` — только расширение |
| `static_cast` (числовое, с возможной ошибкой) | `TryFrom`/`TryInto` | Безопасно, возвращает `Result` | `let i: u8 = 300_u16.try_into()?;` — вернёт Err |
| `dynamic_cast` (приведение вниз) | `match` по перечислению / `Any::downcast_ref` | Безопасно | Сопоставление с образцом для перечислений; `Any` для трейт-объектов |
| `const_cast` | Эквивалента нет | | В безопасном Rust нет способа превратить `&` в `&mut`. Для внутренней изменяемости используйте `Cell`/`RefCell` |
| `reinterpret_cast` | `std::mem::transmute` | **`unsafe`** | Переинтерпретирует битовый шаблон. Почти всегда ошибочно — предпочитайте `from_le_bytes()` и т. п. |

```rust
// Эквиваленты в Rust:

// 1. Числовые приведения — предпочитайте From/Into вместо `as`
let widened: u32 = 42_u8.into();             // Расширение без ошибок — всегда предпочтительно
let truncated = 300_u16 as u8;                // ⚠ Переполняется до 44! Тихая потеря данных
let checked: Result<u8, _> = 300_u16.try_into(); // Err — безопасное преобразование с возможной ошибкой

// 2. Приведение вниз: перечисление (предпочтительно) или Any (когда нужно стирание типа)
use std::any::Any;

fn handle_any(val: &dyn Any) {
    if let Some(s) = val.downcast_ref::<String>() {
        println!("Got string: {s}");
    } else if let Some(n) = val.downcast_ref::<i32>() {
        println!("Got int: {n}");
    }
}

// 3. «const_cast» → внутренняя изменяемость (unsafe не нужен)
use std::cell::Cell;
struct Sensor {
    read_count: Cell<u32>,  // Изменяем через &self
}
impl Sensor {
    fn read(&self) -> f64 {
        self.read_count.set(self.read_count.get() + 1); // &self, а не &mut self
        42.0
    }
}

// 4. reinterpret_cast → transmute (почти никогда не нужно)
// Предпочитайте безопасные альтернативы:
let bytes: [u8; 4] = 0x12345678_u32.to_ne_bytes();  // ✅ Безопасно
let val = u32::from_ne_bytes(bytes);                   // ✅ Безопасно
// unsafe { std::mem::transmute::<u32, [u8; 4]>(val) } // ❌ Избегайте
```

> **Правило**: в идиоматичном Rust `as` должен встречаться редко (для расширения используйте `From`/`Into`, для сужения — `TryFrom`/`TryInto`), `transmute` — исключительный случай, а у `const_cast` нет эквивалента, потому что типы с внутренней изменяемостью делают его ненужным.

---

### Препроцессор → `cfg`, флаги функций и `macro_rules!`

C++ сильно опирается на препроцессор для условной компиляции, констант и генерации кода. Rust заменяет всё это полноценными возможностями языка.

#### `#define`-константы → `const` или `const fn`

```cpp
// C++
#define MAX_RETRIES 5
#define BUFFER_SIZE (1024 * 64)
#define SQUARE(x) ((x) * (x))  // Макрос — текстовая подстановка, без проверки типов
```

```rust
// Rust — с проверкой типов, с областью видимости, без текстовой подстановки
const MAX_RETRIES: u32 = 5;
const BUFFER_SIZE: usize = 1024 * 64;
const fn square(x: u32) -> u32 { x * x }  // Вычисляется на этапе компиляции

// Можно использовать в константных контекстах:
const AREA: u32 = square(12);  // Вычисляется на этапе компиляции
static BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
```

#### `#ifdef` / `#if` → `#[cfg()]` и `cfg!()`

```cpp
// C++
#ifdef DEBUG
    log_verbose("Step 1 complete");
#endif

#if defined(LINUX) && !defined(ARM)
    use_x86_path();
#else
    use_generic_path();
#endif
```

```rust
// Rust — условная компиляция на основе атрибутов
#[cfg(debug_assertions)]
fn log_verbose(msg: &str) { eprintln!("[VERBOSE] {msg}"); }

#[cfg(not(debug_assertions))]
fn log_verbose(_msg: &str) { /* удаляется в релизной сборке */ }

// Комбинирование условий:
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn use_x86_path() { /* ... */ }

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn use_generic_path() { /* ... */ }

// Проверка во время выполнения (условие всё равно вычисляется на этапе компиляции, но его можно использовать в выражениях):
if cfg!(target_os = "windows") {
    println!("Running on Windows");
}
```

#### Флаги функций в `Cargo.toml`

```toml
# Cargo.toml — замена #ifdef FEATURE_FOO
[features]
default = ["json"]
json = ["dep:serde_json"]       # Необязательная зависимость
verbose-logging = []            # Флаг без дополнительной зависимости
gpu-support = ["dep:cuda-sys"]  # Необязательная поддержка GPU
```

```rust
// Условный код на основе флагов функций:
#[cfg(feature = "json")]
pub fn parse_config(data: &str) -> Result<Config, Error> {
    serde_json::from_str(data).map_err(Error::from)
}

#[cfg(feature = "verbose-logging")]
macro_rules! verbose {
    ($($arg:tt)*) => { eprintln!("[VERBOSE] {}", format!($($arg)*)); }
}
#[cfg(not(feature = "verbose-logging"))]
macro_rules! verbose {
    ($($arg:tt)*) => { }; // Компилируется в ничего
}
```

#### `#define MACRO(x)` → `macro_rules!`

```cpp
// C++ — текстовая подстановка, печально известная своей подверженностью ошибкам
#define DIAG_CHECK(cond, msg) \
    do { if (!(cond)) { log_error(msg); return false; } } while(0)
```

```rust
// Rust — гигиеничный, с проверкой типов, работает с синтаксическим деревом
macro_rules! diag_check {
    ($cond:expr, $msg:expr) => {
        if !($cond) {
            log_error($msg);
            return Err(DiagError::CheckFailed($msg.to_string()));
        }
    };
}

fn run_test() -> Result<(), DiagError> {
    diag_check!(temperature < 85.0, "GPU too hot");
    diag_check!(voltage > 0.8, "Rail voltage too low");
    Ok(())
}
```

| Препроцессор C++ | Эквивалент в Rust | Преимущество |
|-----------------|----------------|-----------|
| `#define PI 3.14` | `const PI: f64 = 3.14;` | С типом, с областью видимости, видна отладчику |
| `#define MAX(a,b) ((a)>(b)?(a):(b))` | `macro_rules!` или обобщённая `fn max<T: Ord>` | Нет ошибок двойного вычисления |
| `#ifdef DEBUG` | `#[cfg(debug_assertions)]` | Проверяется компилятором, нет риска опечатки |
| `#ifdef FEATURE_X` | `#[cfg(feature = "x")]` | Cargo управляет флагами; учитывает зависимости |
| `#include "header.h"` | `mod module;` + `use module::Item;` | Нет защиты от повторного включения, нет циклических включений |
| `#pragma once` | Не нужно | Каждый файл `.rs` — модуль, подключается ровно один раз |

---

### Заголовочные файлы и `#include` → модули и `use`

В C++ модель компиляции основана на текстовом включении:

```cpp
// widget.h — каждая единица трансляции, использующая Widget, включает этот файл
#pragma once
#include <string>
#include <vector>

class Widget {
public:
    Widget(std::string name);
    void activate();
private:
    std::string name_;
    std::vector<int> data_;
};
```

```cpp
// widget.cpp — отдельное определение
#include "widget.h"
Widget::Widget(std::string name) : name_(std::move(name)) {}
void Widget::activate() { /* ... */ }
```

В Rust **нет заголовочных файлов, предварительных объявлений и защиты от повторного включения**:

```rust
// src/widget.rs — объявление И определение в одном файле
pub struct Widget {
    name: String,         // По умолчанию приватное
    data: Vec<i32>,
}

impl Widget {
    pub fn new(name: String) -> Self {
        Widget { name, data: Vec::new() }
    }
    pub fn activate(&self) { /* ... */ }
}
```

```rust
// src/main.rs — импорт по пути модуля
mod widget;  // Сообщает компилятору, что нужно включить src/widget.rs
use widget::Widget;

fn main() {
    let w = Widget::new("sensor".to_string());
    w.activate();
}
```

| C++ | Rust | Почему лучше |
|-----|------|-----------------|
| `#include "foo.h"` | `mod foo;` в родителе + `use foo::Item;` | Нет текстового включения, нет нарушений ODR |
| `#pragma once` / защитные макросы | Не нужно | Каждый файл `.rs` — модуль, компилируется один раз |
| Предварительные объявления | Не нужны | Компилятор видит весь крейт; порядок не важен |
| `class Foo;` (неполный тип) | Не нужно | Нет разделения на объявление и определение |
| `.h` + `.cpp` для каждого класса | Один файл `.rs` | Нет ошибок рассогласования объявления и определения |
| `using namespace std;` | `use std::collections::HashMap;` | Всегда явно — нет загрязнения глобального пространства имён |
| Вложенные `namespace a::b` | Вложенные `mod a { mod b { } }` или `a/b.rs` | Структура файловой системы повторяет дерево модулей |

---

### `friend` и контроль доступа → видимость модулей

В C++ `friend` даёт определённым классам или функциям доступ к приватным членам.
В Rust нет ключевого слова `friend` — вместо этого **приватность ограничена модулем**:

```cpp
// C++
class Engine {
    friend class Car;   // Car может обращаться к приватным членам
    int rpm_;
    void set_rpm(int r) { rpm_ = r; }
public:
    int rpm() const { return rpm_; }
};
```

```rust
// Rust — элементы одного модуля могут обращаться ко всем полям, `friend` не нужен
mod vehicle {
    pub struct Engine {
        rpm: u32,  // Приватно для модуля (а не для структуры!)
    }

    impl Engine {
        pub fn new() -> Self { Engine { rpm: 0 } }
        pub fn rpm(&self) -> u32 { self.rpm }
    }

    pub struct Car {
        engine: Engine,
    }

    impl Car {
        pub fn new() -> Self { Car { engine: Engine::new() } }
        pub fn accelerate(&mut self) {
            self.engine.rpm = 3000; // ✅ Тот же модуль — прямой доступ к полю
        }
        pub fn rpm(&self) -> u32 {
            self.engine.rpm  // ✅ Тот же модуль — можно читать приватное поле
        }
    }
}

fn main() {
    let mut car = vehicle::Car::new();
    car.accelerate();
    // car.engine.rpm = 9000;  // ❌ Ошибка компиляции: `engine` приватно
    println!("RPM: {}", car.rpm()); // ✅ Публичный метод Car
}
```

| Доступ в C++ | Эквивалент в Rust | Область действия |
|-----------|----------------|-------|
| `private` | (по умолчанию, без ключевого слова) | Доступно только внутри того же модуля |
| `protected` | Прямого аналога нет | Используйте `pub(super)` для доступа из родительского модуля |
| `public` | `pub` | Доступно везде |
| `friend class Foo` | Поместить `Foo` в тот же модуль | Приватность на уровне модуля заменяет friend |
| — | `pub(crate)` | Видно внутри крейта, но не внешним зависимостям |
| — | `pub(super)` | Видно только родительскому модулю |
| — | `pub(in crate::path)` | Видно внутри конкретного поддерева модулей |

> **Ключевая мысль**: приватность в C++ задаётся на уровне класса. Приватность в Rust — на уровне модуля. Это значит, что вы управляете доступом, выбирая, какие типы находятся в одном модуле — размещённые вместе типы имеют полный доступ к приватным полям друг друга.

---

### `volatile` → атомарные операции и `read_volatile`/`write_volatile`

В C++ `volatile` говорит компилятору не оптимизировать чтения и записи — обычно это используется для регистров аппаратуры, отображённых в память. **В Rust нет ключевого слова `volatile`.**

```cpp
// C++: volatile для аппаратных регистров
volatile uint32_t* const GPIO_REG = reinterpret_cast<volatile uint32_t*>(0x4002'0000);
*GPIO_REG = 0x01;              // Запись не оптимизируется
uint32_t val = *GPIO_REG;     // Чтение не оптимизируется
```

```rust
// Rust: явные volatile-операции — только в unsafe-коде
use std::ptr;

const GPIO_REG: *mut u32 = 0x4002_0000 as *mut u32;

// SAFETY: GPIO_REG — валидный адрес ввода-вывода, отображённый в память.
unsafe {
    ptr::write_volatile(GPIO_REG, 0x01);   // Запись не оптимизируется
    let val = ptr::read_volatile(GPIO_REG); // Чтение не оптимизируется
}
```

Для **совместно используемого состояния в конкурентном коде** (другое частое применение `volatile` в C++) Rust использует атомарные типы:

```cpp
// C++: volatile НЕДОСТАТОЧЕН для потокобезопасности (распространённая ошибка!)
volatile bool stop_flag = false;  // ❌ Гонка данных — неопределённое поведение в C++11+

// Правильно в C++:
std::atomic<bool> stop_flag{false};
```

```rust
// Rust: атомарные типы — единственный способ разделять изменяемое состояние между потоками
use std::sync::atomic::{AtomicBool, Ordering};

static STOP_FLAG: AtomicBool = AtomicBool::new(false);

// Из другого потока:
STOP_FLAG.store(true, Ordering::Release);

// Проверка:
if STOP_FLAG.load(Ordering::Acquire) {
    println!("Stopping");
}
```

| Применение в C++ | Эквивалент в Rust | Примечания |
|-----------|----------------|-------|
| `volatile` для аппаратных регистров | `ptr::read_volatile` / `ptr::write_volatile` | Требует `unsafe` — корректно для MMIO |
| `volatile` для сигнализации между потоками | `AtomicBool` / `AtomicU32` и т. д. | `volatile` в C++ для этого тоже неверен! |
| `std::atomic<T>` | `std::sync::atomic::AtomicT` | Та же семантика, те же порядки доступа |
| `std::atomic<T>::load(memory_order_acquire)` | `AtomicT::load(Ordering::Acquire)` | Соответствие 1:1 |

---

### Переменные `static` → `static`, `const`, `LazyLock`, `OnceLock`

#### Базовые `static` и `const`

```cpp
// C++
const int MAX_RETRIES = 5;                    // Константа времени компиляции
static std::string CONFIG_PATH = "/etc/app";  // Статическая инициализация — порядок не определён!
```

```rust
// Rust
const MAX_RETRIES: u32 = 5;                   // Константа времени компиляции, встраивается
static CONFIG_PATH: &str = "/etc/app";         // Время жизни 'static, фиксированный адрес
```

#### Печально известная проблема порядка статической инициализации

В C++ есть хорошо известная проблема: глобальные конструкторы в разных единицах трансляции выполняются в **неопределённом порядке**. Rust полностью обходит её — значения `static` должны быть константами времени компиляции (без конструкторов).

Для глобальных переменных, инициализируемых во время выполнения, используйте `LazyLock` (Rust 1.80+) или `OnceLock`:

```rust
use std::sync::LazyLock;

// Аналог C++ `static std::regex` — инициализируется при первом обращении, потокобезопасно
static CONFIG_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[a-z]+_diag$").expect("invalid regex")
});

fn is_valid_diag(name: &str) -> bool {
    CONFIG_REGEX.is_match(name)  // Первый вызов инициализирует; последующие работают быстро
}
```

```rust
use std::sync::OnceLock;

// OnceLock: инициализируется один раз, может быть задан из данных времени выполнения
static DB_CONN: OnceLock<String> = OnceLock::new();

fn init_db(connection_string: &str) {
    DB_CONN.set(connection_string.to_string())
        .expect("DB_CONN already initialized");
}

fn get_db() -> &'static str {
    DB_CONN.get().expect("DB not initialized")
}
```

| C++ | Rust | Примечания |
|-----|------|-------|
| `const int X = 5;` | `const X: i32 = 5;` | Оба — на этапе компиляции. В Rust нужна аннотация типа |
| `constexpr int X = 5;` | `const X: i32 = 5;` | `const` в Rust всегда constexpr |
| `static int count = 0;` (на уровне файла) | `static COUNT: AtomicI32 = AtomicI32::new(0);` | Изменяемые статики требуют `unsafe` или атомарных типов |
| `static std::string s = "hi";` | `static S: &str = "hi";` или `LazyLock<String>` | Для простых случаев нет конструктора времени выполнения |
| `static MyObj obj;` (сложная инициализация) | `static OBJ: LazyLock<MyObj> = LazyLock::new(\|\| { ... });` | Потокобезопасно, ленивая инициализация, нет проблем с порядком |
| `thread_local` | `thread_local! { static X: Cell<u32> = Cell::new(0); }` | Та же семантика |

---

### `constexpr` → `const fn`

`constexpr` в C++ помечает функции и переменные для вычисления на этапе компиляции. Rust
использует `const fn` и `const` для той же цели:

```cpp
// C++
constexpr int factorial(int n) {
    return n <= 1 ? 1 : n * factorial(n - 1);
}
constexpr int val = factorial(5);  // Вычисляется на этапе компиляции → 120
```

```rust
// Rust
const fn factorial(n: u32) -> u32 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
const VAL: u32 = factorial(5);  // Вычисляется на этапе компиляции → 120

// Также работает в размерах массивов и в шаблонах match:
const LOOKUP: [u32; 5] = [factorial(1), factorial(2), factorial(3),
                           factorial(4), factorial(5)];
```

| C++ | Rust | Примечания |
|-----|------|-------|
| `constexpr int f()` | `const fn f() -> i32` | Та же цель — вычисляемая на этапе компиляции |
| Переменная `constexpr` | Переменная `const` | `const` в Rust всегда вычисляется на этапе компиляции |
| `consteval` (C++20) | Эквивалента нет | `const fn` может выполняться и во время выполнения |
| `if constexpr` (C++17) | Эквивалента нет (используйте `cfg!` или обобщения) | Специализация трейтов покрывает часть случаев |
| `constinit` (C++20) | `static` с константным инициализатором | `static` в Rust по умолчанию должен инициализироваться константой |

> **Текущие ограничения `const fn`** (стабилизировано в Rust 1.82):
> - Нельзя вызывать методы трейтов (нельзя вызвать `.len()` у `Vec` в константном контексте)
> - Нет выделения памяти в куче (`Box::new`, `Vec::new` не являются `const`)
> - ~~Нет арифметики с плавающей точкой~~ — **стабилизирована в Rust 1.82**
> - Нельзя использовать циклы `for` (используйте рекурсию или `while` с ручным индексом)

---

### SFINAE и `enable_if` → ограничения трейтами и предложения `where`

В C++ SFINAE (Substitution Failure Is Not An Error, «неудача подстановки — не ошибка») — механизм
условного обобщённого программирования. Он мощен, но печально известен нечитаемостью. Rust
полностью заменяет его **ограничениями трейтами** (trait bounds):

```cpp
// C++: условная функция на основе SFINAE (до C++20)
template<typename T,
         std::enable_if_t<std::is_integral_v<T>, int> = 0>
T double_it(T val) { return val * 2; }

template<typename T,
         std::enable_if_t<std::is_floating_point_v<T>, int> = 0>
T double_it(T val) { return val * 2.0; }

// Концепты C++20 — чище, но всё ещё многословны:
template<std::integral T>
T double_it(T val) { return val * 2; }
```

```rust
// Rust: ограничения трейтами — читаемо, компонуемо, отличные сообщения об ошибках
use std::ops::Mul;

fn double_it<T: Mul<Output = T> + From<u8>>(val: T) -> T {
    val * T::from(2)
}

// Или с предложением where для сложных ограничений:
fn process<T>(val: T) -> String
where
    T: std::fmt::Display + Clone + Send,
{
    format!("Processing: {}", val)
}

// Условное поведение через отдельные реализации (замена перегрузок на основе SFINAE):
trait Describable {
    fn describe(&self) -> String;
}

impl Describable for u32 {
    fn describe(&self) -> String { format!("integer: {self}") }
}

impl Describable for f64 {
    fn describe(&self) -> String { format!("float: {self:.2}") }
}
```

| Шаблонное метапрограммирование C++ | Эквивалент в Rust | Читаемость |
|-----------------------------|----------------|-------------|
| `std::enable_if_t<cond>` | `where T: Trait` | 🟢 Понятно, как обычный английский |
| `std::is_integral_v<T>` | Ограничение числовым трейтом или конкретными типами | 🟢 Без суффиксов `_v` / `_t` |
| Наборы перегрузок SFINAE | Отдельные блоки `impl Trait for ConcreteType` | 🟢 Каждая реализация самостоятельна |
| `if constexpr (std::is_same_v<T, int>)` | Специализация через реализации трейтов | 🟢 Диспетчеризация на этапе компиляции |
| Концепт C++20 | `trait` | 🟢 Почти одинаковый смысл |
| Клауза `requires` | Предложение `where` | 🟢 Та же позиция, похожий синтаксис |
| Компиляция падает глубоко внутри шаблона | Компиляция падает в месте вызова с несоответствием трейта | 🟢 Без каскадов ошибок на 200 строк |

> **Ключевая мысль**: концепты C++20 — самое близкое к трейтам Rust. Если вы знакомы с концептами C++20, думайте о трейтах Rust как о концептах, которые являются полноценной возможностью языка с версии 1.0 и имеют согласованную модель реализации (реализации трейтов), а не утиной типизации.

---

### `std::function` → указатели на функции, `impl Fn` и `Box<dyn Fn>`

`std::function<R(Args...)>` в C++ — это вызываемый объект со стёртым типом. В Rust есть три варианта
с разными компромиссами:

```cpp
// C++: универсальное решение (в куче, со стёртым типом)
#include <functional>
std::function<int(int)> make_adder(int n) {
    return [n](int x) { return x + n; };
}
```

```rust
// Вариант 1 в Rust: указатель на функцию — просто, без захватов, без выделения памяти
fn add_one(x: i32) -> i32 { x + 1 }
let f: fn(i32) -> i32 = add_one;
println!("{}", f(5)); // 6

// Вариант 2 в Rust: impl Fn — мономорфизация, без накладных расходов, может захватывать
fn apply(val: i32, f: impl Fn(i32) -> i32) -> i32 { f(val) }
let n = 10;
let result = apply(5, |x| x + n);  // Замыкание захватывает `n`

// Вариант 3 в Rust: Box<dyn Fn> — стирание типа, выделение в куче (как std::function)
fn make_adder(n: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x + n)
}
let adder = make_adder(10);
println!("{}", adder(5));  // 15

// Хранение разнородных вызываемых объектов (как vector<function<int(int)>>):
let callbacks: Vec<Box<dyn Fn(i32) -> i32>> = vec![
    Box::new(|x| x + 1),
    Box::new(|x| x * 2),
    Box::new(make_adder(100)),
];
for cb in &callbacks {
    println!("{}", cb(5));  // 6, 10, 105
}
```

| Когда использовать | Аналог в C++ | Выбор в Rust |
|------------|---------------|-------------|
| Функция верхнего уровня без захватов | Указатель на функцию | `fn(Args) -> Ret` |
| Обобщённая функция, принимающая вызываемые объекты | Параметр шаблона | `impl Fn(Args) -> Ret` (статическая диспетчеризация) |
| Ограничение трейтом в обобщениях | `template<typename F>` | `F: Fn(Args) -> Ret` |
| Хранимый вызываемый объект со стёртым типом | `std::function<R(Args)>` | `Box<dyn Fn(Args) -> Ret>` |
| Обратный вызов, изменяющий состояние | `std::function` с изменяемой лямбдой | `Box<dyn FnMut(Args) -> Ret>` |
| Одноразовый обратный вызов (потребляется) | `std::function` (перемещённый) | `Box<dyn FnOnce(Args) -> Ret>` |

> **Замечание о производительности**: `impl Fn` не имеет накладных расходов (мономорфизация, как шаблон C++).
> `Box<dyn Fn>` имеет те же накладные расходы, что и `std::function` (vtable и выделение в куче).
> Предпочитайте `impl Fn`, если не нужно хранить разнородные вызываемые объекты.

---

### Соответствие контейнеров: STL C++ → `std::collections` Rust

| Контейнер STL C++ | Эквивалент в Rust | Примечания |
|------------------|----------------|-------|
| `std::vector<T>` | `Vec<T>` | Почти идентичный API. Rust проверяет границы по умолчанию |
| `std::array<T, N>` | `[T; N]` | Массив фиксированного размера на стеке |
| `std::deque<T>` | `std::collections::VecDeque<T>` | Кольцевой буфер. Эффективные push/pop с обоих концов |
| `std::list<T>` | `std::collections::LinkedList<T>` | В Rust используется редко — `Vec` почти всегда быстрее |
| `std::forward_list<T>` | Эквивалента нет | Используйте `Vec` или `VecDeque` |
| `std::unordered_map<K, V>` | `std::collections::HashMap<K, V>` | По умолчанию использует `SipHash` (устойчив к DoS) |
| `std::map<K, V>` | `std::collections::BTreeMap<K, V>` | B-дерево; ключи отсортированы; требуется `K: Ord` |
| `std::unordered_set<T>` | `std::collections::HashSet<T>` | Требуется `T: Hash + Eq` |
| `std::set<T>` | `std::collections::BTreeSet<T>` | Отсортированное множество; требуется `T: Ord` |
| `std::priority_queue<T>` | `std::collections::BinaryHeap<T>` | Max-куча по умолчанию (как в C++) |
| `std::stack<T>` | `Vec<T>` с `.push()` / `.pop()` | Отдельный тип стека не нужен |
| `std::queue<T>` | `VecDeque<T>` с `.push_back()` / `.pop_front()` | Отдельный тип очереди не нужен |
| `std::string` | `String` | Гарантированный UTF-8, без завершающего нуля |
| `std::string_view` | `&str` | Заимствованный срез UTF-8 |
| `std::span<T>` (C++20) | `&[T]` / `&mut [T]` | Срезы Rust — полноценный тип с версии 1.0 |
| `std::tuple<A, B, C>` | `(A, B, C)` | Встроенный синтаксис, поддерживает деструктуризацию |
| `std::pair<A, B>` | `(A, B)` | Просто кортеж из двух элементов |
| `std::bitset<N>` | Эквивалента в std нет | Используйте крейт `bitvec` или `[u8; N/8]` |

**Ключевые различия**:
- Для `HashMap`/`HashSet` в Rust требуется `K: Hash + Eq` — компилятор проверяет это на уровне типов, в отличие от C++, где использование ключа без хеширования даёт ошибку шаблона глубоко внутри STL
- Индексация `Vec` (`v[i]`) по умолчанию вызывает panic при выходе за границы. Используйте `.get(i)` для `Option<&T>` или итераторы, чтобы полностью избежать проверок границ
- Нет `std::multimap` и `std::multiset` — используйте `HashMap<K, Vec<V>>` или `BTreeMap<K, Vec<V>>`

---

### Безопасность исключений → безопасность при panic

C++ определяет три уровня безопасности исключений (гарантии Абрахамса):

| Уровень C++ | Значение | Эквивалент в Rust |
|----------|---------|----------------|
| **No-throw** | Функция никогда не выбрасывает исключение | Функция никогда не вызывает panic (возвращает `Result`) |
| **Strong** (фиксация или откат) | Если выброшено исключение, состояние не меняется | Модель владения делает это естественным — если `?` досрочно возвращает управление, частично построенные значения уничтожаются |
| **Basic** | Если выброшено исключение, инварианты сохраняются | Поведение Rust по умолчанию — `Drop` выполняется, утечек нет |

#### Как модель владения помогает

```rust
// Сильная гарантия бесплатно — если file.write() завершится ошибкой, config не меняется
fn update_config(config: &mut Config, path: &str) -> Result<(), Error> {
    let new_data = fetch_from_network()?; // Err → ранний возврат, config не тронут
    let validated = validate(new_data)?;   // Err → ранний возврат, config не тронут
    *config = validated;                   // Выполняется только при успехе (фиксация)
    Ok(())
}
```

В C++ для сильной гарантии нужен ручной откат или идиома copy-and-swap. В Rust
распространение через `?` даёт сильную гарантию по умолчанию для большинства кода.

#### `catch_unwind` — аналог `catch(...)` в Rust

```rust
use std::panic;

// Перехват panic (как catch(...) в C++) — нужно редко
let result = panic::catch_unwind(|| {
    // Код, который может вызвать panic
    let v = vec![1, 2, 3];
    v[10]  // Panic! (выход за границы индекса)
});

match result {
    Ok(val) => println!("Got: {val}"),
    Err(_) => eprintln!("Caught a panic — cleaned up"),
}
```

#### `UnwindSafe` — отметка типов как безопасных при panic

```rust
use std::panic::UnwindSafe;

// Типы за &mut по умолчанию НЕ UnwindSafe — panic мог оставить их
// в частично изменённом состоянии
fn safe_execute<F: FnOnce() + UnwindSafe>(f: F) {
    let _ = std::panic::catch_unwind(f);
}

// Используйте AssertUnwindSafe, чтобы переопределить, когда вы проверили код:
use std::panic::AssertUnwindSafe;
let mut data = vec![1, 2, 3];
let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
    data.push(4);
}));
```

| Шаблон исключений C++ | Эквивалент в Rust |
|-----------------------|-----------------|
| `throw MyException()` | `return Err(MyError::...)` (предпочтительно) или `panic!("...")` |
| `try { } catch (const E& e)` | `match result { Ok(v) => ..., Err(e) => ... }` или `?` |
| `catch (...)` | `std::panic::catch_unwind(...)` |
| `noexcept` | `-> Result<T, E>` (ошибки — это значения, а не исключения) |
| Очистка RAII при раскрутке стека | `Drop::drop()` выполняется при раскрутке panic |
| `std::uncaught_exceptions()` | `std::thread::panicking()` |
| Флаг компиляции `-fno-exceptions` | `panic = "abort"` в секции `[profile]` файла `Cargo.toml` |

> **Главное**: в Rust большинство кода использует `Result<T, E>` вместо исключений, что делает пути ошибок явными и компонуемыми. `panic!` зарезервирован для ошибок в программе (например, при сбое `assert!`), а не для рутинных ошибок. Поэтому «безопасность исключений» во многом перестаёт быть проблемой — система владения автоматически занимается очисткой.

---

## Шаблоны миграции с C++ на Rust

### Краткий справочник: соответствие идиом C++ → Rust

| **Шаблон C++** | **Идиома Rust** | **Примечания** |
|----------------|---------------|----------|
| `class Derived : public Base` | `enum Variant { A {...}, B {...} }` | Для закрытых наборов предпочитайте перечисления |
| `virtual void method() = 0` | `trait MyTrait { fn method(&self); }` | Для открытых/расширяемых интерфейсов |
| `dynamic_cast<Derived*>(ptr)` | `match value { Variant::A(data) => ..., }` | Исчерпывающе, без ошибок во время выполнения |
| `vector<unique_ptr<Base>>` | `Vec<Box<dyn Trait>>` | Только когда действительно нужен полиморфизм |
| `shared_ptr<T>` | `Rc<T>` или `Arc<T>` | Сначала рассмотрите `Box<T>` или владеющие значения |
| `enable_shared_from_this<T>` | Паттерн арены (`Vec<T>` + индексы) | Полностью устраняет циклы ссылок |
| `Base* m_pFramework` в каждом классе | `fn execute(&mut self, ctx: &mut Context)` | Передавайте контекст, не храните указатели |
| `try { } catch (...) { }` | `match result { Ok(v) => ..., Err(e) => ... }` | Или `?` для распространения |
| `std::optional<T>` | `Option<T>` | Требуется `match`, нельзя забыть None |
| Параметр `const std::string&` | Параметр `&str` | Принимает и `String`, и `&str` |
| `enum class Foo { A, B, C }` | `enum Foo { A, B, C }` | Перечисления Rust могут также хранить данные |
| `auto x = std::move(obj)` | `let x = obj;` | Перемещение по умолчанию, `std::move` не нужен |
| CMake + make + линтер | `cargo build / test / clippy / fmt` | Один инструмент для всего |

### Стратегия миграции
1. **Начните с типов данных**: сначала переводите структуры и перечисления — это заставляет думать о владении
2. **Превращайте фабрики в перечисления**: если фабрика создаёт разные производные типы, скорее всего, это должно быть `enum` + `match`
3. **Превращайте божественные объекты в компонуемые структуры**: группируйте связанные поля в сфокусированные структуры
4. **Заменяйте указатели заимствованиями**: переводите хранимые указатели `Base*` в заимствования с ограниченным временем жизни `&'a T`
5. **Используйте `Box<dyn Trait>` экономно**: только для систем плагинов и моков в тестах
6. **Доверьтесь компилятору**: сообщения об ошибках Rust отличны — читайте их внимательно
