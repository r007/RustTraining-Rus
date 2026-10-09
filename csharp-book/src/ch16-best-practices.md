## Лучшие практики для разработчиков C#

> **Что вы узнаете:** пять ключевых сдвигов мышления (GC→владение, исключения→Result, наследование→композиция),
> идиоматичную организацию проекта, стратегию обработки ошибок, паттерны тестирования и самые частые ошибки разработчиков C# в Rust.
>
> **Сложность:** 🟡 Средний

### 1. **Сдвиги мышления**
- **От GC к владению**: думайте о том, кому принадлежат данные и когда их освобождают
- **От исключений к Result**: делайте обработку ошибок явной и видимой
- **От наследования к композиции**: используйте трейты для композиции поведения
- **От null к Option**: делайте отсутствие значения явным в системе типов

### 2. **Организация кода**
```rust
// Структурируем проекты, как решения (solutions) C#
src/
├── main.rs          // Аналог Program.cs
├── lib.rs           // Точка входа библиотеки
├── models/          // Аналог папки Models/ в C#
│   ├── mod.rs
│   ├── user.rs
│   └── product.rs
├── services/        // Аналог папки Services/
│   ├── mod.rs
│   ├── user_service.rs
│   └── product_service.rs
├── controllers/     // Аналог Controllers/ (для веб-приложений)
├── repositories/    // Аналог Repositories/
└── utils/           // Аналог Utilities/
```

### 3. **Стратегия обработки ошибок**
```rust
// Создаём общий тип Result для приложения
pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Business logic error: {message}")]
    Business { message: String },
}

// Используем во всём приложении
pub async fn create_user(data: CreateUserRequest) -> AppResult<User> {
    validate_user_data(&data)?;  // Возвращает AppError::Validation
    let user = repository.create_user(data).await?;  // Возвращает AppError::Database
    Ok(user)
}
```

### 4. **Паттерны тестирования**
```rust
// Структурируем тесты, как модульные тесты C#
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;  // Для параметризованных тестов, как [Theory] в C#
    
    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = "test data";
        
        // Act
        let result = process_data(input);
        
        // Assert
        assert_eq!(result, "expected output");
    }
    
    #[rstest]
    #[case(1, 2, 3)]
    #[case(5, 5, 10)]
    #[case(0, 0, 0)]
    fn test_addition(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
        assert_eq!(add(a, b), expected);
    }
    
    #[tokio::test]  // Для асинхронных тестов
    async fn test_async_functionality() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### 5. **Частые ошибки, которых стоит избегать**
```rust
// [ERROR] Не пытайтесь реализовать наследование
// Вместо:
// struct Manager : Employee  // В Rust такого нет

// [OK] Используйте композицию через трейты
trait Employee {
    fn get_salary(&self) -> u32;
}

trait Manager: Employee {
    fn get_team_size(&self) -> usize;
}

// [ERROR] Не используйте unwrap() везде (это как игнорирование исключений)
let value = might_fail().unwrap();  // Может вызвать панику!

// [OK] Обрабатывайте ошибки правильно
let value = match might_fail() {
    Ok(v) => v,
    Err(e) => {
        log::error!("Operation failed: {}", e);
        return Err(e.into());
    }
};

// [ERROR] Не клонируйте всё подряд (как лишнее копирование объектов)
let data = expensive_data.clone();  // Дорого!

// [OK] Используйте заимствование, когда возможно
let data = &expensive_data;  // Всего лишь ссылка

// [ERROR] Не используйте RefCell повсюду (как превращение всего в изменяемое)
struct Data {
    value: RefCell<i32>,  // Внутренняя изменяемость — используйте осторожно
}

// [OK] Предпочитайте владеемые или заимствованные данные
struct Data {
    value: i32,  // Просто и понятно
}
```

Это руководство даёт разработчикам C# цельное представление о том, как их существующие знания переносятся на Rust, подчёркивая как сходства, так и принципиальные различия в подходе. Ключевое — понимать, что ограничения Rust (например, владение) призваны предотвратить целые классы ошибок, возможных в C#, ценой некоторой первоначальной сложности.

---

### 6. **Избегаем избыточного `clone()`** 🟡

Разработчики C# инстинктивно клонируют данные, потому что стоимость берёт на себя GC. В Rust каждый `.clone()` — это явное выделение памяти. Большинство из них можно убрать с помощью заимствования.

```rust
// [ERROR] Привычка из C#: клонировать строки, чтобы передавать их дальше
fn greet(name: String) {
    println!("Hello, {name}");
}

let user_name = String::from("Alice");
greet(user_name.clone());  // ненужное выделение памяти
greet(user_name.clone());  // и снова

// [OK] Заимствуйте вместо этого — без выделений памяти
fn greet(name: &str) {
    println!("Hello, {name}");
}

let user_name = String::from("Alice");
greet(&user_name);  // заимствуем
greet(&user_name);  // заимствуем снова — без затрат
```

**Когда clone уместен:**
- Перемещение данных в поток или в замыкание `'static` (`Arc::clone` дёшев — он лишь увеличивает счётчик)
- Кэширование: вам действительно нужна независимая копия
- Прототипирование: сначала заставьте работать, потом уберите лишние clone

**Чек-лист решения:**
1. Можно передать `&T` или `&str`? → Так и сделайте
2. Вызываемой стороне нужно владение? → Передавайте перемещением, а не клоном
3. Данные разделяются между потоками? → Используйте `Arc<T>` (clone — это лишь увеличение счётчика ссылок)
4. Ничего из перечисленного? → `clone()` оправдан

---

### 7. **Избегаем `unwrap()` в продакшен-коде** 🟡

Разработчики C#, которые игнорируют исключения, пишут `.unwrap()` везде в Rust. И то и другое одинаково опасно.

```rust
// [ERROR] Ловушка «потом поправлю»
let config = std::fs::read_to_string("config.toml").unwrap();
let port: u16 = config_value.parse().unwrap();
let conn = db_pool.get().await.unwrap();

// [OK] Пробрасываем через ? в прикладном коде
let config = std::fs::read_to_string("config.toml")?;
let port: u16 = config_value.parse()?;
let conn = db_pool.get().await?;

// [OK] Используйте expect() только тогда, когда сбой — действительно ошибка в программе
let home = std::env::var("HOME")
    .expect("HOME environment variable must be set");  // фиксирует инвариант
```

**Эмпирическое правило:**
| Метод | Когда использовать |
|--------|------------|
| `?` | Код приложения и библиотек — передать вызывающему коду |
| `expect("reason")` | Проверки при запуске, инварианты, которые *обязаны* выполняться |
| `unwrap()` | Только в тестах или после проверки `is_some()`/`is_ok()` |
| `unwrap_or(default)` | Когда есть разумное значение по умолчанию |
| `unwrap_or_else(|| ...)` | Когда значение по умолчанию дорого вычислять |

---

### 8. **Борьба с проверщиком заимствований (и как перестать бороться)** 🟡

Каждый разработчик C# проходит этап, когда проверщик заимствований отвергает код, который кажется правильным. Исправление обычно требует изменения структуры, а не обходного манёвра.

```rust
// [ERROR] Попытка изменять коллекцию во время перебора (паттерн foreach + изменение из C#)
let mut items = vec![1, 2, 3, 4, 5];
for item in &items {
    if *item > 3 {
        items.push(*item * 2);  // ОШИБКА: нельзя заимствовать items как изменяемое
    }
}

// [OK] Сначала собираем, потом изменяем
let extras: Vec<i32> = items.iter()
    .filter(|&&x| x > 3)
    .map(|&x| x * 2)
    .collect();
items.extend(extras);
```

```rust
// [ERROR] Возврат ссылки на локальную переменную (в C# ссылки свободно возвращаются благодаря GC)
fn get_greeting() -> &str {
    let s = String::from("hello");
    &s  // ОШИБКА: s уничтожается в конце функции
}

// [OK] Возвращаем владеемые данные
fn get_greeting() -> String {
    String::from("hello")  // вызывающий код владеет ими
}
```

**Распространённые паттерны, которые снимают конфликты с проверщиком заимствований:**

| Привычка из C# | Решение в Rust |
|----------|--------------|
| Хранить ссылки в структурах | Использовать владеемые данные или добавить параметры времён жизни |
| Свободно изменять разделяемое состояние | Использовать `Arc<Mutex<T>>` или перестроить код, чтобы не разделять данные |
| Возвращать ссылки на локальные переменные | Возвращать владеемые значения |
| Изменять коллекцию во время перебора | Сначала собрать изменения, потом применить |
| Несколько изменяемых ссылок | Разбить структуру на независимые части |

---

### 9. **Устраняем «пирамиды» присваиваний** 🟢

Разработчики C# пишут цепочки вида `if (x != null) { if (x.Value > 0) { ... } }`. Конструкции `match`, `if let` и `?` в Rust позволяют выровнять такой код.

```rust
// [ERROR] Вложенный стиль проверок на null из C#
fn process(input: Option<String>) -> Option<usize> {
    match input {
        Some(s) => {
            if !s.is_empty() {
                match s.parse::<usize>() {
                    Ok(n) => {
                        if n > 0 {
                            Some(n * 2)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        }
        None => None,
    }
}

// [OK] Выравниваем с помощью комбинаторов
fn process(input: Option<String>) -> Option<usize> {
    input
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .map(|n| n * 2)
}
```

**Ключевые комбинаторы, которые должен знать каждый разработчик C#:**

| Комбинатор | Что делает | Аналог в C# |
|-----------|-------------|---------------|
| `map` | Преобразует внутреннее значение | `Select` / null-условный `?.` |
| `and_then` | Связывает операции, возвращающие Option/Result | `SelectMany` / `?.Method()` |
| `filter` | Оставляет значение, только если предикат истинен | `Where` |
| `unwrap_or` | Задаёт значение по умолчанию | `?? defaultValue` |
| `ok()` | Превращает `Result` в `Option` (отбрасывая ошибку) | — |
| `transpose` | Меняет местами `Option<Result>` и `Result<Option>` | — |
