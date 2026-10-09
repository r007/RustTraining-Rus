## Избегание избыточного clone()

> **Что вы узнаете:** почему `.clone()` в Rust — это «запах кода», как перестроить владение, чтобы убрать лишние копии, и конкретные паттерны, которые указывают на проблему в дизайне владения.

- Приходя из C++, `.clone()` кажется безопасным значением по умолчанию — «просто скопируй». Но избыточное клонирование скрывает проблемы владения и вредит производительности.
- **Эмпирическое правило**: если вы клонируете, чтобы угодить проверке заимствований, скорее всего, нужно перестроить владение.

### Когда clone() ошибочен

```rust
// ПЛОХО: клонируем String только для того, чтобы передать функции, которая лишь читает его
fn log_message(msg: String) {  // Без нужды забирает владение
    println!("[LOG] {}", msg);
}
let message = String::from("GPU test passed");
log_message(message.clone());  // Расточительно: выделяет новую String целиком
log_message(message);           // Оригинал поглощён — клонирование было бессмысленным
```

```rust
// ХОРОШО: принимаем заимствование — ноль выделений
fn log_message(msg: &str) {    // Заимствует, не владеет
    println!("[LOG] {}", msg);
}
let message = String::from("GPU test passed");
log_message(&message);          // Без клонирования, без выделения памяти
log_message(&message);          // Можно вызвать снова — message не поглощена
```

### Реальный пример: возврат `&str` вместо клонирования
```rust
// Пример: healthcheck.rs — возвращает заимствованное представление, без выделений
pub fn serial_or_unknown(&self) -> &str {
    self.serial.as_deref().unwrap_or(UNKNOWN_VALUE)
}

pub fn model_or_unknown(&self) -> &str {
    self.model.as_deref().unwrap_or(UNKNOWN_VALUE)
}
```
Аналог в C++ вернул бы `const std::string&` или `std::string_view`, но в C++ ни то, ни другое не проверяется по времени жизни. В Rust проверка заимствований гарантирует, что возвращаемый `&str` не переживёт `self`.

### Реальный пример: статические срезы строк — вообще без кучи
```rust
// Пример: healthcheck.rs — таблицы строк, известные на этапе компиляции
const HBM_SCREEN_RECIPES: &[&str] = &[
    "hbm_ds_ntd", "hbm_ds_ntd_gfx", "hbm_dt_ntd", "hbm_dt_ntd_gfx",
    "hbm_burnin_8h", "hbm_burnin_24h",
];
```
В C++ это обычно был бы `std::vector<std::string>` (выделяется в куче при первом использовании). Тип `&'static [&'static str]` в Rust живёт в памяти только для чтения — без накладных расходов во время выполнения.

### Когда clone() действительно уместен

| **Ситуация** | **Почему клонирование допустимо** | **Пример** |
|--------------|--------------------|-----------|
| `Arc::clone()` для потоков | Увеличивает счётчик ссылок (~1 нс), не копирует данные | `let flag = stop_flag.clone();` |
| Перемещение данных в порождённый поток | Поток нуждается в собственной копии | `let ctx = ctx.clone(); thread::spawn(move \|\| { ... })` |
| Извлечение из полей `&self` | Нельзя переместить значение из заимствования | `self.name.clone()`, когда возвращаем владеющий `String` |
| Небольшие типы `Copy` в `Option` | `.copied()` понятнее, чем `.clone()` | `opt.get(0).copied()` для `Option<&u32>` → `Option<u32>` |

### Реальный пример: Arc::clone для совместного использования потоками
```rust
// Пример: workload.rs — Arc::clone дёшев (увеличение счётчика ссылок)
let stop_flag = Arc::new(AtomicBool::new(false));
let stop_flag_clone = stop_flag.clone();   // ~1 нс, данные не копируются
let ctx_clone = ctx.clone();               // Клонируем контекст для перемещения в поток

let sensor_handle = thread::spawn(move || {
    // ...использует stop_flag_clone и ctx_clone
});
```

### Чек-лист: нужно ли клонировать?
1. **Могу ли я принять `&str` / `&T` вместо `String` / `T`?** → Заимствуйте, не клонируйте
2. **Могу ли я перестроить код, чтобы не требовалось два владельца?** → Передавайте по ссылке или используйте области видимости
3. **Это `Arc::clone()`?** → Это нормально, это O(1)
4. **Я перемещаю данные в поток или замыкание?** → Клонирование необходимо
5. **Я клонирую в горячем цикле?** → Профилируйте и рассмотрите заимствование или `Cow<T>`

----

## `Cow<'a, T>`: Clone-on-Write — заимствуй, когда можешь, клонируй, когда должен

`Cow` (Clone on Write, «клонирование при записи») — это перечисление, которое хранит **либо** заимствованную ссылку, **либо**
владеющее значение. Это аналог в Rust идеи «избегай выделения памяти, когда возможно, но выделяй, если нужно изменить».
В C++ прямого аналога нет — ближайший вариант — функция, которая иногда возвращает `const std::string&`, а иногда `std::string`.

### Зачем нужен `Cow`

```rust
// Без Cow — приходится выбирать: всегда заимствовать ИЛИ всегда клонировать
fn normalize(s: &str) -> String {          // Всегда выделяет память!
    if s.contains(' ') {
        s.replace(' ', "_")               // Новая String (выделение необходимо)
    } else {
        s.to_string()                     // Ненужное выделение памяти!
    }
}

// С Cow — заимствуем, если строка не изменилась, выделяем только при изменении
use std::borrow::Cow;

fn normalize(s: &str) -> Cow<'_, str> {
    if s.contains(' ') {
        Cow::Owned(s.replace(' ', "_"))    // Выделяет память (нужно изменить)
    } else {
        Cow::Borrowed(s)                   // Ноль выделений (проход насквозь)
    }
}
```

### Как работает `Cow`

```rust
use std::borrow::Cow;

// Cow<'a, str> — по сути:
// enum Cow<'a, str> {
//     Borrowed(&'a str),     // Ссылка без накладных расходов
//     Owned(String),          // Владеющее значение в куче
// }

fn greet(name: &str) -> Cow<'_, str> {
    if name.is_empty() {
        Cow::Borrowed("stranger")         // Статическая строка — без выделения
    } else if name.starts_with(' ') {
        Cow::Owned(name.trim().to_string()) // Изменено — нужно выделение
    } else {
        Cow::Borrowed(name)               // Проход насквозь — без выделения
    }
}

fn main() {
    let g1 = greet("Alice");     // Cow::Borrowed("Alice")
    let g2 = greet("");          // Cow::Borrowed("stranger")
    let g3 = greet(" Bob ");     // Cow::Owned("Bob")
    
    // Cow<str> реализует Deref<Target = str>, поэтому его можно использовать как &str:
    println!("Hello, {g1}!");    // Работает — Cow автоматически разыменовывается в &str
    println!("Hello, {g2}!");
    println!("Hello, {g3}!");
}
```

### Пример из практики: нормализация значений конфигурации

```rust
use std::borrow::Cow;

/// Нормализует имя SKU: убирает пробелы по краям, приводит к нижнему регистру.
/// Возвращает Cow::Borrowed, если строка уже нормализована (ноль выделений).
fn normalize_sku(sku: &str) -> Cow<'_, str> {
    let trimmed = sku.trim();
    if trimmed == sku && sku.chars().all(|c| c.is_lowercase() || !c.is_alphabetic()) {
        Cow::Borrowed(sku)   // Уже нормализована — без выделения
    } else {
        Cow::Owned(trimmed.to_lowercase())  // Нужно изменить — выделяем
    }
}

fn main() {
    let s1 = normalize_sku("server-x1");   // Borrowed — ноль выделений
    let s2 = normalize_sku("  Server-X1 "); // Owned — приходится выделять
    println!("{s1}, {s2}"); // "server-x1, server-x1"
}
```

### Когда использовать `Cow`

| **Ситуация** | **Использовать `Cow`?** |
|--------------|---------------|
| Функция чаще всего возвращает вход без изменений | ✅ Да — избегаем ненужных клонирований |
| Разбор/нормализация строк (trim, lowercase, replace) | ✅ Да — часто вход уже корректен |
| Всегда что-то меняем — каждый путь выполнения выделяет память | ❌ Нет — просто возвращаем `String` |
| Простой проход насквозь (никогда не меняем) | ❌ Нет — просто возвращаем `&str` |
| Данные долгосрочно хранятся в структуре | ❌ Нет — используйте `String` (владеющую) |

> **Сравнение с C++**: `Cow<str>` похож на функцию, которая возвращает `std::variant<std::string_view, std::string>`, — только с автоматическим разыменованием и без шаблонного кода для доступа к значению.

----

## `Weak<T>`: разрыв циклов ссылок — `weak_ptr` в Rust

`Weak<T>` — это аналог `std::weak_ptr<T>` из C++. Он хранит невладеющую ссылку на значение `Rc<T>` или `Arc<T>`. Значение может быть освобождено, пока существуют слабые ссылки `Weak` — вызов `upgrade()` вернёт `None`, если значения уже нет.

### Зачем нужен `Weak`

`Rc<T>` и `Arc<T>` образуют циклы ссылок, если два значения указывают друг на друга — ни одно из них никогда не достигнет счётчика 0, поэтому ни одно не будет уничтожено (утечка памяти). `Weak` разрывает цикл:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: String,
    parent: RefCell<Weak<Node>>,      // Weak — не мешает родителю уничтожиться
    children: RefCell<Vec<Rc<Node>>>,  // Сильная — родитель владеет потомками
}

impl Node {
    fn new(value: &str) -> Rc<Node> {
        Rc::new(Node {
            value: value.to_string(),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }

    fn add_child(parent: &Rc<Node>, child: &Rc<Node>) {
        // У потомка слабая ссылка на родителя (цикла нет)
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        // У родителя сильная ссылка на потомка
        parent.children.borrow_mut().push(Rc::clone(child));
    }
}

fn main() {
    let root = Node::new("root");
    let child = Node::new("child");
    Node::add_child(&root, &child);

    // Доступ к родителю из потомка через upgrade()
    if let Some(parent) = child.parent.borrow().upgrade() {
        println!("Child's parent: {}", parent.value); // "root"
    }
    
    println!("Root strong count: {}", Rc::strong_count(&root));  // 1
    println!("Root weak count: {}", Rc::weak_count(&root));      // 1
}
```

### Сравнение с C++

```cpp
// C++ — weak_ptr для разрыва цикла shared_ptr
struct Node {
    std::string value;
    std::weak_ptr<Node> parent;                  // Слабая — без владения
    std::vector<std::shared_ptr<Node>> children;  // Сильные — владеет потомками

    static auto create(const std::string& v) {
        return std::make_shared<Node>(Node{v, {}, {}});
    }
};

auto root = Node::create("root");
auto child = Node::create("child");
child->parent = root;          // Присваивание weak_ptr
root->children.push_back(child);

if (auto p = child->parent.lock()) {   // lock() → shared_ptr или null
    std::cout << "Parent: " << p->value << std::endl;
}
```

| C++ | Rust | Примечания |
|-----|------|-------|
| `shared_ptr<T>` | `Rc<T>` (один поток) / `Arc<T>` (несколько потоков) | Та же семантика |
| `weak_ptr<T>` | `Weak<T>` из `Rc::downgrade()` / `Arc::downgrade()` | Та же семантика |
| `weak_ptr::lock()` → `shared_ptr` или null | `Weak::upgrade()` → `Option<Rc<T>>` | `None`, если значение уничтожено |
| `shared_ptr::use_count()` | `Rc::strong_count()` | Тот же смысл |

### Когда использовать `Weak`

| **Ситуация** | **Паттерн** |
|--------------|-----------|
| Связи «родитель ↔ потомок» в дереве | Родитель хранит `Rc<Child>`, потомок хранит `Weak<Parent>` |
| Паттерн «наблюдатель» / слушатели событий | Источник событий хранит `Weak<Observer>`, наблюдатель хранит `Rc<Source>` |
| Кэш, который не мешает освобождению | `HashMap<Key, Weak<Value>>` — записи естественным образом устаревают |
| Разрыв циклов в графовых структурах | Перекрёстные связи используют `Weak`, рёбра дерева — `Rc`/`Arc` |

> **В новом коде предпочитайте паттерн арены** (кейс 2) вместо `Rc/Weak` для древовидных структур. `Vec<T>` с индексами проще, быстрее и не имеет накладных расходов на подсчёт ссылок. Используйте `Rc/Weak`, когда нужно совместное владение с динамическими временами жизни.

----

## Copy и Clone, PartialEq и Eq — что и когда выводить через derive

- **Copy ≈ тривиально копируемые типы C++ (без пользовательского конструктора копирования и деструктора).** Такие типы, как `int`, `enum` и простые POD-структуры, — компилятор автоматически генерирует побитовый `memcpy`. В Rust `Copy` — та же идея: присваивание `let b = a;` выполняет неявное побитовое копирование, и обе переменные остаются валидными.
- **Clone ≈ глубокое копирование в конструкторе копирования / `operator=` C++.** Когда у класса C++ есть пользовательский конструктор копирования (например, для глубокого копирования члена `std::vector`), в Rust эквивалент — реализация `Clone`. Вызывать `.clone()` нужно явно — Rust никогда не прячет дорогое копирование за `=`.
- **Ключевое различие:** в C++ и тривиальные, и глубокие копии происходят неявно через один и тот же синтаксис `=`. Rust заставляет выбирать: типы `Copy` копируются молча (дёшево), нециклопические типы по умолчанию **перемещаются**, а дорогое дублирование нужно включить явно через `.clone()`.
- Аналогично, C++ `operator==` не различает типы, где `a == a` всегда истинно (как целые), и типы, где это не так (как `float` с NaN). Rust кодирует это в `PartialEq` и `Eq`.

### Copy и Clone

| | **Copy** | **Clone** |
|---|---------|----------|
| **Как работает** | Побитовый memcpy (неявно) | Пользовательская логика (явно через `.clone()`) |
| **Когда происходит** | При присваивании: `let b = a;` | Только при вызове `.clone()` |
| **После копирования/клонирования** | И `a`, и `b` валидны | И `a`, и `b` валидны |
| **Без них** | `let b = a;` **перемещает** `a` (a больше нет) | `let b = a;` **перемещает** `a` (a больше нет) |
| **Допустимо для** | Типов без данных в куче | Любых типов |
| **Аналогия в C++** | Тривиально копируемые / POD-типы (без пользовательского конструктора копирования) | Пользовательский конструктор копирования (глубокая копия) |

### Реальный пример: Copy — простые перечисления
```rust
// Из fan_diag/src/sensor.rs — все варианты без данных, помещаются в 1 байт
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FanStatus {
    #[default]
    Normal,
    Low,
    High,
    Missing,
    Failed,
    Unknown,
}

let status = FanStatus::Normal;
let copy = status;   // Неявное копирование — status по-прежнему валиден
println!("{:?} {:?}", status, copy);  // Работает оба
```

### Реальный пример: Copy — перечисление с целочисленными данными
```rust
// Пример: healthcheck.rs — полезная нагрузка u32 является Copy, значит, и всё перечисление тоже
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthcheckStatus {
    Pass,
    ProgramError(u32),
    DmesgError(u32),
    RasError(u32),
    OtherError(u32),
    Unknown,
}
```

### Реальный пример: только Clone — структура с данными в куче
```rust
// Пример: components.rs — String мешает Copy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FruData {
    pub technology: DeviceTechnology,
    pub physical_location: String,      // ← String: в куче, не может быть Copy
    pub expected: bool,
    pub removable: bool,
}
// let a = fru_data;   → ПЕРЕМЕЩАЕТ (a — это единственный владелец, fru_data больше недоступна)
// let a = fru_data.clone();  → КЛОНИРУЕТ (fru_data по-прежнему валидна, новое выделение в куче)
```

### Правило: может ли тип быть Copy?
```text
Содержит ли тип String, Vec, Box, HashMap,
Rc, Arc или любой другой тип, владеющий кучей?
    ДА  → только Clone (не может быть Copy)
    НЕТ → МОЖНО выводить Copy (и стоит, если тип небольшой)
```

### PartialEq и Eq

| | **PartialEq** | **Eq** |
|---|--------------|-------|
| **Что даёт** | Операторы `==` и `!=` | Маркер: «равенство рефлексивно» |
| **Рефлексивно? (a == a)** | Не гарантируется | **Гарантируется** |
| **Почему важно** | `f32::NAN != f32::NAN` | Ключи `HashMap` **требуют** `Eq` |
| **Когда выводить** | Почти всегда | Когда в типе нет полей `f32`/`f64` |
| **Аналогия в C++** | `operator==` | Прямого аналога нет (C++ не проверяет) |

### Реальный пример: Eq — используется как ключ HashMap
```rust
// Из hms_trap/src/cpu_handler.rs — Hash требует Eq
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpuFaultType {
    InvalidFaultType,
    CpuCperFatalErr,
    CpuLpddr5UceErr,
    CpuC2CUceFatalErr,
    // ...
}
// Используется как: HashMap<CpuFaultType, FaultHandler>
// Ключи HashMap должны быть Eq + Hash — одного PartialEq недостаточно для компиляции
```

### Реальный пример: Eq невозможен — тип содержит f32
```rust
// Пример: types.rs — f32 не позволяет Eq
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemperatureSensors {
    pub warning_threshold: Option<f32>,   // ← f32: NaN ≠ NaN
    pub critical_threshold: Option<f32>,  // ← нельзя вывести Eq
    pub sensor_names: Vec<String>,
}
// Нельзя использовать как ключ HashMap. Нельзя вывести Eq.
// Потому что: f32::NAN == f32::NAN ложно, что нарушает рефлексивность.
```

### PartialOrd и Ord

| | **PartialOrd** | **Ord** |
|---|---------------|--------|
| **Что даёт** | `<`, `>`, `<=`, `>=` | `.sort()`, ключи `BTreeMap` |
| **Полный порядок?** | Нет (некоторые пары могут быть несравнимы) | **Да** (любые два значения сравнимы) |
| **f32/f64?** | Только PartialOrd (NaN ломает порядок) | Нельзя вывести Ord |

### Реальный пример: Ord — ранжирование серьёзности
```rust
// Из hms_trap/src/fault.rs — порядок вариантов задаёт серьёзность
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultSeverity {
    Info,      // самый низкий  (дискриминант 0)
    Warning,   //               (дискриминант 1)
    Error,     //               (дискриминант 2)
    Critical,  // самый высокий (дискриминант 3)
}
// FaultSeverity::Info < FaultSeverity::Critical → true
// Позволяет: if severity >= FaultSeverity::Error { escalate(); }
```

### Реальный пример: Ord — уровни диагностики для сравнения
```rust
// Пример: orchestration.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum GpuDiagLevel {
    #[default]
    Quick,     // самый низкий
    Standard,
    Extended,
    Full,      // самый высокий
}
// Позволяет: if requested_level >= GpuDiagLevel::Extended { run_extended_tests(); }
```

### Дерево решений по derive

```text
                        Ваш новый тип
                            │
                   Содержит String/Vec/Box?
                      /              \
                    ДА                НЕТ
                     │                  │
              Только Clone        Clone + Copy
                     │                  │
              Содержит f32/f64?    Содержит f32/f64?
                /          \         /          \
              ДА            НЕТ     ДА            НЕТ
               │             │      │             │
         Только          PartialEq  Только      PartialEq
         PartialEq       + Eq       PartialEq   + Eq
                          │                      │
                    Нужна сортировка?      Нужна сортировка?
                      /       \               /       \
                    ДА         НЕТ          ДА         НЕТ
                     │          │              │          │
               PartialOrd    Готово      PartialOrd    Готово
               + Ord                     + Ord
                     │                        │
               Нужен как                Нужен как
               ключ словаря?            ключ словаря?
                  │                        │
                + Hash                   + Hash
```

### Краткий справочник: типичные комбинации derive из продакшн-кода на Rust

| **Категория типа** | **Типичный derive** | **Пример** |
|-------------------|--------------------|------------|
| Простое перечисление статусов | `Copy, Clone, PartialEq, Eq, Default` | `FanStatus` |
| Перечисление как ключ HashMap | `Copy, Clone, PartialEq, Eq, Hash` | `CpuFaultType`, `SelComponent` |
| Сортируемое перечисление серьёзности | `Copy, Clone, PartialEq, Eq, PartialOrd, Ord` | `FaultSeverity`, `GpuDiagLevel` |
| Структура данных со строками | `Clone, Debug, Serialize, Deserialize` | `FruData`, `OverallSummary` |
| Сериализуемая конфигурация | `Clone, Debug, Default, Serialize, Deserialize` | `DiagConfig` |

----

