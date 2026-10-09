## Основные инструменты Rust для разработчиков C#

> **Что вы узнаете:** инструменты разработки Rust и их аналоги в C# — Clippy (анализаторы Roslyn),
> rustfmt (dotnet format), cargo doc (XML-документация), cargo watch (dotnet watch) и расширения VS Code.
>
> **Сложность:** 🟢 Начальный

### Сравнение инструментов

| Инструмент C# | Аналог в Rust | Установка | Назначение |
|---------|----------------|---------|---------|
| Анализаторы Roslyn | **Clippy** | `rustup component add clippy` | Линтинг и подсказки по стилю |
| `dotnet format` | **rustfmt** | `rustup component add rustfmt` | Автоматическое форматирование |
| XML-комментарии к документации | **`cargo doc`** | Встроен | Генерация HTML-документации |
| OmniSharp / Roslyn | **rust-analyzer** | Расширение VS Code | Поддержка в IDE |
| `dotnet watch` | **cargo-watch** | `cargo install cargo-watch` | Автоматическая пересборка при сохранении |
| — | **cargo-expand** | `cargo install cargo-expand` | Просмотр раскрытия макросов |
| `dotnet audit` | **cargo-audit** | `cargo install cargo-audit` | Проверка уязвимостей безопасности |

### Clippy: ваш автоматический код-ревьюер
```bash
# Запуск Clippy для проекта
cargo clippy

# Считать предупреждения ошибками (CI/CD)
cargo clippy -- -D warnings

# Автоматическое применение подсказок
cargo clippy --fix
```

```rust
// Clippy находит сотни антипаттернов:

// До Clippy:
if x == true { }           // предупреждение: проверка равенства с bool
let _ = vec.len() == 0;    // предупреждение: используйте .is_empty()
for i in 0..vec.len() { }  // предупреждение: используйте .iter().enumerate()

// После применения подсказок Clippy:
if x { }
let _ = vec.is_empty();
for (i, item) in vec.iter().enumerate() { }
```

### rustfmt: единообразное форматирование
```bash
# Форматирование всех файлов
cargo fmt

# Проверка форматирования без изменений (CI/CD)
cargo fmt -- --check
```

```toml
# rustfmt.toml — настройка форматирования (как .editorconfig)
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
```

### cargo doc: генерация документации
```bash
# Генерация и открытие документации (включая зависимости)
cargo doc --open

# Запуск тестов документации
cargo test --doc
```

```rust
/// Вычисляет площадь круга.
///
/// # Аргументы
/// * `radius` — радиус круга (должен быть неотрицательным)
///
/// # Примеры
/// ```
/// let area = my_crate::circle_area(5.0);
/// assert!((area - 78.54).abs() < 0.01);
/// ```
///
/// # Паника
/// Паникует, если `radius` отрицательный.
pub fn circle_area(radius: f64) -> f64 {
    assert!(radius >= 0.0, "radius must be non-negative");
    std::f64::consts::PI * radius * radius
}
// Код внутри блоков /// ``` компилируется и запускается во время `cargo test`!
```

### cargo watch: автоматическая пересборка
```bash
# Пересборка при изменении файлов (как dotnet watch)
cargo watch -x check          # Только проверка типов (самый быстрый)
cargo watch -x test           # Запуск тестов при сохранении
cargo watch -x 'run -- args'  # Запуск программы при сохранении
cargo watch -x clippy         # Линтинг при сохранении
```

### cargo expand: просмотр того, что генерируют макросы
```bash
# Просмотр раскрытого вывода производных макросов
cargo expand --lib            # Раскрыть lib.rs
cargo expand module_name      # Раскрыть конкретный модуль
```

### Рекомендуемые расширения VS Code

| Расширение | Назначение |
|-----------|-----------|
| **rust-analyzer** | Автодополнение кода, встроенные ошибки, рефакторинг |
| **CodeLLDB** | Отладчик (как отладчик Visual Studio) |
| **Even Better TOML** | Подсветка синтаксиса Cargo.toml |
| **crates** | Показ последних версий крейтов в Cargo.toml |
| **Error Lens** | Вывод ошибок и предупреждений прямо в строке |

***

Для более глубокого изучения продвинутых тем, упомянутых в этом руководстве, см. сопутствующие учебные материалы:

- **[Паттерны Rust](../../rust-patterns-book/src/SUMMARY.md)** — проекции Pin, пользовательские аллокаторы, арена-паттерны, структуры данных без блокировок и продвинутые паттерны unsafe
- **[Async Rust Training](../../async-book/src/SUMMARY.md)** — подробно о tokio, безопасности отмены в асинхронном коде, обработке потоков и производственной архитектуре async
- **[Rust для программистов C/C++](../../c-cpp-book/src/SUMMARY.md)** — полезно, если в команде есть опыт работы с C++; рассматривает соответствие семантики перемещения, различия RAII и шаблоны против обобщений
- **[Rust для программистов C](../../c-cpp-book/src/SUMMARY.md)** — актуально для сценариев взаимодействия; рассматривает паттерны FFI, отладку встраиваемого Rust и программирование с `no_std`
