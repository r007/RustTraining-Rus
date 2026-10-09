# Крейты и модули в Rust

> **Что вы узнаете:** как Rust организует код в модули и крейты — видимость по умолчанию (приватная), модификаторы `pub`, рабочие пространства и экосистему `crates.io`. Заменяет заголовочные файлы C/C++, `#include` и управление зависимостями в CMake.

- Модули — базовая организационная единица кода внутри крейтов
    - Каждый исходный файл (.rs) — это отдельный модуль, и он может создавать вложенные модули с помощью ключевого слова ```mod```.
    - Все типы внутри (под)модуля по умолчанию **приватные** и не видны извне в пределах того же крейта, если явно не помечены как ```pub``` (публичные). Область действия ```pub``` можно дополнительно ограничить через ```pub(crate)``` и т. д.
    - Даже публичный тип автоматически не становится видимым в другом модуле, если он не импортирован с помощью ключевого слова ```use```. Дочерние подмодули могут обращаться к типам родительской области через ```use super::```
    - Исходные файлы (.rs) **не** включаются в крейт автоматически, если явно не перечислены в ```main.rs``` (исполняемый файл) или ```lib.rs```

# Упражнение: модули и функции
- Посмотрим, как изменить наш [hello world](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=522d86dbb8c4af71ff2ec081fb76aee7), чтобы он вызывал другую функцию
    - Как уже говорилось, функции определяются ключевым словом ```fn```. Ключевой элемент ```->``` объявляет, что функция возвращает значение (по умолчанию — void) типа ```u32``` (беззнаковое 32-битное целое)
    - Функции ограничены областью модуля, то есть две функции с одинаковым именем в разных модулях не конфликтуют
        - Ограничение модулем распространяется на все типы (например, ```struct foo``` в ```mod a { struct foo; }``` — это отдельный тип (```a::foo```), отличный от ```mod b { struct foo; }``` (```b::foo```))

**Стартовый код** — дополните функции:
```rust
mod math {
    // TODO: реализуйте pub fn add(a: u32, b: u32) -> u32
}

fn greet(name: &str) -> String {
    // TODO: верните "Hello, <name>! The secret number is <math::add(21,21)>"
    todo!()
}

fn main() {
    println!("{}", greet("Rustacean"));
}
```

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
mod math {
    pub fn add(a: u32, b: u32) -> u32 {
        a + b
    }
}

fn greet(name: &str) -> String {
    format!("Hello, {}! The secret number is {}", name, math::add(21, 21))
}

fn main() {
    println!("{}", greet("Rustacean"));
}
// Вывод: Hello, Rustacean! The secret number is 42
```

</details>


## Рабочие пространства и крейты (пакеты)

- Любой серьёзный проект на Rust должен использовать рабочие пространства (workspaces) для организации составных крейтов
    - Рабочее пространство — это просто набор локальных крейтов, которые используются для сборки целевых бинарных файлов. `Cargo.toml` в корне рабочего пространства должен содержать ссылки на составные пакеты (крейты)

```toml
[workspace]
resolver = "2"
members = ["package1", "package2"]
```

```text
workspace_root/
|-- Cargo.toml      # Конфигурация рабочего пространства
|-- package1/
|   |-- Cargo.toml  # Конфигурация пакета 1
|   `-- src/
|       `-- lib.rs  # Исходный код пакета 1
|-- package2/
|   |-- Cargo.toml  # Конфигурация пакета 2
|   `-- src/
|       `-- main.rs # Исходный код пакета 2
```

---
## Упражнение: использование рабочих пространств и зависимостей пакетов
- Создадим простой пакет и используем его из нашей программы ```hello world```
- Создайте каталог рабочего пространства
```bash
mkdir workspace
cd workspace
```
- Создайте файл Cargo.toml и добавьте в него следующее. Это создаёт пустое рабочее пространство
```toml
[workspace]
resolver = "2"
members = []
```
- Добавьте пакеты (```cargo new --lib``` создаёт библиотеку вместо исполняемого файла)
```bash
cargo new hello
cargo new --lib hellolib
```

## Упражнение: использование рабочих пространств и зависимостей пакетов
- Посмотрите сгенерированные Cargo.toml в ```hello``` и ```hellolib```. Обратите внимание, что оба добавлены в Cargo.toml верхнего уровня
- Наличие ```lib.rs``` в ```hellolib``` означает, что это библиотечный пакет (см. https://doc.rust-lang.org/cargo/reference/cargo-targets.html, там описаны варианты настройки)
- Добавляем зависимость от ```hellolib``` в Cargo.toml для ```hello```
```toml
[dependencies]
hellolib = {path = "../hellolib"}
```
- Используем ```add()``` из ```hellolib```
```rust
fn main() {
    println!("Hello, world! {}", hellolib::add(21, 21));
}
```

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

Полная настройка рабочего пространства:

```bash
# Команды терминала
mkdir workspace && cd workspace

# Создаём Cargo.toml рабочего пространства
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = ["hello", "hellolib"]
EOF

cargo new hello
cargo new --lib hellolib
```

```toml
# hello/Cargo.toml — добавляем зависимость
[dependencies]
hellolib = {path = "../hellolib"}
```

```rust
// hellolib/src/lib.rs — add() уже есть после cargo new --lib
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
```

```rust,ignore
// hello/src/main.rs
fn main() {
    println!("Hello, world! {}", hellolib::add(21, 21));
}
// Вывод: Hello, world! 42
```

</details>

# Использование крейтов сообщества из crates.io
- В Rust богатая экосистема крейтов сообщества (см. https://crates.io/)
    - Философия Rust — держать стандартную библиотеку компактной и выносить функциональность в крейты сообщества
    - Жёсткого правила насчёт использования крейтов сообщества нет, но ориентир такой: крейт должен быть достаточно зрелым (это видно по номеру версии) и активно поддерживаться. Если сомневаетесь в крейте, уточните во внутренних источниках
- Каждый крейт, опубликованный на ```crates.io```, имеет мажорную и минорную версии
    - Ожидается, что крейты соблюдают рекомендации ```SemVer`` по мажорной и минорной версиям: https://doc.rust-lang.org/cargo/reference/semver.html
    - Вкратце: внутри одной минорной версии не должно быть несовместимых изменений. Например, v0.11 должна быть совместима с v0.15 (а вот v0.20 может содержать несовместимые изменения)

# Зависимости крейтов и SemVer
- Крейты могут объявлять зависимости от конкретной версии крейта, конкретной минорной или мажорной версии либо быть безразличными к версии. Следующие примеры показывают записи в ```Cargo.toml``` для объявления зависимости от крейта ```rand```
- Не ниже ```0.10.0```, но подходит всё, что ```< 0.11.0```
```toml
[dependencies]
rand = { version = "0.10.0"}
```
- Только ```0.10.0```, и ничего другого
```toml
[dependencies]
rand = { version = "=0.10.0"}
```
- Версия не важна; ```cargo``` выберет последнюю
```toml
[dependencies]
rand = { version = "*"}
```
- Справка: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
----
# Упражнение: использование крейта rand
- Измените пример ```helloworld```, чтобы он выводил случайное число
- Используйте ```cargo add rand``` для добавления зависимости
- Используйте ```https://docs.rs/rand/latest/rand/``` как справочник по API

**Стартовый код** — добавьте это в `main.rs` после выполнения `cargo add rand`:
```rust,ignore
use rand::RngExt;

fn main() {
    let mut rng = rand::rng();
    // TODO: Сгенерируйте и выведите случайное u32 в диапазоне 1..=100
    // TODO: Сгенерируйте и выведите случайный bool
    // TODO: Сгенерируйте и выведите случайный f64
}
```

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
use rand::RngExt;

fn main() {
    let mut rng = rand::rng();
    let n: u32 = rng.random_range(1..=100);
    println!("Random number (1-100): {n}");

    // Генерируем случайный булев
    let b: bool = rng.random();
    println!("Random bool: {b}");

    // Генерируем случайное число с плавающей точкой от 0.0 до 1.0
    let f: f64 = rng.random();
    println!("Random float: {f:.4}");
}
```

</details>

# Cargo.toml и Cargo.lock
- Как уже говорилось, Cargo.lock автоматически генерируется из Cargo.toml
    - Основная идея Cargo.lock — обеспечить воспроизводимые сборки. Например, если в ```Cargo.toml``` указана версия ```0.10.0```, cargo вправе выбрать любую версию ```< 0.11.0```
    - Cargo.lock содержит *конкретную* версию крейта rand, которая использовалась при сборке.
    - Рекомендуется включать ```Cargo.lock``` в git-репозиторий, чтобы сборки были воспроизводимыми

## Функция cargo test
- Модульные тесты Rust по соглашению находятся в том же исходном файле и обычно группируются в отдельный модуль
    - Тестовый код никогда не попадает в итоговый бинарный файл. Это возможно благодаря функции ```cfg``` (конфигурация). Конфигурации полезны, например, для платформо-специфичного кода (```Linux``` и ```Windows```)
    - Тесты запускаются командой ```cargo test```. Справка: https://doc.rust-lang.org/reference/conditional-compilation.html

```rust
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
// Будет включён только во время тестирования
#[cfg(test)]
mod tests {
    use super::*; // Делает видимыми все типы родительской области
    #[test]
    fn it_works() {
        let result = add(2, 2); // Или то же самое: super::add(2, 2);
        assert_eq!(result, 4);
    }
}
```

# Другие возможности Cargo
- У ```cargo``` есть и другие полезные возможности, в том числе:
    - ```cargo clippy``` — отличный способ линтинга кода на Rust. Как правило, предупреждения стоит исправлять (подавлять их имеет смысл лишь в действительно обоснованных случаях)
    - ```cargo format``` запускает инструмент ```rustfmt``` для форматирования исходного кода. Использование этого инструмента обеспечивает единообразное форматирование кода в репозитории и кладёт конец спорам о стиле
    - ```cargo doc``` генерирует документацию из комментариев в стиле ```///```. Документация для всех крейтов на ```crates.io``` создана именно этим способом

### Профили сборки: управление оптимизацией

В C вы передаёте `-O0`, `-O2`, `-Os`, `-flto` в `gcc`/`clang`. В Rust профили сборки настраиваются в `Cargo.toml`:

```toml
# Cargo.toml — настройка профилей сборки

[profile.dev]
opt-level = 0          # Без оптимизаций (быстрая компиляция, как -O0)
debug = true           # Полные отладочные символы (как -g)

[profile.release]
opt-level = 3          # Максимальная оптимизация (как -O3)
lto = "fat"            # Оптимизация на этапе компоновки (как -flto)
strip = true           # Удалить символы (как команда strip)
codegen-units = 1      # Один блок генерации кода — компиляция медленнее, оптимизация лучше
panic = "abort"        # Без таблиц раскрутки стека (меньше бинарный файл)
```

| Флаг C/GCC | Ключ в Cargo.toml | Значения |
|------------|---------------|--------|
| `-O0` / `-O2` / `-O3` | `opt-level` | `0`, `1`, `2`, `3`, `"s"`, `"z"` |
| `-flto` | `lto` | `false`, `"thin"`, `"fat"` |
| `-g` / без `-g` | `debug` | `true`, `false`, `"line-tables-only"` |
| команда `strip` | `strip` | `"none"`, `"debuginfo"`, `"symbols"`, `true`/`false` |
| — | `codegen-units` | `1` = лучшая оптимизация, самая медленная компиляция |

```bash
cargo build              # Использует [profile.dev]
cargo build --release    # Использует [profile.release]
```

### Скрипты сборки (`build.rs`): подключение C-библиотек

В C для подключения библиотек и запуска генерации кода используют Makefile или CMake.
В Rust для этого в корне крейта есть файл `build.rs`:

```rust
// build.rs — запускается перед компиляцией крейта

fn main() {
    // Подключаем системную C-библиотеку (как -lbmc_ipmi в gcc)
    println!("cargo::rustc-link-lib=bmc_ipmi");

    // Где искать библиотеку (как -L/usr/lib/bmc)
    println!("cargo::rustc-link-search=/usr/lib/bmc");

    // Пересобрать, если изменится заголовочный файл C
    println!("cargo::rerun-if-changed=wrapper.h");
}
```

Можно даже компилировать исходники на C прямо из крейта Rust:

```toml
# Cargo.toml
[build-dependencies]
cc = "1"  # Интеграция с C-компилятором
```

```rust
// build.rs
fn main() {
    cc::Build::new()
        .file("src/c_helpers/ipmi_raw.c")
        .include("/usr/include/bmc")
        .compile("ipmi_raw");   // Создаёт libipmi_raw.a, который подключается автоматически
    println!("cargo::rerun-if-changed=src/c_helpers/ipmi_raw.c");
}
```

| C / Make / CMake | Rust `build.rs` |
|-----------------|-----------------|
| `-lfoo` | `println!("cargo::rustc-link-lib=foo")` |
| `-L/path` | `println!("cargo::rustc-link-search=/path")` |
| Компиляция исходников C | `cc::Build::new().file("foo.c").compile("foo")` |
| Генерация кода | Записать файлы в `$OUT_DIR`, затем `include!()` |

### Кросс-компиляция

В C для кросс-компиляции нужно установить отдельный тулчейн (`arm-linux-gnueabihf-gcc`)
и настроить Make/CMake. В Rust:

```bash
# Устанавливаем цель для кросс-компиляции
rustup target add aarch64-unknown-linux-gnu

# Кросс-компиляция
cargo build --target aarch64-unknown-linux-gnu --release
```

Компоновщик указывается в `.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

| Кросс-компиляция в C | Аналог в Rust |
|-----------------|-----------------|
| `apt install gcc-aarch64-linux-gnu` | `rustup target add aarch64-unknown-linux-gnu` + установка компоновщика |
| `CC=aarch64-linux-gnu-gcc make` | `.cargo/config.toml`, `[target.X] linker = "..."` |
| `#ifdef __aarch64__` | `#[cfg(target_arch = "aarch64")]` |
| Отдельные цели Makefile | `cargo build --target ...` |

### Флаги функций: условная компиляция

В C для условной компиляции используют `#ifdef` и `-DFOO`. В Rust — флаги функций,
объявленные в `Cargo.toml`:

```toml
# Cargo.toml
[features]
default = ["json"]         # Включено по умолчанию
json = ["dep:serde_json"]  # Необязательная зависимость
verbose = []               # Флаг без зависимости
gpu = ["dep:cuda-sys"]     # Необязательная поддержка GPU
```

```rust
// Код, зависящий от флагов:
#[cfg(feature = "json")]
pub fn parse_config(data: &str) -> Result<Config, Error> {
    serde_json::from_str(data).map_err(Error::from)
}

#[cfg(feature = "verbose")]
macro_rules! verbose {
    ($($arg:tt)*) => { eprintln!("[VERBOSE] {}", format!($($arg)*)); }
}
#[cfg(not(feature = "verbose"))]
macro_rules! verbose {
    ($($arg:tt)*) => {}; // Компилируется в ничего
}
```

| Препроцессор C | Флаги функций Rust |
|---------------|-------------------|
| `gcc -DDEBUG` | `cargo build --features verbose` |
| `#ifdef DEBUG` | `#[cfg(feature = "verbose")]` |
| `#define MAX 100` | `const MAX: u32 = 100;` |
| `#ifdef __linux__` | `#[cfg(target_os = "linux")]` |

### Интеграционные тесты и модульные тесты

Модульные тесты располагаются рядом с кодом и помечаются `#[cfg(test)]`. **Интеграционные тесты** находятся в каталоге
`tests/` и проверяют только **публичный API** вашего крейта:

```rust
// tests/smoke_test.rs — без #[cfg(test)]
use my_crate::parse_config;

#[test]
fn parse_valid_config() {
    let config = parse_config("test_data/valid.json").unwrap();
    assert_eq!(config.max_retries, 5);
}
```

| Аспект | Модульные тесты (`#[cfg(test)]`) | Интеграционные тесты (`tests/`) |
|--------|----------------------------|------------------------------|
| Расположение | Тот же файл, что и код | Отдельный каталог `tests/` |
| Доступ | Приватные и публичные элементы | **Только публичный API** |
| Команда запуска | `cargo test` | `cargo test --test smoke_test` |


### Паттерны и стратегии тестирования

Команды C-прошивок обычно пишут тесты на CUnit, CMocka или собственных фреймворках, с большим количеством шаблонного кода. Встроенный тестовый харнесс Rust гораздо мощнее. В этом разделе рассматриваются паттерны, которые понадобятся в продакшн-коде.

#### `#[should_panic]` — тестирование ожидаемых сбоев

```rust
// Проверяем, что определённые условия приводят к panic (как срабатывание assert в C)
#[test]
#[should_panic(expected = "index out of bounds")]
fn test_bounds_check() {
    let v = vec![1, 2, 3];
    let _ = v[10];  // Должно вызвать panic
}

#[test]
#[should_panic(expected = "temperature exceeds safe limit")]
fn test_thermal_shutdown() {
    fn check_temperature(celsius: f64) {
        if celsius > 105.0 {
            panic!("temperature exceeds safe limit: {celsius}°C");
        }
    }
    check_temperature(110.0);
}
```

#### `#[ignore]` — медленные тесты и тесты, зависящие от оборудования

```rust
// Помечаем тесты, которым нужны особые условия (как #ifdef HARDWARE_TEST в C)
#[test]
#[ignore = "requires GPU hardware"]
fn test_gpu_ecc_scrub() {
    // Этот тест запускается только на машинах с GPU
    // Запуск: cargo test -- --ignored
    // Запуск: cargo test -- --include-ignored  (запускает ВСЕ тесты)
}
```

#### Тесты, возвращающие Result (вместо цепочек unwrap)

```rust
// Вместо множества вызовов unwrap(), которые скрывают реальный сбой:
#[test]
fn test_config_parsing() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"hostname": "node-01", "port": 8080}"#;
    let config: ServerConfig = serde_json::from_str(json)?;  // ? вместо unwrap()
    assert_eq!(config.hostname, "node-01");
    assert_eq!(config.port, 8080);
    Ok(())  // Тест проходит, если мы дошли сюда без ошибок
}
```

#### Фикстуры тестов с функциями-билдерами

В C используют функции `setUp()`/`tearDown()`. В Rust — вспомогательные функции и `Drop`:

```rust
struct TestFixture {
    temp_dir: std::path::PathBuf,
    config: Config,
}

impl TestFixture {
    fn new() -> Self {
        let temp_dir = std::env::temp_dir().join(format!("test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config = Config {
            log_dir: temp_dir.clone(),
            max_retries: 3,
            ..Default::default()
        };
        Self { temp_dir, config }
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Автоматическая очистка — как tearDown() в C, но её нельзя забыть вызвать
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn test_with_fixture() {
    let fixture = TestFixture::new();
    // Используем fixture.config, fixture.temp_dir...
    assert!(fixture.temp_dir.exists());
    // fixture автоматически уничтожается здесь → выполняется очистка
}
```

#### Мокирование трейтов для аппаратных интерфейсов

В C мокирование оборудования требует хитростей препроцессора или подмены указателей на функции.
В Rust трейты делают это естественным:

```rust
// Продакшн-трейт для связи по IPMI
trait IpmiTransport {
    fn send_command(&self, cmd: u8, data: &[u8]) -> Result<Vec<u8>, String>;
}

// Реальная реализация (используется в продакшне)
struct RealIpmi { /* Параметры подключения к BMC */ }
impl IpmiTransport for RealIpmi {
    fn send_command(&self, cmd: u8, data: &[u8]) -> Result<Vec<u8>, String> {
        // Реально обращается к аппаратуре BMC
        todo!("Real IPMI call")
    }
}

// Мок-реализация (используется в тестах)
struct MockIpmi {
    responses: std::collections::HashMap<u8, Vec<u8>>,
}
impl IpmiTransport for MockIpmi {
    fn send_command(&self, cmd: u8, _data: &[u8]) -> Result<Vec<u8>, String> {
        self.responses.get(&cmd)
            .cloned()
            .ok_or_else(|| format!("No mock response for cmd 0x{cmd:02x}"))
    }
}

// Обобщённая функция, которая работает и с реальной, и с мок-реализацией
fn read_sensor_temperature(transport: &dyn IpmiTransport) -> Result<f64, String> {
    let response = transport.send_command(0x2D, &[])?;
    if response.len() < 2 {
        return Err("Response too short".into());
    }
    Ok(response[0] as f64 + (response[1] as f64 / 256.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_reading() {
        let mut mock = MockIpmi { responses: std::collections::HashMap::new() };
        mock.responses.insert(0x2D, vec![72, 128]); // 72.5°C

        let temp = read_sensor_temperature(&mock).unwrap();
        assert!((temp - 72.5).abs() < 0.01);
    }

    #[test]
    fn test_short_response() {
        let mock = MockIpmi { responses: std::collections::HashMap::new() };
        // Ответ не настроен → ошибка
        assert!(read_sensor_temperature(&mock).is_err());
    }
}
```

#### Property-based тестирование с `proptest`

Вместо проверки конкретных значений тестируйте **свойства**, которые должны выполняться всегда:

```rust
// Cargo.toml: [dev-dependencies] proptest = "1"
use proptest::prelude::*;

fn parse_sensor_id(s: &str) -> Option<u32> {
    s.strip_prefix("sensor_")?.parse().ok()
}

fn format_sensor_id(id: u32) -> String {
    format!("sensor_{id}")
}

proptest! {
    #[test]
    fn roundtrip_sensor_id(id in 0u32..10000) {
        // Свойство: форматирование, а затем разбор должны вернуть исходное значение
        let formatted = format_sensor_id(id);
        let parsed = parse_sensor_id(&formatted);
        prop_assert_eq!(parsed, Some(id));
    }

    #[test]
    fn parse_rejects_garbage(s in "[^s].*") {
        // Свойство: строки, не начинающиеся с 's', никогда не должны разбираться
        let result = parse_sensor_id(&s);
        prop_assert!(result.is_none());
    }
}
```

#### Сравнение тестирования в C и Rust

| Тестирование в C | Аналог в Rust |
|-----------|----------------|
| `CUnit`, `CMocka`, собственный фреймворк | Встроенный `#[test]` + `cargo test` |
| `setUp()` / `tearDown()` | Функция-билдер + трейт `Drop` |
| Mock-функции через `#ifdef TEST` | Внедрение зависимостей через трейты |
| `assert(x == y)` | `assert_eq!(x, y)` с автоматическим выводом различий |
| Отдельный исполняемый файл для тестов | Тот же бинарный файл, условная компиляция через `#[cfg(test)]` |
| `valgrind --leak-check=full ./test` | `cargo test` (безопасно по памяти по умолчанию) + `cargo miri test` |
| Покрытие кода: `gcov` / `lcov` | `cargo tarpaulin` или `cargo llvm-cov` |
| Обнаружение тестов: ручная регистрация | Автоматическое — любая функция `#[test]` обнаруживается |



