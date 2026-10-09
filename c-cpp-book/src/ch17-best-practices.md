# Сводка лучших практик Rust

> **Что вы узнаете:** практические рекомендации по написанию идиоматичного Rust — организация кода, соглашения об именовании, паттерны обработки ошибок и документирование. Краткая справочная глава, к которой вы будете возвращаться часто.

## Организация кода
- **Предпочитайте небольшие функции**: их легко тестировать и понимать
- **Используйте описательные имена**: `calculate_total_price()` вместо `calc()`
- **Группируйте связанную функциональность**: используйте модули и отдельные файлы
- **Пишите документацию**: используйте `///` для публичных API

## Обработка ошибок
- **Избегайте `unwrap()`, если операция не может завершиться неудачей**: используйте его, только когда на 100% уверены, что panic не будет
```rust
// Плохо: может вызвать panic
let value = some_option.unwrap();

// Хорошо: обрабатываем случай None
let value = some_option.unwrap_or(default_value);
let value = some_option.unwrap_or_else(|| expensive_computation());
let value = some_option.unwrap_or_default(); // Использует трейт Default

// Для Result<T, E>
let value = some_result.unwrap_or(fallback_value);
let value = some_result.unwrap_or_else(|err| {
    eprintln!("Error occurred: {err}");
    default_value
});
```
- **Используйте `expect()` с понятными сообщениями**: если unwrap оправдан, объясните почему
```rust
let config = std::env::var("CONFIG_PATH")
    .expect("CONFIG_PATH environment variable must be set");
```
- **Возвращайте `Result<T, E>` из операций, которые могут завершиться ошибкой**: пусть вызывающий код решает, как обрабатывать ошибки
- **Используйте `thiserror` для собственных типов ошибок**: это удобнее ручной реализации
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {message}")]
    Parse { message: String },
    
    #[error("Value {value} is out of range")]
    OutOfRange { value: i32 },
}
```
- **Цепочка ошибок через оператор `?`**: передавайте ошибки вверх по стеку вызовов
- **Предпочитайте `thiserror` перед `anyhow`**: соглашение нашей команды — определять явные перечисления ошибок через `#[derive(thiserror::Error)]`, чтобы вызывающий код мог сопоставлять конкретные варианты.
  `anyhow::Error` удобен для быстрых прототипов, но стирает тип ошибки, из-за чего вызывающему коду сложнее обрабатывать конкретные сбои. Используйте `thiserror` для библиотечного и продакшн-кода; `anyhow` оставьте для одноразовых скриптов или бинарников верхнего уровня, где нужно только вывести ошибку.
- **Когда `unwrap()` допустим**:
  - **Модульные тесты**: `assert_eq!(result.unwrap(), expected)`
  - **Прототипирование**: быстрый и грязный код, который вы потом замените
  - **Операции, которые не могут завершиться ошибкой**: когда вы можете доказать, что сбоя не будет
```rust
let numbers = vec![1, 2, 3];
let first = numbers.get(0).unwrap(); // Безопасно: мы только что создали вектор с элементами

// Лучше: используем expect() с объяснением
let first = numbers.get(0).expect("numbers vec is non-empty by construction");
```
- **Быстро падайте (fail fast)**: проверяйте предусловия в начале и сразу возвращайте ошибки

## Управление памятью
- **Предпочитайте заимствование клонированию**: используйте `&T` вместо клонирования, когда это возможно
- **Используйте `Rc<T>` умеренно**: только когда нужно совместное владение
- **Ограничивайте времена жизни**: используйте области видимости `{}`, чтобы контролировать, когда значения уничтожаются
- **Избегайте `RefCell<T>` в публичных API**: внутреннюю изменяемость держите внутри

## Производительность
- **Профилируйте перед оптимизацией**: используйте `cargo bench` и инструменты профилирования
- **Предпочитайте итераторы циклам**: они читаемее и часто быстрее
- **Используйте `&str` вместо `String`**: когда не нужно владение
- **Рассмотрите `Box<T>` для больших объектов на стеке**: при необходимости переносите их в кучу

## Трейты, которые стоит реализовать

### Базовые трейты, о которых стоит подумать для каждого типа

Создавая собственные типы, рассмотрите реализацию этих базовых трейтов, чтобы ваши типы ощущались в Rust «родными»:

#### **Debug и Display**
```rust
use std::fmt;

#[derive(Debug)]  // Автоматическая реализация для отладки
struct Person {
    name: String,
    age: u32,
}

// Ручная реализация Display для вывода, который видит пользователь
impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (age {})", self.name, self.age)
    }
}

// Использование:
let person = Person { name: "Alice".to_string(), age: 30 };
println!("{:?}", person);  // Debug: Person { name: "Alice", age: 30 }
println!("{}", person);    // Display: Alice (age 30)
```

#### **Clone и Copy**
```rust
// Copy: неявное дублирование для малых простых типов
#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

// Clone: явное дублирование для сложных типов
#[derive(Debug, Clone)]
struct Person {
    name: String,  // String не реализует Copy
    age: u32,
}

let p1 = Point { x: 1, y: 2 };
let p2 = p1;  // Copy (неявно)

let person1 = Person { name: "Bob".to_string(), age: 25 };
let person2 = person1.clone();  // Clone (явно)
```

#### **PartialEq и Eq**
```rust
#[derive(Debug, PartialEq, Eq)]
struct UserId(u64);

#[derive(Debug, PartialEq)]
struct Temperature {
    celsius: f64,  // f64 не реализует Eq (из-за NaN)
}

let id1 = UserId(123);
let id2 = UserId(123);
assert_eq!(id1, id2);  // Работает благодаря PartialEq

let temp1 = Temperature { celsius: 20.0 };
let temp2 = Temperature { celsius: 20.0 };
assert_eq!(temp1, temp2);  // Работает с PartialEq
```

#### **PartialOrd и Ord**
```rust
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Priority(u8);

let high = Priority(1);
let low = Priority(10);
assert!(high < low);  // Меньшие числа = выше приоритет

// Использование в коллекциях
let mut priorities = vec![Priority(5), Priority(1), Priority(8)];
priorities.sort();  // Работает, потому что Priority реализует Ord
```

#### **Default**
```rust
#[derive(Debug, Default)]
struct Config {
    debug: bool,           // false (по умолчанию)
    max_connections: u32,  // 0 (по умолчанию)
    timeout: Option<u64>,  // None (по умолчанию)
}

// Собственная реализация Default
impl Default for Config {
    fn default() -> Self {
        Config {
            debug: false,
            max_connections: 100,  // Собственное значение по умолчанию
            timeout: Some(30),     // Собственное значение по умолчанию
        }
    }
}

let config = Config::default();
let config = Config { debug: true, ..Default::default() };  // Частичное переопределение
```

#### **From и Into**
```rust
struct UserId(u64);
struct UserName(String);

// Реализуем From, а Into получаем бесплатно
impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        UserId(id)
    }
}

impl From<String> for UserName {
    fn from(name: String) -> Self {
        UserName(name)
    }
}

impl From<&str> for UserName {
    fn from(name: &str) -> Self {
        UserName(name.to_string())
    }
}

// Использование:
let user_id: UserId = 123u64.into();         // Через Into
let user_id = UserId::from(123u64);          // Через From
let username = UserName::from("alice");      // &str -> UserName
let username: UserName = "bob".into();       // Через Into
```

#### **TryFrom и TryInto**
```rust
use std::convert::TryFrom;

struct PositiveNumber(u32);

#[derive(Debug)]
struct NegativeNumberError;

impl TryFrom<i32> for PositiveNumber {
    type Error = NegativeNumberError;
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value >= 0 {
            Ok(PositiveNumber(value as u32))
        } else {
            Err(NegativeNumberError)
        }
    }
}

// Использование:
let positive = PositiveNumber::try_from(42)?;     // Ok(PositiveNumber(42))
let error = PositiveNumber::try_from(-5);         // Err(NegativeNumberError)
```

#### **Serde (для сериализации)**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

// Автоматическая сериализация и десериализация в JSON
let user = User {
    id: 1,
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};

let json = serde_json::to_string(&user)?;
let deserialized: User = serde_json::from_str(&json)?;
```

### Чек-лист реализации трейтов

Для любого нового типа рассмотрите этот чек-лист:

```rust
#[derive(
    Debug,          // [OK] Всегда реализуйте для отладки
    Clone,          // [OK] Если тип должен можно было дублировать
    PartialEq,      // [OK] Если тип должен можно было сравнивать
    Eq,             // [OK] Если сравнение рефлексивно и транзитивно
    PartialOrd,     // [OK] Если у типа есть порядок
    Ord,            // [OK] Если порядок полный
    Hash,           // [OK] Если тип будет ключом HashMap
    Default,        // [OK] Если есть разумное значение по умолчанию
)]
struct MyType {
    // поля...
}

// Ручные реализации, которые стоит рассмотреть:
impl Display for MyType { /* представление для пользователя */ }
impl From<OtherType> for MyType { /* удобное преобразование */ }
impl TryFrom<FallibleType> for MyType { /* преобразование, которое может завершиться ошибкой */ }
```

### Когда НЕ реализовывать трейты

- **Не реализуйте Copy для типов с данными в куче**: `String`, `Vec`, `HashMap` и т. д.
- **Не реализуйте Eq, если значения могут быть NaN**: типы, содержащие `f32`/`f64`
- **Не реализуйте Default, если нет разумного значения по умолчанию**: файловые дескрипторы, сетевые соединения
- **Не реализуйте Clone, если клонирование дорогое**: большие структуры данных (вместо этого рассмотрите `Rc<T>`)

### Итоги: преимущества трейтов

| Трейт | Преимущество | Когда использовать |
|-------|-----------|-------------|
| `Debug` | `println!("{:?}", value)` | Всегда (кроме редких случаев) |
| `Display` | `println!("{}", value)` | Типы для пользователя |
| `Clone` | `value.clone()` | Когда явное дублирование имеет смысл |
| `Copy` | Неявное дублирование | Малые простые типы |
| `PartialEq` | Операторы `==` и `!=` | Большинство типов |
| `Eq` | Рефлексивное равенство | Когда равенство математически корректно |
| `PartialOrd` | `<`, `>`, `<=`, `>=` | Типы с естественным порядком |
| `Ord` | `sort()`, `BinaryHeap` | Когда порядок полный |
| `Hash` | Ключи `HashMap` | Типы, используемые как ключи словаря |
| `Default` | `Default::default()` | Типы с очевидными значениями по умолчанию |
| `From/Into` | Удобные преобразования | Частые преобразования типов |
| `TryFrom/TryInto` | Преобразования с возможной ошибкой | Преобразования, которые могут завершиться неудачей |

----

----

