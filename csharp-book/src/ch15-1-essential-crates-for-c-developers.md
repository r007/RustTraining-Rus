## Основные крейты для разработчиков C#

> **Что вы узнаете:** аналоги крейтов Rust для распространённых библиотек .NET — serde (JSON.NET),
> reqwest (HttpClient), tokio (Task/async), sqlx (Entity Framework), а также подробный разбор системы атрибутов serde
> в сравнении с `System.Text.Json`.
>
> **Сложность:** 🟡 Средний

### Аналоги основной функциональности

```rust
// Зависимости Cargo для разработчиков C#
[dependencies]
# Сериализация (как Newtonsoft.Json или System.Text.Json)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP-клиент (как HttpClient)
reqwest = { version = "0.11", features = ["json"] }

# Асинхронный рантайм (как Task.Run, async/await)
tokio = { version = "1.0", features = ["full"] }

# Обработка ошибок (как собственные исключения)
thiserror = "1.0"
anyhow = "1.0"

# Логирование (как ILogger, Serilog)
log = "0.4"
env_logger = "0.10"

# Дата и время (как DateTime)
chrono = { version = "0.4", features = ["serde"] }

# UUID (как System.Guid)
uuid = { version = "1.0", features = ["v4", "serde"] }

# Коллекции (как List<T>, Dictionary<K,V>)
# Встроены в std, но для продвинутых коллекций:
indexmap = "2.0"  # HashMap с сохранением порядка

# Конфигурация (как IConfiguration)
config = "0.13"

# База данных (как Entity Framework)
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }

# Тестирование (как xUnit, NUnit)
# Встроено в std, но для расширенных возможностей:
rstest = "0.18"  # Параметризованные тесты

# Моки (как Moq)
mockall = "0.11"

# Параллельная обработка (как Parallel.ForEach)
rayon = "1.7"
```

### Примеры использования

```rust
use serde::{Deserialize, Serialize};
use reqwest;
use tokio;
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Модели данных (как POCO с атрибутами в C#)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

// Собственные типы ошибок (как собственные исключения)
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("User not found: {id}")]
    UserNotFound { id: Uuid },
    
    #[error("Validation failed: {message}")]
    Validation { message: String },
}

// Аналог класса сервиса
pub struct UserService {
    client: reqwest::Client,
    base_url: String,
}

impl UserService {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        UserService { client, base_url }
    }
    
    // Асинхронный метод (как async Task<User> в C#)
    pub async fn get_user(&self, id: Uuid) -> Result<User, ApiError> {
        let url = format!("{}/users/{}", self.base_url, id);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(ApiError::UserNotFound { id });
        }
        
        let user = response.json::<User>().await?;
        Ok(user)
    }
    
    // Создание пользователя (как async Task<User> в C#)
    pub async fn create_user(&self, name: String, email: String) -> Result<User, ApiError> {
        if name.trim().is_empty() {
            return Err(ApiError::Validation {
                message: "Name cannot be empty".to_string(),
            });
        }
        
        let new_user = User {
            id: Uuid::new_v4(),
            name,
            email,
            created_at: Utc::now(),
        };
        
        let response = self.client
            .post(&format!("{}/users", self.base_url))
            .json(&new_user)
            .send()
            .await?;
        
        let created_user = response.json::<User>().await?;
        Ok(created_user)
    }
}

// Пример использования (как метод Main в C#)
#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Инициализация логирования (как настройка ILogger)
    env_logger::init();
    
    let service = UserService::new("https://api.example.com".to_string());
    
    // Создание пользователя
    let user = service.create_user(
        "John Doe".to_string(),
        "john@example.com".to_string(),
    ).await?;
    
    println!("Created user: {:?}", user);
    
    // Получение пользователя
    let retrieved_user = service.get_user(user.id).await?;
    println!("Retrieved user: {:?}", retrieved_user);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]  // Как [Test] или [Fact] в C#
    async fn test_user_creation() {
        let service = UserService::new("http://localhost:8080".to_string());
        
        let result = service.create_user(
            "Test User".to_string(),
            "test@example.com".to_string(),
        ).await;
        
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
    }
    
    #[test]
    fn test_validation() {
        // Синхронный тест
        let error = ApiError::Validation {
            message: "Invalid input".to_string(),
        };
        
        assert_eq!(error.to_string(), "Validation failed: Invalid input");
    }
}
```

***

<!-- ch15.1a: Serde Deep Dive for C# Developers -->
## Глубокое погружение в serde: сериализация JSON для разработчиков C#

Разработчики C# активно используют `System.Text.Json` или `Newtonsoft.Json`. В Rust **serde** (serialize/deserialize) — универсальный фреймворк для этого. Понимание его системы атрибутов открывает большинство сценариев работы с данными.

### Базовый derive: отправная точка
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    age: u32,
    email: String,
}

let user = User { name: "Alice".into(), age: 30, email: "alice@co.com".into() };
let json = serde_json::to_string_pretty(&user)?;
let parsed: User = serde_json::from_str(&json)?;
```

```csharp
// Аналог в C#
public class User
{
    public string Name { get; set; }
    public int Age { get; set; }
    public string Email { get; set; }
}
var json = JsonSerializer.Serialize(user, new JsonSerializerOptions { WriteIndented = true });
var parsed = JsonSerializer.Deserialize<User>(json);
```

### Атрибуты уровня полей (аналог `[JsonProperty]`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    // Переименование поля в JSON (как [JsonPropertyName("user_id")])
    #[serde(rename = "user_id")]
    id: u64,

    // Разные имена при сериализации и десериализации
    #[serde(rename(serialize = "userName", deserialize = "user_name"))]
    name: String,

    // Полностью пропустить поле (как [JsonIgnore])
    #[serde(skip)]
    internal_cache: Option<String>,

    // Пропустить только при сериализации
    #[serde(skip_serializing)]
    password_hash: String,

    // Значение по умолчанию, если поля нет в JSON (как значения в конструкторе по умолчанию)
    #[serde(default)]
    is_active: bool,

    // Пользовательское значение по умолчанию
    #[serde(default = "default_role")]
    role: String,

    // Развернуть вложенную структуру в родительскую (как [JsonExtensionData])
    #[serde(flatten)]
    metadata: Metadata,

    // Пропустить, если значение None (не выводить null-поля)
    #[serde(skip_serializing_if = "Option::is_none")]
    nickname: Option<String>,
}

fn default_role() -> String { "viewer".into() }

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    created_at: String,
    version: u32,
}
```

```csharp
// Эквивалентные атрибуты в C#
public class ApiResponse
{
    [JsonPropertyName("user_id")]
    public ulong Id { get; set; }

    [JsonIgnore]
    public string? InternalCache { get; set; }

    [JsonExtensionData]
    public Dictionary<string, JsonElement>? Metadata { get; set; }
}
```

### Представления перечислений (ключевое отличие от C#)

В serde для перечислений поддерживаются **четыре разных представления в JSON** — понятие, которому в C# нет прямого аналога, поскольку перечисления C# всегда целочисленные или строковые.

```rust
use serde::{Deserialize, Serialize};

// 1. Внешне тегированное (ПО УМОЛЧАНИЮ) — самое распространённое
#[derive(Serialize, Deserialize)]
enum Message {
    Text(String),
    Image { url: String, width: u32 },
    Ping,
}
// Вариант Text:  {"Text": "hello"}
// Вариант Image: {"Image": {"url": "...", "width": 100}}
// Вариант Ping:  "Ping"

// 2. Внутренне тегированное — похоже на дискриминируемые объединения в других языках
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event {
    Created { id: u64, name: String },
    Deleted { id: u64 },
    Updated { id: u64, fields: Vec<String> },
}
// {"type": "Created", "id": 1, "name": "Alice"}
// {"type": "Deleted", "id": 1}

// 3. Смежно тегированное — тег и содержимое в отдельных полях
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum ApiResult {
    Success(UserData),
    Error(String),
}
// {"t": "Success", "c": {"name": "Alice"}}
// {"t": "Error", "c": "not found"}

// 4. Без тега — serde пробует варианты по порядку
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum FlexibleValue {
    Integer(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}
// 42, 3.14, "hello", true — serde сам определяет вариант
```

### Пользовательская сериализация (аналог `JsonConverter`)
```rust
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Пользовательская сериализация для конкретного поля
#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    timeout: std::time::Duration,
}

fn serialize_duration<S: Serializer>(dur: &std::time::Duration, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u64(dur.as_millis() as u64)
}

fn deserialize_duration<'de, D: Deserializer<'de>>(d: D) -> Result<std::time::Duration, D::Error> {
    let ms = u64::deserialize(d)?;
    Ok(std::time::Duration::from_millis(ms))
}
// JSON: {"timeout": 5000}  ↔  Config { timeout: Duration::from_millis(5000) }
```

### Атрибуты уровня контейнера

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // Все поля в JSON становятся camelCase
struct UserProfile {
    first_name: String,      // → "firstName"
    last_name: String,       // → "lastName"
    email_address: String,   // → "emailAddress"
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]  // Отвергать JSON с лишними полями (строгий разбор)
struct StrictConfig {
    port: u16,
    host: String,
}
// serde_json::from_str::<StrictConfig>(r#"{"port":8080,"host":"localhost","extra":true}"#)
// → Error: unknown field `extra`
```

### Краткая справка по атрибутам serde

| Атрибут | Уровень | Аналог в C# | Назначение |
|-----------|-------|---------------|---------|
| `#[serde(rename = "...")]` | Поле | `[JsonPropertyName]` | Переименование в JSON |
| `#[serde(skip)]` | Поле | `[JsonIgnore]` | Полностью пропустить |
| `#[serde(default)]` | Поле | Значение по умолчанию | Использовать `Default::default()`, если поля нет |
| `#[serde(flatten)]` | Поле | `[JsonExtensionData]` | Слить вложенную структуру в родительскую |
| `#[serde(skip_serializing_if = "...")]` | Поле | `JsonIgnoreCondition` | Условный пропуск |
| `#[serde(rename_all = "camelCase")]` | Контейнер | `JsonSerializerOptions.PropertyNamingPolicy` | Соглашение об именовании |
| `#[serde(deny_unknown_fields)]` | Контейнер | — | Строгая десериализация |
| `#[serde(tag = "type")]` | Перечисление | Паттерн дискриминатора | Внутреннее тегирование |
| `#[serde(untagged)]` | Перечисление | — | Пробовать варианты по порядку |
| `#[serde(with = "...")]` | Поле | `[JsonConverter]` | Пользовательская сериализация и десериализация |

### За пределами JSON: serde работает везде
```rust
// ТОТ ЖЕ derive работает для ВСЕХ форматов — нужно лишь сменить крейт
let user = User { name: "Alice".into(), age: 30, email: "a@b.com".into() };

let json  = serde_json::to_string(&user)?;        // JSON
let toml  = toml::to_string(&user)?;               // TOML (конфигурационные файлы)
let yaml  = serde_yaml::to_string(&user)?;          // YAML
let cbor  = serde_cbor::to_vec(&user)?;             // CBOR (бинарный, компактный)
let msgpk = rmp_serde::to_vec(&user)?;              // MessagePack (бинарный)

// Один #[derive(Serialize, Deserialize)] — каждый формат бесплатно
```

***


