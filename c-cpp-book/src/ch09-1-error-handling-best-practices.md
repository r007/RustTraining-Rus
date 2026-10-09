# Ключевые выводы по Option и Result в Rust

> **Что вы узнаете:** идиоматические паттерны обработки ошибок — безопасные альтернативы `unwrap()`, оператор `?` для распространения ошибок, пользовательские типы ошибок и когда использовать `anyhow`, а когда `thiserror` в продакшн-коде.

- ```Option``` и ```Result``` — неотъемлемая часть идиоматичного Rust
- **Безопасные альтернативы `unwrap()`**:
```rust
// Безопасные альтернативы для Option<T>
let value = opt.unwrap_or(default);              // Запасное значение
let value = opt.unwrap_or_else(|| compute());    // Ленивое вычисление запасного значения
let value = opt.unwrap_or_default();             // Использовать реализацию трейта Default
let value = opt.expect("descriptive message");   // Только когда panic допустим

// Безопасные альтернативы для Result<T, E>
let value = result.unwrap_or(fallback);          // Игнорировать ошибку, использовать запасное значение
let value = result.unwrap_or_else(|e| handle(e)); // Обработать ошибку и вернуть запасное значение
let value = result.unwrap_or_default();          // Использовать трейт Default
```
- **Сопоставление с образцом для явного контроля**:
```rust
match some_option {
    Some(value) => println!("Got: {}", value),
    None => println!("No value found"),
}

match some_result {
    Ok(value) => process(value),
    Err(error) => log_error(error),
}
```
- **Используйте оператор `?` для распространения ошибок**: досрочный выход и передача ошибок наверх
```rust
fn process_file(path: &str) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(path)?; // Автоматически возвращает ошибку
    Ok(content.to_uppercase())
}
```
- **Методы преобразования**:
    - `map()`: преобразует успешное значение `Ok(T)` -> `Ok(U)` или `Some(T)` -> `Some(U)`
    - `map_err()`: преобразует тип ошибки `Err(E)` -> `Err(F)`
    - `and_then()`: объединяет цепочкой операции, которые могут завершиться неудачей
- **Используйте в собственных API**: предпочитайте `Result<T, E>` исключениям или кодам ошибок
- **Ссылки**: [документация Option](https://doc.rust-lang.org/std/option/enum.Option.html) | [документация Result](https://doc.rust-lang.org/std/result/enum.Result.html)

# Частые ошибки Rust и советы по отладке
- **Проблемы с заимствованием**: самая распространённая ошибка новичков
    - "cannot borrow as mutable" -> одновременно допускается только одна изменяемая ссылка
    - "borrowed value does not live long enough" -> ссылка переживает данные, на которые указывает
    - **Решение**: используйте области видимости `{}`, чтобы ограничить время жизни ссылок, или клонируйте данные при необходимости
- **Отсутствующие реализации трейтов**: ошибки "method not found"
    - **Решение**: добавьте `#[derive(Debug, Clone, PartialEq)]` для распространённых трейтов
    - Используйте `cargo check`, чтобы получать более понятные сообщения об ошибках, чем при `cargo run`
- **Переполнение целых в отладочной сборке**: Rust вызывает panic при переполнении
    - **Решение**: используйте `wrapping_add()`, `saturating_add()` или `checked_add()` для явного поведения
- **Путаница между String и &str**: разные типы для разных задач
    - Используйте `&str` для срезов строк (заимствованных), `String` — для владеющих строк
    - **Решение**: используйте `.to_string()` или `String::from()`, чтобы преобразовать `&str` в `String`
- **Борьба с проверкой заимствований**: не пытайтесь её перехитрить
    - **Решение**: перестройте код так, чтобы он работал с правилами владения, а не против них
    - Для сложных случаев совместного использования рассмотрите `Rc<RefCell<T>>` (но умеренно)

## Примеры обработки ошибок: хорошо и плохо
```rust
// [ОШИБКА] ПЛОХО: может неожиданно вызвать panic
fn bad_config_reader() -> String {
    let config = std::env::var("CONFIG_FILE").unwrap(); // Panic, если не задано!
    std::fs::read_to_string(config).unwrap()           // Panic, если файла нет!
}

// [OK] ХОРОШО: обрабатывает ошибки корректно
fn good_config_reader() -> Result<String, ConfigError> {
    let config_path = std::env::var("CONFIG_FILE")
        .unwrap_or_else(|_| "default.conf".to_string()); // Запасной вариант по умолчанию
    
    let content = std::fs::read_to_string(config_path)
        .map_err(ConfigError::FileRead)?;                // Преобразовать и передать ошибку дальше
    
    Ok(content)
}

// [OK] ЕЩЁ ЛУЧШЕ: с правильными типами ошибок
use thiserror::Error;

#[derive(Error, Debug)]
enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(#[from] std::io::Error),
    
    #[error("Invalid configuration: {message}")]
    Invalid { message: String },
}
```

Разберём, что здесь происходит. У `ConfigError` всего **два варианта** — один для ошибок ввода-вывода и один для ошибок валидации. Это правильная отправная точка для большинства модулей:

| Вариант `ConfigError` | Содержит | Создаётся |
|----------------------|-------|-----------|
| `FileRead(io::Error)` | Исходную ошибку ввода-вывода | `#[from]` автоматически преобразует через `?` |
| `Invalid { message }` | Понятное человеку объяснение | Ваш код валидации |

Теперь можно писать функции, которые возвращают `Result<T, ConfigError>`:

```rust
fn read_config(path: &str) -> Result<String, ConfigError> {
    let content = std::fs::read_to_string(path)?;  // io::Error → ConfigError::FileRead
    if content.is_empty() {
        return Err(ConfigError::Invalid {
            message: "config file is empty".to_string(),
        });
    }
    Ok(content)
}
```

> **🟢 Контрольная точка для самостоятельного изучения:** прежде чем продолжить, убедитесь, что можете ответить:
> 1. Почему `?` после вызова `read_to_string` работает? (Потому что `#[from]` генерирует `impl From<io::Error> for ConfigError`)
> 2. Что произойдёт, если добавить третий вариант `MissingKey(String)` — какой код придётся менять? (Просто добавить вариант; существующий код по-прежнему компилируется)

## Типы ошибок на уровне крейта и псевдонимы Result

По мере роста проекта за пределы одного файла вы будете объединять ошибки отдельных модулей в **тип ошибок на уровне крейта**. Это стандартный паттерн в продакшн-коде на Rust. Построим его на основе `ConfigError` выше.

В реальных проектах на Rust каждый крейт (или значимый модуль) определяет собственное перечисление `Error`
и псевдоним типа `Result`. Это идиоматичный паттерн — аналогичный тому, как в C++
вы определяли бы иерархию исключений для библиотеки и `using Result = std::expected<T, Error>`.

### Шаблон

```rust
// src/error.rs  (или в начале lib.rs)
use thiserror::Error;

/// Все ошибки, которые может породить этот крейт.
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),          // автоматически преобразуется через From

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),     // автоматически преобразуется через From

    #[error("Invalid sensor id: {0}")]
    InvalidSensor(u32),                  // специфичный для предметной области вариант

    #[error("Timeout after {ms} ms")]
    Timeout { ms: u64 },
}

/// Псевдоним Result на уровне всего крейта — экономит набор текста во всём крейте.
pub type Result<T> = core::result::Result<T, Error>;
```

### Как это упрощает каждую функцию

Без псевдонима вы бы писали:

```rust
// Многословно — тип ошибки повторяется везде
fn read_sensor(id: u32) -> Result<f64, crate::Error> { ... }
fn parse_config(path: &str) -> Result<Config, crate::Error> { ... }
```

С псевдонимом:

```rust
// Чисто — просто `Result<T>`
use crate::{Error, Result};

fn read_sensor(id: u32) -> Result<f64> {
    if id > 128 {
        return Err(Error::InvalidSensor(id));
    }
    let raw = std::fs::read_to_string(format!("/dev/sensor/{id}"))?; // io::Error → Error::Io
    let value: f64 = raw.trim().parse()
        .map_err(|_| Error::InvalidSensor(id))?;
    Ok(value)
}
```

Атрибут `#[from]` на `Io` генерирует эту реализацию бесплатно:

```rust
// Генерируется автоматически атрибутом #[from] из thiserror
impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        Error::Io(source)
    }
}
```

Именно это делает `?` работающим: когда функция возвращает `std::io::Error`, а ваша функция
возвращает `Result<T>` (ваш псевдоним), компилятор вызывает `From::from()`, чтобы автоматически
преобразовать ошибку.

### Композиция ошибок отдельных модулей

В больших крейтах ошибки разделяют по модулям, а затем объединяют в корне крейта:

```rust
// src/config/error.rs
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Missing key: {0}")]
    MissingKey(String),
    #[error("Invalid value for '{key}': {reason}")]
    InvalidValue { key: String, reason: String },
}

// src/error.rs  (уровень крейта)
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]               // делегирует Display внутренней ошибке
    Config(#[from] crate::config::ConfigError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
pub type Result<T> = core::result::Result<T, Error>;
```

Вызывающий код по-прежнему может сопоставлять конкретные ошибки конфигурации:

```rust
match result {
    Err(Error::Config(ConfigError::MissingKey(k))) => eprintln!("Add '{k}' to config"),
    Err(e) => eprintln!("Other error: {e}"),
    Ok(v) => use_value(v),
}
```

### Сравнение с C++

| Понятие | C++ | Rust |
|---------|-----|------|
| Иерархия ошибок | `class AppError : public std::runtime_error` | `#[derive(thiserror::Error)] enum Error { ... }` |
| Возврат ошибки | `std::expected<T, Error>` или `throw` | `fn foo() -> Result<T>` |
| Преобразование ошибки | Ручной `try/catch` + повторный `throw` | `#[from]` + `?` — без шаблонного кода |
| Псевдоним Result | `template<class T> using Result = std::expected<T, Error>;` | `pub type Result<T> = core::result::Result<T, Error>;` |
| Сообщение об ошибке | Переопределить `what()` | `#[error("...")]` — компилируется в реализацию `Display` |

