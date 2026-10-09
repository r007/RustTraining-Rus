# 15. Архитектура крейтов и дизайн API 🟡

> **Что вы узнаете:**
> - Соглашения о структуре модулей и стратегии реэкспорта
> - Чек-лист проектирования публичного API для аккуратных крейтов
> - Эргономичные паттерны параметров: `impl Into`, `AsRef`, `Cow`
> - «Parse, don't validate» с `TryFrom` и проверенными типами
> - Флаги функций, условную компиляцию и организацию воркспейсов

## Соглашения о структуре модулей

```text
my_crate/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Корень крейта: реэкспорты и публичный API
│   ├── config.rs       # Модуль функциональности
│   ├── parser/         # Сложный модуль с подмодулями
│   │   ├── mod.rs      # или parser.rs на уровне родителя (Rust 2018+)
│   │   ├── lexer.rs
│   │   └── ast.rs
│   ├── error.rs        # Типы ошибок
│   └── utils.rs        # Внутренние утилиты (pub(crate))
├── tests/
│   └── integration.rs  # Интеграционные тесты
├── benches/
│   └── perf.rs         # Бенчмарки
└── examples/
    └── basic.rs        # cargo run --example basic
```

```rust
// lib.rs: курируйте публичный API через реэкспорты:
mod config;
mod error;
mod parser;
mod utils;

// Реэкспортируем то, что нужно пользователям:
pub use config::Config;
pub use error::Error;
pub use parser::Parser;

// Публичные типы находятся в корне крейта, пользователи пишут:
// use my_crate::Config;
// НЕ так: use my_crate::config::Config;
```

**Модификаторы видимости**:

| Модификатор | Видим для |
|-------------|-----------|
| `pub` | Всех |
| `pub(crate)` | Только этого крейта |
| `pub(super)` | Родительского модуля |
| `pub(in path)` | Конкретного модуля-предка |
| (нет) | Текущего модуля и его потомков |

### Чек-лист проектирования публичного API

1. **Принимайте ссылки, возвращайте владеющие значения**: `fn process(input: &str) -> String`
2. **Используйте `impl Trait` для параметров**: `fn read(r: impl Read)` вместо `fn read<R: Read>(r: R)` для более чистых сигнатур
3. **Возвращайте `Result`, а не `panic!`**: пусть вызывающий код решает, как обрабатывать ошибки
4. **Реализуйте стандартные трейты**: `Debug`, `Display`, `Clone`, `Default`, `From`/`Into`
5. **Делайте недопустимые состояния невыразимыми**: используйте type-state и newtype
6. **Используйте паттерн builder для сложной конфигурации**: с type-state, если поля обязательны
7. **Запечатывайте трейты, которые не должны реализовывать пользователи**: `pub trait Sealed: private::Sealed {}`
8. **Помечайте типы и функции `#[must_use]`**: это предотвращает молчаливое игнорирование важных `Result`, защитников или значений. Применяйте к любому типу, где игнорирование возвращаемого значения почти наверняка является ошибкой:
   ```rust
   #[must_use = "немедленное уничтожение защитника сразу освобождает блокировку"]
   pub struct LockGuard<'a, T> { /* ... */ }

   #[must_use]
   pub fn validate(input: &str) -> Result<ValidInput, ValidationError> { /* ... */ }
   ```

```rust
// Паттерн sealed-трейта: пользователи могут использовать, но не реализовать:
mod private {
    pub trait Sealed {}
}

pub trait DatabaseDriver: private::Sealed {
    fn connect(&self, url: &str) -> Connection;
}

// Только типы ЭТОГО крейта могут реализовать Sealed, значит, только мы можем реализовать DatabaseDriver
pub struct PostgresDriver;
impl private::Sealed for PostgresDriver {}
impl DatabaseDriver for PostgresDriver {
    fn connect(&self, url: &str) -> Connection { /* ... */ }
}
```

> **`#[non_exhaustive]`**: помечайте публичные перечисления и структуры так, чтобы добавление вариантов или полей не было несовместимым изменением. Внешние крейты должны использовать шаблонную ветку (`_ =>`) в `match` и не могут создавать этот тип с помощью синтаксиса литерала структуры:
> ```rust
> #[non_exhaustive]
> pub enum DiagError {
>     Timeout,
>     HardwareFault,
>     // Добавление нового варианта в будущем выпуске НЕ является несовместимым изменением semver.
> }
> ```

### Эргономичные паттерны параметров: `impl Into`, `AsRef`, `Cow`

Один из самых влиятельных паттернов API в Rust: принимать в параметрах функций **наиболее общий тип**, чтобы вызывающему коду не приходилось на каждом шагу писать повторяющиеся `.to_string()`, `&*s` или `.as_ref()`. Это версия принципа «будь либерален в том, что принимаешь», специфичная для Rust.

#### `impl Into<T>`: принимаем всё, что конвертируется

```rust
// ❌ Неудобно: вызывающие должны конвертировать вручную
fn connect(host: String, port: u16) -> Connection {
    // ...
}
connect("localhost".to_string(), 5432);  // Раздражающий .to_string()
connect(hostname.clone(), 5432);          // Лишнее клонирование, если уже есть String

// ✅ Удобно: принимаем всё, что конвертируется в String
fn connect(host: impl Into<String>, port: u16) -> Connection {
    let host = host.into();  // Конвертируем один раз, внутри функции
    // ...
}
connect("localhost", 5432);     // &str: без лишних усилий
connect(hostname, 5432);        // String: перемещаем, без клонирования
```

Это работает, потому что пара трейтов `From`/`Into` обеспечивает общие преобразования. Принимая `impl Into<T>`, вы говорите: «дайте мне всё, что умеет стать `T`».

#### `AsRef<T>`: заимствование как ссылка

`AsRef<T>` это заимствующий аналог `Into<T>`. Используйте его, когда нужно только *читать* данные, а не забирать владение:

```rust
use std::path::Path;

// ❌ Вынуждает вызывающих конвертировать в &Path
fn file_exists(path: &Path) -> bool {
    path.exists()
}
file_exists(Path::new("/tmp/test.txt"));  // Неудобно

// ✅ Принимаем всё, что может вести себя как &Path
fn file_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}
file_exists("/tmp/test.txt");                    // &str ✅
file_exists(String::from("/tmp/test.txt"));      // String ✅
file_exists(Path::new("/tmp/test.txt"));         // &Path ✅
file_exists(PathBuf::from("/tmp/test.txt"));     // PathBuf ✅

// Тот же паттерн для строковых параметров:
fn log_message(msg: impl AsRef<str>) {
    println!("[LOG] {}", msg.as_ref());
}
log_message("hello");                    // &str ✅
log_message(String::from("hello"));      // String ✅
```

#### `Cow<T>`: клонирование при записи

`Cow<'a, T>` (Clone on Write) откладывает выделение памяти до момента, когда понадобится изменение. Он хранит либо заимствованный `&T`, либо владеющий `T::Owned`. Это идеально, когда большинство вызовов не требует изменения данных:

```rust
use std::borrow::Cow;

/// Нормализует диагностическое сообщение: выделяет память, только если нужны изменения.
fn normalize_message(msg: &str) -> Cow<'_, str> {
    if msg.contains('\t') || msg.contains('\r') {
        // Нужно выделить память: требуется изменить содержимое
        Cow::Owned(msg.replace('\t', "    ").replace('\r', ""))
    } else {
        // Без выделения памяти: просто заимствуем исходное
        Cow::Borrowed(msg)
    }
}

// Большинство сообщений проходят без выделения памяти:
let clean = normalize_message("Все тесты пройдены");        // Borrowed: бесплатно
let fixed = normalize_message("Ошибка:\tсбой\r\n");         // Owned: выделено

// Cow<str> реализует Deref<Target=str>, поэтому работает как &str:
println!("{}", clean);
println!("{}", fixed.to_uppercase());
```

#### Краткая справка: что выбрать

```text
Нужно ли функции владеть данными внутри?
├── ДА → impl Into<T>
│         «Дайте мне всё, что может стать T»
└── НЕТ → Нужно ли только читать?
     ├── ДА → impl AsRef<T> или &T
     │         «Дайте мне всё, что можно заимствовать как &T»
     └── ВОЗМОЖНО (иногда нужно изменять?)
          └── Cow<'_, T>
              «Заимствуем, если возможно, клонируем, только когда это необходимо»
```

| Паттерн | Владение | Выделение памяти | Когда использовать |
|---------|----------|------------------|--------------------|
| `&str` | Заимствование | Никогда | Простые строковые параметры |
| `impl AsRef<str>` | Заимствование | Никогда | Принимает String, &str и т. д.: только чтение |
| `impl Into<String>` | Владение | При конвертации | Принимает &str и String: будет хранить или владеть |
| `Cow<'_, str>` | Любое | Только при изменении | Обработка, которая обычно не меняет данные |
| `&[u8]` / `impl AsRef<[u8]>` | Заимствование | Никогда | API для работы с байтами |

> **`Borrow<T>` против `AsRef<T>`**: оба дают `&T`, но `Borrow<T>` дополнительно гарантирует, что `Eq`, `Ord` и `Hash` **согласованы** между исходной и заимствованной формой. Поэтому `HashMap<String, V>::get()` принимает `&Q` при условии `String: Borrow<Q>`, а не `AsRef`. Используйте `Borrow`, когда заимствованная форма применяется как ключ поиска, а `AsRef` для общих параметров «дайте мне ссылку».

#### Композиция преобразований в API

```rust
/// Хорошо спроектированный API для диагностики с эргономичными параметрами:
pub struct DiagRunner {
    name: String,
    config_path: PathBuf,
    results: HashMap<String, TestResult>,
}

impl DiagRunner {
    /// Принимаем любой строковый тип для name и любой путь для config.
    pub fn new(
        name: impl Into<String>,
        config_path: impl Into<PathBuf>,
    ) -> Self {
        DiagRunner {
            name: name.into(),
            config_path: config_path.into(),
        }
    }

    /// Принимаем любой AsRef<str> для поиска только для чтения.
    pub fn get_result(&self, test_name: impl AsRef<str>) -> Option<&TestResult> {
        self.results.get(test_name.as_ref())
    }
}

// Все эти вызовы работают без лишних усилий со стороны вызывающего кода:
let runner = DiagRunner::new("GPU Diag", "/etc/diag_tool/config.json");
let runner = DiagRunner::new(format!("Diag-{}", node_id), config_path);
let runner = DiagRunner::new(name_string, path_buf);
```

***

## Пример из практики: проектирование публичного API крейта, до и после

Реальный пример перехода от внутреннего API, построенного на строках («строковый» API, stringly-typed), к эргономичному типобезопасному публичному API. Рассмотрим крейт-парсер конфигурации:

**До** (строки повсюду, легко ошибиться):

```rust
// ❌ Все параметры — строки: нет проверки на этапе компиляции
pub fn parse_config(path: &str, format: &str, strict: bool) -> Result<Config, String> {
    // Какие форматы допустимы? "json"? "JSON"? "Json"?
    // path — это путь к файлу или URL?
    // Что вообще значит "strict"?
    todo!()
}
```

**После** (типобезопасно, самодокументируется):

```rust
use std::path::Path;

/// Поддерживаемые форматы конфигурации.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]  // Добавление форматов не сломает внешний код
pub enum Format {
    Json,
    Toml,
    Yaml,
}

/// Управляет строгостью разбора.
#[derive(Debug, Clone, Copy, Default)]
pub enum Strictness {
    /// Отвергать неизвестные поля (по умолчанию для библиотек)
    #[default]
    Strict,
    /// Игнорировать неизвестные поля (полезно для конфигов с прямой совместимостью)
    Lenient,
}

pub fn parse_config(
    path: &Path,          // Проверяется типом: должен быть путём файловой системы
    format: Format,       // Перечисление: невозможно передать неверный формат
    strictness: Strictness,  // Именованные варианты, а не голый bool
) -> Result<Config, ConfigError> {
    todo!()
}
```

**Что улучшилось**:

| Аспект | До | После |
|--------|----|-------|
| Проверка формата | Сравнение строк во время выполнения | Перечисление на этапе компиляции |
| Тип пути | Сырой `&str` (что угодно) | `&Path` (специфичный для файловой системы) |
| Строгость | Загадочный `bool` | Самодокументируемое перечисление |
| Тип ошибки | `String` (непрозрачный) | `ConfigError` (структурированный) |
| Расширяемость | Несовместимые изменения | `#[non_exhaustive]` |

> **Практическое правило**: если вы пишете `match` по строковым значениям, подумайте о замене параметра перечислением. Если параметр это булев флаг, смысл которого неочевиден из контекста, используйте перечисление с двумя вариантами.

***

### Parse Don't Validate: `TryFrom` и проверенные типы

«Parse, don't validate» (разбирай, а не проверяй) это принцип, который говорит: **не проверяйте данные и затем не передавайте дальше непроверенную сырую форму. Вместо этого разберите их в тип, который может существовать только тогда, когда данные корректны.** Стандартный инструмент Rust для этого трейт `TryFrom`.

#### Проблема: проверка без принуждения

```rust
// ❌ Проверяем, а потом используем: ничто не мешает использовать некорректное значение после проверки
fn process_port(port: u16) {
    if port == 0 || port > 65535 {
        panic!("Некорректный порт");      // Мы проверили, но...
    }
    start_server(port);                    // Что если кто-то вызовет start_server(0) напрямую?
}

// ❌ Строка на все случаи: email это просто String, любой мусор проходит
fn send_email(to: String, body: String) {
    // Действительно ли `to` корректный email? Мы не знаем.
    // Кто-то может передать "not-an-email", и мы узнаем об этом только на SMTP-сервере.
}
```

#### Решение: разбор в проверенные newtype через `TryFrom`

```rust
use std::convert::TryFrom;
use std::fmt;

/// Проверенный номер TCP-порта (1–65535).
/// Если у вас есть `Port`, он гарантированно корректен.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(u16);

impl TryFrom<u16> for Port {
    type Error = PortError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value == 0 {
            Err(PortError::Zero)
        } else {
            Ok(Port(value))
        }
    }
}

impl Port {
    pub fn get(&self) -> u16 { self.0 }
}

#[derive(Debug)]
pub enum PortError {
    Zero,
    InvalidFormat,
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortError::Zero => write!(f, "порт должен быть ненулевым"),
            PortError::InvalidFormat => write!(f, "некорректный формат порта"),
        }
    }
}

impl std::error::Error for PortError {}

// Теперь система типов обеспечивает корректность:
fn start_server(port: Port) {
    // Проверка не нужна: Port можно создать только через TryFrom,
    // который уже проверил его корректность.
    println!("Слушаем порт {}", port.get());
}

// Использование:
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = Port::try_from(8080)?;   // ✅ Проверяем один раз на границе
    start_server(port);                  // Никакой повторной проверки ниже по цепочке

    let bad = Port::try_from(0);         // ❌ Err(PortError::Zero)
    Ok(())
}
```

#### Пример из практики: проверенный адрес IPMI

```rust
/// Проверенный адрес ведомого устройства IPMI (0x20–0xFE, только чётные).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpmiAddr(u8);

#[derive(Debug)]
pub enum IpmiAddrError {
    Odd(u8),
    OutOfRange(u8),
}

impl fmt::Display for IpmiAddrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpmiAddrError::Odd(v) => write!(f, "адрес IPMI 0x{v:02X} должен быть чётным"),
            IpmiAddrError::OutOfRange(v) => {
                write!(f, "адрес IPMI 0x{v:02X} вне диапазона (0x20..=0xFE)")
            }
        }
    }
}

impl TryFrom<u8> for IpmiAddr {
    type Error = IpmiAddrError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value % 2 != 0 {
            Err(IpmiAddrError::Odd(value))
        } else if value < 0x20 || value > 0xFE {
            Err(IpmiAddrError::OutOfRange(value))
        } else {
            Ok(IpmiAddr(value))
        }
    }
}

impl IpmiAddr {
    pub fn get(&self) -> u8 { self.0 }
}

// Код ниже по цепочке никогда не нужно проверять заново:
fn send_ipmi_command(addr: IpmiAddr, cmd: u8, data: &[u8]) -> Result<Vec<u8>, IpmiError> {
    // addr.get() гарантированно является корректным чётным адресом IPMI
    raw_ipmi_send(addr.get(), cmd, data)
}
```

#### Разбор строк через `FromStr`

Для типов, которые часто разбираются из текста (аргументы CLI, файлы конфигурации), реализуйте `FromStr`:

```rust
use std::str::FromStr;

impl FromStr for Port {
    type Err = PortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n: u16 = s.parse().map_err(|_| PortError::InvalidFormat)?;
        Port::try_from(n)
    }
}

// Теперь работает с .parse():
let port: Port = "8080".parse()?;   // Проверяется за один шаг

// И с разбором аргументов clap:
// #[derive(Parser)]
// struct Args {
//     #[arg(short, long)]
//     port: Port,   // clap вызывает FromStr автоматически
// }
```

#### Цепочка `TryFrom` для сложной проверки

```rust
// Заглушки типов для этого примера. В продакшене они были бы в
// отдельных модулях со своими реализациями TryFrom.
```

```rust
# struct Hostname(String);
# impl TryFrom<String> for Hostname {
#     type Error = String;
#     fn try_from(s: String) -> Result<Self, String> { Ok(Hostname(s)) }
# }
# struct Timeout(u64);
# impl TryFrom<u64> for Timeout {
#     type Error = String;
#     fn try_from(ms: u64) -> Result<Self, String> {
#         if ms == 0 { Err("таймаут должен быть > 0".into()) } else { Ok(Timeout(ms)) }
#     }
# }
# struct RawConfig { host: String, port: u16, timeout_ms: u64 }
# #[derive(Debug)]
# enum ConfigError {
#     InvalidHost(String),
#     InvalidPort(PortError),
#     InvalidTimeout(String),
# }
# impl From<std::io::Error> for ConfigError {
#     fn from(e: std::io::Error) -> Self { ConfigError::InvalidHost(e.to_string()) }
# }
# impl From<serde_json::Error> for ConfigError {
#     fn from(e: serde_json::Error) -> Self { ConfigError::InvalidHost(e.to_string()) }
# }
/// Проверенная конфигурация, которая может существовать, только если все поля корректны.
pub struct ValidConfig {
    pub host: Hostname,
    pub port: Port,
    pub timeout_ms: Timeout,
}

impl TryFrom<RawConfig> for ValidConfig {
    type Error = ConfigError;

    fn try_from(raw: RawConfig) -> Result<Self, Self::Error> {
        Ok(ValidConfig {
            host: Hostname::try_from(raw.host)
                .map_err(ConfigError::InvalidHost)?,
            port: Port::try_from(raw.port)
                .map_err(ConfigError::InvalidPort)?,
            timeout_ms: Timeout::try_from(raw.timeout_ms)
                .map_err(ConfigError::InvalidTimeout)?,
        })
    }
}

// Разбираем один раз на границе, используем проверенный тип везде:
fn load_config(path: &str) -> Result<ValidConfig, ConfigError> {
    let raw: RawConfig = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    ValidConfig::try_from(raw)  // Вся проверка происходит здесь
}
```

#### Итоги: проверка против разбора

| Подход | Данные проверены? | Компилятор следит за корректностью? | Нужна повторная проверка? |
|--------|:-----------------:|:----------------------------------:|:-------------------------:|
| Проверки во время выполнения (if/assert) | ✅ | ❌ | На каждой границе функции |
| Проверенный newtype + `TryFrom` | ✅ | ✅ | Никогда: тип является доказательством |

Правило: **разбирайте на границе, а внутри используйте проверенные типы.** Сырые строки, целые числа и срезы байтов попадают в вашу систему, разбираются в проверенные типы через `TryFrom`/`FromStr`, и с этого момента система типов гарантирует их корректность.

### Флаги функций и условная компиляция

```toml
# Cargo.toml
[features]
default = ["json"]          # Включена по умолчанию
json = ["dep:serde_json"]   # Включает поддержку JSON
xml = ["dep:quick-xml"]     # Включает поддержку XML
full = ["json", "xml"]      # Мета-фича: включает всё

[dependencies]
serde = "1"
serde_json = { version = "1", optional = true }
quick-xml = { version = "0.31", optional = true }
```

```rust
// Условная компиляция в зависимости от фич:
#[cfg(feature = "json")]
pub fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

#[cfg(feature = "xml")]
pub fn to_xml<T: serde::Serialize>(value: &T) -> String {
    quick_xml::se::to_string(value).unwrap()
}

// Ошибка компиляции, если нужная фича не включена:
#[cfg(not(any(feature = "json", feature = "xml")))]
compile_error!("Должна быть включена хотя бы одна фича формата (json, xml)");
```

**Лучшие практики**:
- Держите фичи в `default` минимальными: пользователи сами включат то, что им нужно
- Используйте синтаксис `dep:` (Rust 1.60+) для необязательных зависимостей, чтобы не создавать неявные фичи
- Документируйте фичи в README и в документации крейта

### Организация воркспейса

Для больших проектов используйте воркспейс Cargo, чтобы разделять зависимости и артефакты сборки:

```toml
# Корневой Cargo.toml
[workspace]
members = [
    "core",         # Общие типы и трейты
    "parser",       # Библиотека разбора
    "server",       # Бинарный файл: основное приложение
    "client",       # Клиентская библиотека
    "cli",          # Бинарный файл CLI
]

# Общие версии зависимостей:
[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"

# В Cargo.toml каждого участника:
# [dependencies]
# serde = { workspace = true }
```

**Преимущества**:
- Один `Cargo.lock`: все крейты используют одни и те же версии зависимостей
- `cargo test --workspace` запускает все тесты
- Общий кэш сборки: компиляция одного крейта ускоряет остальные
- Чистые границы зависимостей между компонентами

### `.cargo/config.toml`: конфигурация на уровне проекта

Файл `.cargo/config.toml` (в корне воркспейса или в `$HOME/.cargo/`) настраивает поведение Cargo без изменения `Cargo.toml`:

```toml
# .cargo/config.toml

# Цель сборки по умолчанию для этого воркспейса
[build]
target = "x86_64-unknown-linux-gnu"

# Пользовательский runner: например, запуск через QEMU для кросс-компилированных бинарников
[target.aarch64-unknown-linux-gnu]
runner = "qemu-aarch64-static"
linker = "aarch64-linux-gnu-gcc"

# Псевдонимы Cargo: пользовательские короткие команды
[alias]
xt = "test --workspace --release"        # cargo xt = запустить все тесты в release
ci = "clippy --workspace -- -D warnings" # cargo ci = линтинг с ошибками на предупреждениях
cov = "llvm-cov --workspace"             # cargo cov = покрытие (требует cargo-llvm-cov)

# Переменные окружения для build-скриптов
[env]
IPMI_LIB_PATH = "/usr/lib/bmc"

# Использовать пользовательский реестр (для внутренних пакетов)
# [registries.internal]
# index = "https://gitlab.internal/crates/index"
```

Типичные настройки:

| Настройка | Назначение | Пример |
|-----------|------------|--------|
| `[build] target` | Цель компиляции по умолчанию | `x86_64-unknown-linux-musl` для статических сборок |
| `[target.X] runner` | Как запускать бинарный файл | `"qemu-aarch64-static"` для кросс-компилированных |
| `[target.X] linker` | Какой линкер использовать | `"aarch64-linux-gnu-gcc"` |
| `[alias]` | Пользовательские подкоманды `cargo` | `xt = "test --workspace"` |
| `[env]` | Переменные окружения на этапе сборки | Пути к библиотекам, переключатели фич |
| `[net] offline` | Запрет доступа к сети | `true` для сборок в изолированной среде |

### Переменные окружения на этапе компиляции: `env!()` и `option_env!()`

Rust может встраивать переменные окружения в бинарный файл на этапе компиляции. Это полезно для строк версий, метаданных сборки и конфигурации:

```rust
// env!() паникует на этапе компиляции, если переменной нет
const VERSION: &str = env!("CARGO_PKG_VERSION"); // "0.1.0" из Cargo.toml
const PKG_NAME: &str = env!("CARGO_PKG_NAME");   // Имя крейта из Cargo.toml

// option_env!() возвращает Option<&str> и не паникует, если переменной нет
const BUILD_SHA: Option<&str> = option_env!("GIT_SHA");
const BUILD_TIME: Option<&str> = option_env!("BUILD_TIMESTAMP");

fn print_version() {
    println!("{PKG_NAME} v{VERSION}");
    if let Some(sha) = BUILD_SHA {
        println!("  коммит: {sha}");
    }
    if let Some(time) = BUILD_TIME {
        println!("  собран: {time}");
    }
}
```

Cargo автоматически задаёт множество полезных переменных окружения:

| Переменная | Значение | Сценарий использования |
|------------|----------|------------------------|
| `CARGO_PKG_VERSION` | `"1.2.3"` | Вывод версии |
| `CARGO_PKG_NAME` | `"diag_tool"` | Идентификация бинарного файла |
| `CARGO_PKG_AUTHORS` | Из `Cargo.toml` | Текст «о программе» и справки |
| `CARGO_MANIFEST_DIR` | Абсолютный путь к `Cargo.toml` | Поиск файлов тестовых данных |
| `OUT_DIR` | Каталог выходных данных сборки | Цель для генерации кода в `build.rs` |
| `TARGET` | Target triple | Логика, специфичная для платформы, в `build.rs` |

Свои переменные окружения можно задать из `build.rs`:

```rust
// build.rs
fn main() {
    println!("cargo::rustc-env=GIT_SHA={}", git_sha());
    println!("cargo::rustc-env=BUILD_TIMESTAMP={}", timestamp());
}
```

### `cfg_attr`: условные атрибуты

`cfg_attr` применяет атрибут **только тогда**, когда условие истинно. Это точечнее, чем `#[cfg()]`, который включает или исключает целые элементы:

```rust
// Derive Serialize только когда включена фича "serde":
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct DiagResult {
    pub fc: u32,
    pub passed: bool,
    pub message: String,
}
// Без фичи "serde": зависимость от serde вообще не нужна
// С фичей "serde": DiagResult можно сериализовать

// Условный атрибут для тестов:
#[cfg_attr(test, derive(PartialEq))]  // Derive PartialEq только в тестовых сборках
pub struct LargeStruct { /* ... */ }

// Атрибуты функций, зависящие от платформы:
#[cfg_attr(target_os = "linux", link_name = "ioctl")]
#[cfg_attr(target_os = "freebsd", link_name = "__ioctl")]
extern "C" fn platform_ioctl(fd: i32, request: u64) -> i32;
```

| Паттерн | Что делает |
|---------|------------|
| `#[cfg(feature = "x")]` | Включает или исключает весь элемент |
| `#[cfg_attr(feature = "x", derive(Foo))]` | Добавляет `derive(Foo)` только при включённой фиче «x» |
| `#[cfg_attr(test, allow(unused))]` | Подавляет предупреждения только в тестовых сборках |
| `#[cfg_attr(doc, doc = "...")]` | Документация видна только в `cargo doc` |

### `cargo deny` и `cargo audit`: безопасность цепочки поставок

```bash
# Установка инструментов безопасности
cargo install cargo-deny
cargo install cargo-audit

# Проверка известных уязвимостей в зависимостях
cargo audit

# Комплексные проверки: лицензии, запреты, уязвимости, источники
cargo deny check
```

Настройте `cargo deny` с помощью `deny.toml` в корне воркспейса:

```toml
# deny.toml
[advisories]
vulnerability = "deny"      # Завершать с ошибкой при известных уязвимостях
unmaintained = "warn"        # Предупреждать о неподдерживаемых крейтах

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause"]
deny = ["GPL-3.0"]          # Отвергать копилефт-лицензии

[bans]
multiple-versions = "warn"  # Предупреждать, если есть несколько версий одного крейта
deny = [
    { name = "openssl" },   # Принудительно использовать rustls вместо неё
]

[sources]
allow-git = []              # Никаких git-зависимостей в продакшене
```

| Инструмент | Назначение | Когда запускать |
|------------|------------|-----------------|
| `cargo audit` | Проверка известных CVE в зависимостях | CI-конвейер, перед выпуском |
| `cargo deny check` | Лицензии, запреты, уязвимости, источники | CI-конвейер |
| `cargo deny check licenses` | Только соответствие лицензиям | Перед открытием исходного кода |
| `cargo deny check bans` | Запрет конкретных крейтов | Соблюдение архитектурных решений |

### Doc-тесты: тесты внутри документации

Комментарии документации Rust (`///`) могут содержать блоки кода, которые **компилируются и запускаются как тесты**:

```rust
/// Разбирает код неисправности диагностики из строки.
///
/// # Примеры
///
/// ```
/// use my_crate::parse_fc;
///
/// let fc = parse_fc("FC:12345").unwrap();
/// assert_eq!(fc, 12345);
/// ```
///
/// Некорректный ввод возвращает ошибку:
///
/// ```
/// use my_crate::parse_fc;
///
/// assert!(parse_fc("not-a-fc").is_err());
/// ```
pub fn parse_fc(input: &str) -> Result<u32, ParseError> {
    input.strip_prefix("FC:")
        .ok_or(ParseError::MissingPrefix)?
        .parse()
        .map_err(ParseError::InvalidNumber)
}
```

```bash
cargo test --doc  # Запустить только doc-тесты
cargo test        # Запускает модульные, интеграционные и doc-тесты
```

**Документация уровня модуля** пишется с помощью `//!` в начале файла:

```rust
//! # Фреймворк диагностики
//!
//! Этот крейт предоставляет ядро движка выполнения диагностики.
//! Он запускает диагностические тесты, собирает результаты
//! и передаёт их в BMC через IPMI.
//!
//! ## Быстрый старт
//!
//! ```no_run
//! use diag_framework::Framework;
//!
//! let mut fw = Framework::new("config.json")?;
//! fw.run_all_tests()?;
//! ```
```

### Бенчмаркинг с criterion

> **Полное описание**: см. раздел [Бенчмаркинг с criterion](ch14-testing-and-benchmarking-patterns.md#бенчмаркинг-с-criterion) в главе 14 (Паттерны тестирования и бенчмаркинга). Там полная настройка `criterion`, примеры API и сравнительная таблица с `cargo bench`. Ниже приведена краткая справка для использования, специфичного для архитектуры.

При бенчмаркинге публичного API крейта помещайте бенчмарки в `benches/` и держите их сфокусированными на горячем пути: обычно это парсеры, сериализаторы или границы валидации:

```bash
cargo bench                  # Запустить все бенчмарки
cargo bench -- parse_config  # Запустить конкретный бенчмарк
# Результаты в target/criterion/ с HTML-отчётами
```

> **Ключевые выводы: архитектура и дизайн API**
> - Принимайте самый общий тип (`impl Into`, `impl AsRef`, `Cow`), а возвращайте самый конкретный
> - Parse, don't validate: используйте `TryFrom`, чтобы создавать типы, корректные по построению
> - `#[non_exhaustive]` на публичных перечислениях предотвращает несовместимые изменения при добавлении вариантов
> - `#[must_use]` ловит молчаливое игнорирование важных значений

> **См. также:** [гл. 10 — Обработка ошибок](ch10-error-handling-patterns.md) о проектировании типов ошибок в публичных API. [гл. 14 — Тестирование](ch14-testing-and-benchmarking-patterns.md) о тестировании публичного API крейта.

---

### Упражнение: рефакторинг API крейта ★★ (~30 минут)

Перепишите следующий «строковый» API в вариант, который использует `TryFrom`, newtype и паттерн builder:

```rust,ignore
// ДО: легко ошибиться
fn create_server(host: &str, port: &str, max_conn: &str) -> Server { ... }
```

Спроектируйте `ServerConfig` с проверенными типами `Host`, `Port` (1–65535) и `MaxConnections` (1–10000), которые отвергают некорректные значения на этапе разбора.

<details>
<summary>🔑 Решение</summary>

```rust
#[derive(Debug, Clone)]
struct Host(String);

impl TryFrom<&str> for Host {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, String> {
        if s.is_empty() { return Err("имя хоста не может быть пустым".into()); }
        if s.contains(' ') { return Err("имя хоста не может содержать пробелы".into()); }
        Ok(Host(s.to_string()))
    }
}

#[derive(Debug, Clone, Copy)]
struct Port(u16);

impl TryFrom<u16> for Port {
    type Error = String;
    fn try_from(p: u16) -> Result<Self, String> {
        if p == 0 { return Err("порт должен быть >= 1".into()); }
        Ok(Port(p))
    }
}

#[derive(Debug, Clone, Copy)]
struct MaxConnections(u32);

impl TryFrom<u32> for MaxConnections {
    type Error = String;
    fn try_from(n: u32) -> Result<Self, String> {
        if n == 0 || n > 10_000 {
            return Err(format!("max_connections должен быть в диапазоне 1–10000, получено {n}"));
        }
        Ok(MaxConnections(n))
    }
}

#[derive(Debug)]
struct ServerConfig {
    host: Host,
    port: Port,
    max_connections: MaxConnections,
}

impl ServerConfig {
    fn new(host: Host, port: Port, max_connections: MaxConnections) -> Self {
        ServerConfig { host, port, max_connections }
    }
}

fn main() {
    let config = ServerConfig::new(
        Host::try_from("localhost").unwrap(),
        Port::try_from(8080).unwrap(),
        MaxConnections::try_from(100).unwrap(),
    );
    println!("{config:?}");

    // Некорректные значения отлавливаются на этапе разбора:
    assert!(Host::try_from("").is_err());
    assert!(Port::try_from(0).is_err());
    assert!(MaxConnections::try_from(99999).is_err());
}
```

</details>

***
