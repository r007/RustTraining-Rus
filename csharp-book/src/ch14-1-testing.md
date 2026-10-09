## Тестирование в Rust и C#

> **Что вы узнаете:** встроенный `#[test]` против xUnit, параметризованные тесты с `rstest` (аналог `[Theory]`),
> property-тестирование с `proptest`, моки с `mockall` и паттерны асинхронных тестов.
>
> **Сложность:** 🟡 Средний

### Модульные тесты
```csharp
// C# — xUnit
using Xunit;

public class CalculatorTests
{
    [Fact]
    public void Add_ReturnsSum()
    {
        var calc = new Calculator();
        Assert.Equal(5, calc.Add(2, 3));
    }

    [Theory]
    [InlineData(1, 2, 3)]
    [InlineData(0, 0, 0)]
    [InlineData(-1, 1, 0)]
    public void Add_Theory(int a, int b, int expected)
    {
        Assert.Equal(expected, new Calculator().Add(a, b));
    }
}
```

```rust
// Rust — встроенное тестирование, внешний фреймворк не нужен
pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]  // Компилируется только при `cargo test`
mod tests {
    use super::*;  // Импорт из родительского модуля

    #[test]
    fn add_returns_sum() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn add_negative_numbers() {
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn add_overflow_panics() {
        let _ = add(i32::MAX, 1); // паникует в debug-режиме
    }
}
```

### Параметризованные тесты (аналог `[Theory]`)
```rust
// Для параметризованных тестов используйте крейт `rstest`
use rstest::rstest;

#[rstest]
#[case(1, 2, 3)]
#[case(0, 0, 0)]
#[case(-1, 1, 0)]
fn test_add(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
    assert_eq!(add(a, b), expected);
}

// Фикстуры — аналог методов настройки тестов
#[rstest]
fn test_with_fixture(#[values(1, 2, 3)] x: i32) {
    assert!(x > 0);
}
```

### Сравнение утверждений

| C# (xUnit) | Rust | Примечания |
|-------------|------|-------|
| `Assert.Equal(expected, actual)` | `assert_eq!(expected, actual)` | При провале выводит diff |
| `Assert.NotEqual(a, b)` | `assert_ne!(a, b)` | |
| `Assert.True(condition)` | `assert!(condition)` | |
| `Assert.Contains("sub", str)` | `assert!(str.contains("sub"))` | |
| `Assert.Throws<T>(() => ...)` | `#[should_panic]` | Или используйте `std::panic::catch_unwind` |
| `Assert.Null(obj)` | `assert!(option.is_none())` | Нет null — используйте `Option` |

### Организация тестов

```text
my_crate/
├── src/
│   ├── lib.rs          # Модульные тесты в #[cfg(test)] mod tests { }
│   └── parser.rs       # У каждого модуля может быть свой тестовый модуль
├── tests/              # Интеграционные тесты (каждый файл — отдельный крейт)
│   ├── parser_test.rs  # Тестирует публичный API как внешний потребитель
│   └── api_test.rs
└── benches/            # Бенчмарки (с крейтом criterion)
    └── my_benchmark.rs
```

```rust
// tests/parser_test.rs — интеграционный тест
// Имеет доступ только к ПУБЛИЧНОМУ API (как тестирование извне сборки)
use my_crate::parser;

#[test]
fn test_parse_valid_input() {
    let result = parser::parse("valid input");
    assert!(result.is_ok());
}
```

### Асинхронные тесты
```csharp
// C# — асинхронный тест в xUnit
[Fact]
public async Task GetUser_ReturnsUser()
{
    var service = new UserService();
    var user = await service.GetUserAsync(1);
    Assert.Equal("Alice", user.Name);
}
```

```rust
// Rust — асинхронный тест с tokio
#[tokio::test]
async fn get_user_returns_user() {
    let service = UserService::new();
    let user = service.get_user(1).await.unwrap();
    assert_eq!(user.name, "Alice");
}
```

### Моки с mockall
```rust
use mockall::automock;

#[automock]                         // Генерирует структуру MockUserRepo
trait UserRepo {
    fn find_by_id(&self, id: u32) -> Option<User>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_returns_user_from_repo() {
        let mut mock = MockUserRepo::new();
        mock.expect_find_by_id()
            .with(mockall::predicate::eq(1))
            .returning(|_| Some(User { name: "Alice".into() }));

        let service = UserService::new(mock);
        let user = service.get_user(1).unwrap();
        assert_eq!(user.name, "Alice");
    }
}
```

```csharp
// C# — эквивалент с Moq
var mock = new Mock<IUserRepo>();
mock.Setup(r => r.FindById(1)).Returns(new User { Name = "Alice" });
var service = new UserService(mock.Object);
Assert.Equal("Alice", service.GetUser(1).Name);
```

<details>
<summary><strong>🏋️ Упражнение: напишите полноценные тесты</strong> (нажмите, чтобы раскрыть)</summary>

**Задача**: для этой функции напишите тесты, которые покрывают: основной сценарий, пустой ввод, числовые строки и Unicode.

```rust
pub fn title_case(input: &str) -> String {
    input.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str().to_lowercase()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
```

<details>
<summary>🔑 Решение</summary>

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path() {
        assert_eq!(title_case("hello world"), "Hello World");
    }

    #[test]
    fn empty_input() {
        assert_eq!(title_case(""), "");
    }

    #[test]
    fn single_word() {
        assert_eq!(title_case("rust"), "Rust");
    }

    #[test]
    fn already_title_case() {
        assert_eq!(title_case("Hello World"), "Hello World");
    }

    #[test]
    fn all_caps() {
        assert_eq!(title_case("HELLO WORLD"), "Hello World");
    }

    #[test]
    fn extra_whitespace() {
        // split_whitespace обрабатывает несколько пробелов
        assert_eq!(title_case("  hello   world  "), "Hello World");
    }

    #[test]
    fn unicode() {
        assert_eq!(title_case("café résumé"), "Café Résumé");
    }

    #[test]
    fn numeric_words() {
        assert_eq!(title_case("hello 42 world"), "Hello 42 World");
    }
}
```

**Ключевая мысль**: встроенный тестовый фреймворк Rust покрывает большинство потребностей в модульном тестировании. Используйте `rstest` для параметризованных тестов и `mockall` для моков — большой тестовый фреймворк вроде xUnit не нужен.

</details>
</details>


<!-- ch14a.1: Property Testing with proptest -->
## Property-тестирование: доказательство корректности в масштабе

Разработчикам C#, знакомым с **FsCheck**, property-тестирование будет понятно сразу: вместо написания отдельных тест-кейсов вы описываете *свойства*, которые должны выполняться для **всех возможных входных данных**, а фреймворк генерирует тысячи случайных входов, чтобы их опровергнуть.

### Зачем нужно property-тестирование
```csharp
// C# — написанные вручную модульные тесты проверяют конкретные случаи
[Fact]
public void Reverse_Twice_Returns_Original()
{
    var list = new List<int> { 1, 2, 3 };
    list.Reverse();
    list.Reverse();
    Assert.Equal(new[] { 1, 2, 3 }, list);
}
// Но что насчёт пустых списков? Одиночных элементов? 10 000 элементов? Отрицательных чисел?
// Понадобились бы десятки тестов, написанных вручную.
```

```rust
// Rust — proptest автоматически генерирует тысячи входов
use proptest::prelude::*;

fn reverse<T: Clone>(v: &[T]) -> Vec<T> {
    v.iter().rev().cloned().collect()
}

proptest! {
    #[test]
    fn reverse_twice_is_identity(ref v in prop::collection::vec(any::<i32>(), 0..1000)) {
        let reversed_twice = reverse(&reverse(v));
        prop_assert_eq!(v, &reversed_twice);
    }
    // proptest запускает это на сотнях случайных значений Vec<i32>:
    // [], [0], [i32::MIN, i32::MAX], [42; 999], случайные последовательности...
    // Если тест падает, он СЖИМАЕТСЯ до наименьшего падающего входа!
}
```

### Начало работы с proptest
```toml
# Cargo.toml
[dev-dependencies]
proptest = "1.4"
```

### Распространённые паттерны для разработчиков C#

```rust
use proptest::prelude::*;

// 1. Свойство «туда и обратно»: сериализация → десериализация = тождество
// (Как тестирование JsonSerializer.Serialize → Deserialize)
proptest! {
    #[test]
    fn json_roundtrip(name in "[a-zA-Z]{1,50}", age in 0u32..150) {
        let user = User { name: name.clone(), age };
        let json = serde_json::to_string(&user).unwrap();
        let parsed: User = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(user, parsed);
    }
}

// 2. Свойство-инвариант: результат всегда удовлетворяет условию
proptest! {
    #[test]
    fn sort_output_is_sorted(ref v in prop::collection::vec(any::<i32>(), 0..500)) {
        let mut sorted = v.clone();
        sorted.sort();
        // Каждая соседняя пара должна быть в порядке
        for window in sorted.windows(2) {
            prop_assert!(window[0] <= window[1]);
        }
    }
}

// 3. Свойство-оракул: сравниваем две реализации
proptest! {
    #[test]
    fn fast_path_matches_slow_path(input in "[0-9a-f]{1,100}") {
        let result_fast = parse_hex_fast(&input);
        let result_slow = parse_hex_slow(&input);
        prop_assert_eq!(result_fast, result_slow);
    }
}

// 4. Пользовательские стратегии: генерируем данные, специфичные для предметной области
fn valid_email() -> impl Strategy<Value = String> {
    ("[a-z]{1,20}", "[a-z]{1,10}", prop::sample::select(vec!["com", "org", "io"]))
        .prop_map(|(user, domain, tld)| format!("{}@{}.{}", user, domain, tld))
}

proptest! {
    #[test]
    fn email_parsing_accepts_valid_emails(email in valid_email()) {
        let result = Email::new(&email);
        prop_assert!(result.is_ok(), "Failed to parse: {}", email);
    }
}
```

### Сравнение proptest и FsCheck

| Возможность | C# FsCheck | Rust proptest |
|---------|-----------|---------------|
| Генерация случайных входов | `Arb.Generate<T>()` | `any::<T>()` |
| Пользовательские генераторы | `Arb.Register<T>()` | `impl Strategy<Value = T>` |
| Сжатие при падении (shrinking) | Автоматическое | Автоматическое |
| Строковые шаблоны | Вручную | Стратегия `"[regex]"` |
| Генерация коллекций | `Gen.ListOf` | `prop::collection::vec(strategy, range)` |
| Композиция генераторов | `Gen.Select` | `.prop_map()`, `.prop_flat_map()` |
| Конфигурация (число случаев) | `Config.MaxTest` | `#![proptest_config(ProptestConfig::with_cases(10000))]` внутри блока `proptest!` |

### Когда использовать property-тестирование, а когда модульное

| Используйте **модульные тесты**, когда | Используйте **proptest**, когда |
|------------------------|----------------------|
| Проверяются конкретные пограничные случаи | Проверяются инварианты на всех входах |
| Проверяются сообщения об ошибках/коды | Свойства «туда и обратно» (разбор ↔ форматирование) |
| Интеграционные и тесты с моками | Сравнение двух реализаций |
| Поведение зависит от точных значений | «Для всех X выполняется свойство P» |

---

## Интеграционные тесты: каталог `tests/`

Модульные тесты живут внутри `src/` с `#[cfg(test)]`. Интеграционные тесты находятся в отдельном каталоге `tests/` и проверяют **публичный API** крейта — так же, как интеграционные тесты в C# ссылаются на проект как на внешнюю сборку.

```
my_crate/
├── src/
│   ├── lib.rs          // публичный API
│   └── internal.rs     // приватная реализация
├── tests/
│   ├── smoke.rs        // каждый файл — отдельный тестовый бинарник
│   ├── api_tests.rs
│   └── common/
│       └── mod.rs      // общие вспомогательные функции для тестов
└── Cargo.toml
```

### Написание интеграционных тестов

Каждый файл в `tests/` компилируется как отдельный крейт, который зависит от вашей библиотеки:

```rust
// tests/smoke.rs — имеет доступ только к pub-элементам my_crate
use my_crate::{process_order, Order, OrderResult};

#[test]
fn process_valid_order_returns_confirmation() {
    let order = Order::new("SKU-001", 3);
    let result = process_order(order);
    assert!(matches!(result, OrderResult::Confirmed { .. }));
}
```

### Общие вспомогательные функции для тестов

Общий код настройки кладите в `tests/common/mod.rs` (а не `tests/common.rs`, который будет считаться отдельным тестовым файлом):

```rust
// tests/common/mod.rs
use my_crate::Config;

pub fn test_config() -> Config {
    Config::builder()
        .database_url("sqlite::memory:")
        .build()
        .expect("test config must be valid")
}
```

```rust
// tests/api_tests.rs
mod common;

use my_crate::App;

#[test]
fn app_starts_with_test_config() {
    let config = common::test_config();
    let app = App::new(config);
    assert!(app.is_healthy());
}
```

### Запуск отдельных типов тестов

```bash
cargo test                  # запустить все тесты (модульные + интеграционные)
cargo test --lib            # только модульные тесты (как dotnet test --filter Category=Unit)
cargo test --test smoke     # запустить только tests/smoke.rs
cargo test --test api_tests # запустить только tests/api_tests.rs
```

**Ключевое отличие от C#:** файлы интеграционных тестов видят только `pub`-API вашего крейта. Приватные функции невидимы — это заставляет тестировать через публичный интерфейс, что обычно даёт лучший дизайн тестов.

***


