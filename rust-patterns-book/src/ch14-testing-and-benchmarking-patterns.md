# 14. Паттерны тестирования и бенчмаркинга 🟢

> **Что вы узнаете:**
> - Три уровня тестов в Rust: модульные, интеграционные и doc-тесты
> - Тестирование свойств (property-based testing) с proptest для поиска граничных случаев
> - Бенчмаркинг с criterion для надёжного измерения производительности
> - Стратегии моков без громоздких фреймворков

## Модульные тесты, интеграционные тесты, doc-тесты

В Rust есть три уровня тестирования, встроенные в язык:

```rust
// --- Модульные тесты: в том же файле, что и код ---
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial_zero() {
        // (1..=0).product() возвращает 1: единица умножения для пустого диапазона
        assert_eq!(factorial(0), 1);
    }

    #[test]
    fn test_factorial_five() {
        assert_eq!(factorial(5), 120);
    }

    #[test]
    #[cfg(debug_assertions)] // Проверки переполнения включены только в отладочном режиме
    #[should_panic(expected = "overflow")]
    fn test_factorial_overflow() {
        // ⚠️ Этот тест проходит только в отладочном режиме (проверки переполнения включены).
        // В релизном режиме (`cargo test --release`) арифметика u64 молча переполняется,
        // и паники не происходит. Для безопасности в релизе используйте `checked_mul`
        // или настройку профиля `overflow-checks = true`.
        factorial(100); // Должна паниковать при переполнении
    }

    #[test]
    fn test_with_result() -> Result<(), Box<dyn std::error::Error>> {
        // Тесты могут возвращать Result: внутри работает `?`!
        let value: u64 = "42".parse()?;
        assert_eq!(value, 42);
        Ok(())
    }
}
```

```rust
// --- Интеграционные тесты: в директории tests/ ---
// tests/integration_test.rs
// Эти тесты проверяют ТОЛЬКО публичный API вашего крейта

use my_crate::factorial;

#[test]
fn test_factorial_from_outside() {
    assert_eq!(factorial(10), 3_628_800);
}
```

```rust
// --- Doc-тесты: в комментариях к документации ---
/// Вычисляет факториал `n`.
///
/// # Примеры
///
/// ```
/// use my_crate::factorial;
/// assert_eq!(factorial(5), 120);
/// ```
///
/// # Паника
///
/// Паникует, если результат переполняет `u64`.
///
/// ```should_panic
/// my_crate::factorial(100);
/// ```
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}
// Doc-тесты компилируются и запускаются через `cargo test`: они не дают примерам устареть.
```

### Фикстуры и подготовка тестов

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Общая подготовка: вынесем в вспомогательную функцию
    fn setup_database() -> TestDb {
        let db = TestDb::new_in_memory();
        db.run_migrations();
        db.seed_test_data();
        db
    }

    #[test]
    fn test_user_creation() {
        let db = setup_database();
        let user = db.create_user("Alice", "alice@test.com").unwrap();
        assert_eq!(user.name, "Alice");
    }

    #[test]
    fn test_user_deletion() {
        let db = setup_database();
        db.create_user("Bob", "bob@test.com").unwrap();
        assert!(db.delete_user("Bob").is_ok());
        assert!(db.get_user("Bob").is_none());
    }

    // Очистка через Drop (RAII):
    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            // Cargo.toml: rand = "0.8"
            let path = std::env::temp_dir().join(format!("test_{}", rand::random::<u32>()));
            std::fs::create_dir_all(&path).unwrap();
            TempDir { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_file_operations() {
        let dir = TempDir::new(); // Создан
        std::fs::write(dir.path.join("test.txt"), "hello").unwrap();
        assert!(dir.path.join("test.txt").exists());
    } // dir уничтожается здесь → временный каталог удаляется
}
```

### Тестирование свойств (proptest)

Вместо проверки конкретных значений тестируйте *свойства*, которые должны выполняться всегда:

```rust
// Cargo.toml: proptest = "1"
use proptest::prelude::*;

fn reverse(v: &[i32]) -> Vec<i32> {
    v.iter().rev().cloned().collect()
}

proptest! {
    #[test]
    fn test_reverse_twice_is_identity(v in prop::collection::vec(any::<i32>(), 0..100)) {
        // Свойство: двойной разворот возвращает исходный вектор
        assert_eq!(reverse(&reverse(&v)), v);
    }

    #[test]
    fn test_reverse_preserves_length(v in prop::collection::vec(any::<i32>(), 0..100)) {
        // Свойство: разворот сохраняет длину
        assert_eq!(reverse(&v).len(), v.len());
    }

    #[test]
    fn test_sort_is_idempotent(mut v in prop::collection::vec(any::<i32>(), 0..100)) {
        v.sort();
        let sorted_once = v.clone();
        v.sort();
        assert_eq!(v, sorted_once); // Двойная сортировка = однократная сортировка
    }

    #[test]
    fn test_parse_roundtrip(x in any::<f64>().prop_filter("finite", |x| x.is_finite())) {
        // Свойство: форматирование, затем разбор возвращают то же значение
        let s = format!("{x}");
        let parsed: f64 = s.parse().unwrap();
        prop_assert!((x - parsed).abs() < f64::EPSILON);
    }
}
```

> **Когда использовать proptest**: когда вы тестируете функцию с большим пространством входных данных и хотите быть уверены, что она работает на граничных случаях, о которых вы не подумали. proptest генерирует сотни случайных входов и минимизирует найденные ошибки до минимального воспроизводящего примера.

### Бенчмаркинг с criterion

```rust
// Cargo.toml:
// [dev-dependencies]
// criterion = { version = "0.5", features = ["html_reports"] }
//
// [[bench]]
// name = "my_benchmarks"
// harness = false

// benches/my_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn bench_fibonacci(c: &mut Criterion) {
    c.bench_function("fibonacci 20", |b| {
        b.iter(|| fibonacci(black_box(20)))
    });

    // Сравниваем разные входные данные:
    let mut group = c.benchmark_group("fibonacci_compare");
    for size in [10, 15, 20, 25] {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            &size,
            |b, &size| b.iter(|| fibonacci(black_box(size))),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_fibonacci);
criterion_main!(benches);

// Запуск: cargo bench
// Создаёт HTML-отчёты в target/criterion/
```

### Стратегии моков без фреймворков

Система трейтов Rust даёт естественное внедрение зависимостей, и никакой фреймворк для моков не нужен:

```rust
// Описываем поведение через трейт
trait Clock {
    fn now(&self) -> std::time::Instant;
}

trait HttpClient {
    fn get(&self, url: &str) -> Result<String, String>;
}

// Реализации для продакшена
struct RealClock;
impl Clock for RealClock {
    fn now(&self) -> std::time::Instant { std::time::Instant::now() }
}

// Сервис зависит от абстракций
struct CacheService<C: Clock, H: HttpClient> {
    clock: C,
    client: H,
    ttl: std::time::Duration,
}

impl<C: Clock, H: HttpClient> CacheService<C, H> {
    fn fetch(&self, url: &str) -> Result<String, String> {
        // Использует self.clock и self.client: их можно подменить
        self.client.get(url)
    }
}

// Тест с моками-реализациями: никакой фреймворк не нужен!
#[cfg(test)]
mod tests {
    use super::*;

    struct MockClock {
        fixed_time: std::time::Instant,
    }
    impl Clock for MockClock {
        fn now(&self) -> std::time::Instant { self.fixed_time }
    }

    struct MockHttpClient {
        response: String,
    }
    impl HttpClient for MockHttpClient {
        fn get(&self, _url: &str) -> Result<String, String> {
            Ok(self.response.clone())
        }
    }

    #[test]
    fn test_cache_service() {
        let service = CacheService {
            clock: MockClock { fixed_time: std::time::Instant::now() },
            client: MockHttpClient { response: "cached data".into() },
            ttl: std::time::Duration::from_secs(300),
        };

        assert_eq!(service.fetch("http://example.com").unwrap(), "cached data");
    }
}
```

> **Философия тестирования**: в интеграционных тестах предпочитайте реальные зависимости, а в модульных тестах моки на основе трейтов. Не используйте фреймворки для моков, если граф зависимостей не сложный: обобщения трейтов справляются с большинством случаев естественным образом.

> **Ключевые выводы: тестирование**
> - Doc-тесты (`///`) одновременно служат документацией и регрессионными тестами: их компилируют и запускают
> - `proptest` генерирует случайные входные данные, чтобы находить граничные случаи, которые вы никогда не написали бы вручную
> - `criterion` даёт статистически обоснованные бенчмарки с HTML-отчётами
> - Мокайте через обобщения трейтов и тестовые двойники, а не через фреймворки для моков

> **См. также:** [гл. 13 — Макросы](ch13-macros-code-that-writes-code.md) о тестировании кода, сгенерированного макросами. [гл. 15 — Дизайн API](ch15-crate-architecture-and-api-design.md) о том, как структура модулей влияет на организацию тестов.

---

### Упражнение: тестирование свойств с proptest ★★ (~25 минут)

Напишите обёртку `SortedVec<T: Ord>`, которая поддерживает инвариант сортировки. Используйте `proptest`, чтобы проверить, что:
1. После любой последовательности вставок внутренний вектор всегда отсортирован
2. `contains()` даёт тот же результат, что и `Vec::contains()` из стандартной библиотеки
3. Длина равна количеству вставок

<details>
<summary>🔑 Решение</summary>

```rust,ignore
#[derive(Debug)]
struct SortedVec<T: Ord> {
    inner: Vec<T>,
}

impl<T: Ord> SortedVec<T> {
    fn new() -> Self { SortedVec { inner: Vec::new() } }

    fn insert(&mut self, value: T) {
        let pos = self.inner.binary_search(&value).unwrap_or_else(|p| p);
        self.inner.insert(pos, value);
    }

    fn contains(&self, value: &T) -> bool {
        self.inner.binary_search(value).is_ok()
    }

    fn len(&self) -> usize { self.inner.len() }
    fn as_slice(&self) -> &[T] { &self.inner }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn always_sorted(values in proptest::collection::vec(-1000i32..1000, 0..100)) {
            let mut sv = SortedVec::new();
            for v in &values {
                sv.insert(*v);
            }
            for w in sv.as_slice().windows(2) {
                prop_assert!(w[0] <= w[1]);
            }
            prop_assert_eq!(sv.len(), values.len());
        }

        #[test]
        fn contains_matches_stdlib(values in proptest::collection::vec(0i32..50, 1..30)) {
            let mut sv = SortedVec::new();
            for v in &values {
                sv.insert(*v);
            }
            for v in &values {
                prop_assert!(sv.contains(v));
            }
            prop_assert!(!sv.contains(&9999));
        }
    }
}
```

</details>

***
