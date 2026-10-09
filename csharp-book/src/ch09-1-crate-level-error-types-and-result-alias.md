## Типы ошибок уровня крейта и псевдонимы Result

> **Что вы узнаете:** производственный паттерн определения перечисления ошибок для каждого крейта с помощью `thiserror`,
> создание псевдонима типа `Result<T>`, а также когда выбирать `thiserror` (библиотеки) или `anyhow` (приложения).
>
> **Сложность:** 🟡 Средний

Ключевой паттерн для производственного кода на Rust: определить перечисление ошибок для каждого крейта и псевдоним типа `Result`, чтобы убрать шаблонный код.

### Паттерн
```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },
}

/// Псевдоним Result для всего крейта — каждая функция возвращает именно его
pub type Result<T> = std::result::Result<T, AppError>;
```

### Использование во всём крейте
```rust
use crate::error::{AppError, Result};

// Предполагается, что пул соединений с базой доступен, например:
// async fn get_user(pool: &PgPool, id: Uuid) -> Result<User>
// Здесь показан паттерн, где `pool` — сокращение.
pub async fn get_user(id: Uuid) -> Result<User> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_optional(&pool)
        .await?;  // sqlx::Error → AppError::Database через #[from]

    user.ok_or_else(|| AppError::NotFound {
        entity: "User".into(),
        id: id.to_string(),
    })
}

pub async fn create_user(req: CreateUserRequest) -> Result<User> {
    if req.name.trim().is_empty() {
        return Err(AppError::Validation {
            message: "Name cannot be empty".into(),
        });
    }
    // ...
}
```

### Сравнение с C#
```csharp
// Аналогичный паттерн в C#
public class AppException : Exception
{
    public string ErrorCode { get; }
    public AppException(string code, string message) : base(message)
    {
        ErrorCode = code;
    }
}

// Но в C# вызывающий код не знает, какие исключения ожидать!
// В Rust тип ошибки указан в сигнатуре функции.
```

### Почему это важно
- **`thiserror`** автоматически генерирует реализации `Display` и `Error`
- **`#[from]`** позволяет оператору `?` автоматически преобразовывать ошибки библиотек
- Псевдоним `Result<T>` делает каждую сигнатуру функции чистой: `fn foo() -> Result<Bar>`
- **В отличие от исключений C#** вызывающий код видит все возможные варианты ошибок в типе


### thiserror против anyhow: что выбрать

Обработку ошибок в Rust определяют два крейта. Выбор между ними — первое решение, которое вам предстоит принять:

| | `thiserror` | `anyhow` |
|---|---|---|
| **Назначение** | Определение структурированных типов ошибок для **библиотек** | Быстрая обработка ошибок для **приложений** |
| **Результат** | Собственное перечисление, которым вы управляете | Непрозрачная обёртка `anyhow::Error` |
| **Что видит вызывающий код** | Все варианты ошибок в типе | Просто `anyhow::Error` — непрозрачный |
| **Лучше всего для** | Библиотечных крейтов, API, любого кода с потребителями | Бинарников, скриптов, прототипов, CLI-утилит |
| **Приведение типов** | `match` по вариантам напрямую | `error.downcast_ref::<MyError>()` |

```rust
// thiserror — для БИБЛИОТЕК (вызывающему коду нужно сопоставлять варианты ошибок)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("File not found: {path}")]
    NotFound { path: String },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn read_config(path: &str) -> Result<String, StorageError> {
    std::fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => StorageError::NotFound { path: path.into() },
        std::io::ErrorKind::PermissionDenied => StorageError::PermissionDenied(path.into()),
        _ => StorageError::Io(e),
    })
}
```

```rust
// anyhow — для ПРИЛОЖЕНИЙ (просто пробрасываем ошибки, не определяя типов)
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = std::fs::read_to_string("config.toml")
        .context("Failed to read config file")?;

    let port: u16 = config.parse()
        .context("Failed to parse port number")?;

    println!("Listening on port {port}");
    Ok(())
}
// anyhow::Result<T> = Result<T, anyhow::Error>
// .context() добавляет понятный для человека контекст к любой ошибке
```

```csharp
// Сравнение с C#:
// thiserror ≈ определение собственных классов исключений с конкретными свойствами
// anyhow ≈ перехват Exception и оборачивание с сообщением:
//   throw new InvalidOperationException("Failed to read config", ex);
```

**Правило**: если ваш код — **библиотека** (его вызывает другой код), используйте `thiserror`. Если ваш код — **приложение** (финальный бинарник), используйте `anyhow`. Многие проекты используют оба: `thiserror` для публичного API библиотечного крейта, `anyhow` в бинарнике в `main()`.

### Паттерны восстановления после ошибок

Разработчики C# привыкли к блокам `try/catch`, которые восстанавливаются после конкретных исключений. В Rust для того же используются комбинаторы над `Result`:

```rust
use std::fs;

// Паттерн 1: восстановление через значение по умолчанию
let config = fs::read_to_string("config.toml")
    .unwrap_or_else(|_| String::from("port = 8080"));  // значение по умолчанию, если файла нет

// Паттерн 2: восстанавливаемся после конкретных ошибок, остальные пробрасываем
fn read_or_create(path: &str) -> Result<String, std::io::Error> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let default = String::from("# new file");
            fs::write(path, &default)?;
            Ok(default)
        }
        Err(e) => Err(e),  // пробрасываем ошибки доступа и прочие
    }
}

// Паттерн 3: добавляем контекст перед распространением
use anyhow::Context;

fn load_config() -> anyhow::Result<Config> {
    let text = fs::read_to_string("config.toml")
        .context("Failed to read config.toml")?;
    let config: Config = toml::from_str(&text)
        .context("Failed to parse config.toml")?;
    Ok(config)
}

// Паттерн 4: преобразуем ошибки в доменный тип
fn parse_port(s: &str) -> Result<u16, AppError> {
    s.parse::<u16>()
        .map_err(|_| AppError::Validation {
            message: format!("Invalid port: {s}"),
        })
}
```

```csharp
// Аналоги в C#:
try { config = File.ReadAllText("config.toml"); }
catch (FileNotFoundException) { config = "port = 8080"; }  // Паттерн 1

try { /* ... */ }
catch (FileNotFoundException) { /* создать файл */ }        // Паттерн 2
catch { throw; }                                            // остальные пробрасываем
```

**Когда восстанавливаться, а когда пробрасывать:**
- **Восстанавливайтесь**, когда у ошибки есть разумное значение по умолчанию или стратегия повтора
- **Пробрасывайте через `?`**, когда решение должен принимать *вызывающий* код
- **Добавляйте контекст** (`.context()`) на границах модулей, чтобы построить цепочку ошибок

---

## Упражнения

<details>
<summary><strong>🏋️ Упражнение: спроектируйте тип ошибок крейта</strong> (нажмите, чтобы раскрыть)</summary>

Вы создаёте сервис регистрации пользователей. Спроектируйте тип ошибок с помощью `thiserror`:

1. Определите `RegistrationError` с вариантами: `DuplicateEmail(String)`, `WeakPassword(String)`, `DatabaseError(#[from] sqlx::Error)`, `RateLimited { retry_after_secs: u64 }`
2. Создайте псевдоним `type Result<T> = std::result::Result<T, RegistrationError>;`
3. Напишите `register_user(email: &str, password: &str) -> Result<()>`, которая демонстрирует распространение через `?` и явное создание ошибок

<details>
<summary>🔑 Решение</summary>

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistrationError {
    #[error("Email already registered: {0}")]
    DuplicateEmail(String),

    #[error("Password too weak: {0}")]
    WeakPassword(String),

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Rate limited — retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
}

pub type Result<T> = std::result::Result<T, RegistrationError>;

pub fn register_user(email: &str, password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(RegistrationError::WeakPassword(
            "must be at least 8 characters".into(),
        ));
    }

    // Этот ? преобразует sqlx::Error → RegistrationError::Database автоматически
    // db.check_email_unique(email).await?;

    // Это явное создание ошибки для доменной логики
    if email.contains("+spam") {
        return Err(RegistrationError::DuplicateEmail(email.to_string()));
    }

    Ok(())
}
```

**Ключевой паттерн**: `#[from]` включает `?` для ошибок библиотек; явный `Err(...)` — для доменной логики. Псевдоним Result делает каждую сигнатуру чистой.

</details>
</details>

***


