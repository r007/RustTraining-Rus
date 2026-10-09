## Паттерны тестирования для программистов C++

> **Что вы узнаете:** встроенный тестовый фреймворк Rust — `#[test]`, `#[should_panic]`, тесты, возвращающие `Result`, паттерны-билдеры для тестовых данных, мокирование на основе трейтов, property-based тестирование с `proptest`, snapshot-тестирование с `insta` и организацию интеграционных тестов. Тестирование без настройки, которое заменяет Google Test + CMake.

Тестирование в C++ обычно опирается на внешние фреймворки (Google Test, Catch2, Boost.Test)
со сложной интеграцией в сборку. Тестовый фреймворк Rust **встроен в язык
и тулчейн** — никаких зависимостей, никакой интеграции с CMake, никакой настройки тестового раннера.

### Атрибуты тестов помимо `#[test]`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pass() {
        assert_eq!(2 + 2, 4);
    }

    // Ожидаем panic — аналог EXPECT_DEATH в GTest
    #[test]
    #[should_panic]
    fn out_of_bounds_panics() {
        let v = vec![1, 2, 3];
        let _ = v[10]; // Panic — тест проходит
    }

    // Ожидаем panic с определённой подстрокой в сообщении
    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn specific_panic_message() {
        let v = vec![1, 2, 3];
        let _ = v[10];
    }

    // Тесты, возвращающие Result<(), E> — используйте ? вместо unwrap()
    #[test]
    fn test_with_result() -> Result<(), String> {
        let value: u32 = "42".parse().map_err(|e| format!("{e}"))?;
        assert_eq!(value, 42);
        Ok(())
    }

    // По умолчанию пропускаем медленные тесты — запуск через `cargo test -- --ignored`
    #[test]
    #[ignore]
    fn slow_integration_test() {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
```

```bash
cargo test                          # Запустить все не игнорируемые тесты
cargo test -- --ignored             # Запустить только игнорируемые тесты
cargo test -- --include-ignored     # Запустить ВСЕ тесты, включая игнорируемые
cargo test test_name                # Запустить тесты, имя которых соответствует шаблону
cargo test -- --nocapture           # Показывать вывод println! во время тестов
cargo test -- --test-threads=1      # Запускать тесты последовательно (для общего состояния)
```

### Вспомогательные функции для тестов: паттерн «строитель» для тестовых данных

В C++ вы бы использовали фикстуры Google Test (`class MyTest : public ::testing::Test`).
В Rust используйте функции-билдеры или трейт `Default`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Функция-билдер — создаёт тестовые данные с разумными значениями по умолчанию
    fn make_gpu_event(severity: Severity, fault_code: u32) -> DiagEvent {
        DiagEvent {
            source: "accel_diag".to_string(),
            severity,
            message: format!("Test event FC:{fault_code}"),
            fault_code,
        }
    }

    // Переиспользуемая фикстура — набор заранее созданных событий
    fn sample_events() -> Vec<DiagEvent> {
        vec![
            make_gpu_event(Severity::Critical, 67956),
            make_gpu_event(Severity::Warning, 32709),
            make_gpu_event(Severity::Info, 10001),
        ]
    }

    #[test]
    fn filter_critical_events() {
        let events = sample_events();
        let critical: Vec<_> = events.iter()
            .filter(|e| e.severity == Severity::Critical)
            .collect();
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].fault_code, 67956);
    }
}
```

### Мокирование с помощью трейтов

В C++ для мокирования нужны фреймворки вроде Google Mock или ручные переопределения virtual-методов.
В Rust вы определяете трейт для зависимости и подменяете реализации в тестах:

```rust
// Продакшн-трейт
trait SensorReader {
    fn read_temperature(&self, sensor_id: u32) -> Result<f64, String>;
}

// Продакшн-реализация
struct HwSensorReader;
impl SensorReader for HwSensorReader {
    fn read_temperature(&self, sensor_id: u32) -> Result<f64, String> {
        // Реальный вызов оборудования...
        Ok(72.5)
    }
}

// Тестовый мок — возвращает предсказуемые значения
#[cfg(test)]
struct MockSensorReader {
    temperatures: std::collections::HashMap<u32, f64>,
}

#[cfg(test)]
impl SensorReader for MockSensorReader {
    fn read_temperature(&self, sensor_id: u32) -> Result<f64, String> {
        self.temperatures.get(&sensor_id)
            .copied()
            .ok_or_else(|| format!("Unknown sensor {sensor_id}"))
    }
}

// Тестируемая функция — обобщённая по типу reader
fn check_overtemp(reader: &impl SensorReader, ids: &[u32], threshold: f64) -> Vec<u32> {
    ids.iter()
        .filter(|&&id| reader.read_temperature(id).unwrap_or(0.0) > threshold)
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_overtemp_sensors() {
        let mut mock = MockSensorReader { temperatures: Default::default() };
        mock.temperatures.insert(0, 72.5);
        mock.temperatures.insert(1, 91.0);  // Выше порога
        mock.temperatures.insert(2, 65.0);

        let hot = check_overtemp(&mock, &[0, 1, 2], 80.0);
        assert_eq!(hot, vec![1]);
    }
}
```

### Временные файлы и каталоги в тестах

В C++ тесты часто используют платформо-специфичные временные каталоги. В Rust есть крейт `tempfile`:

```rust
// Cargo.toml: [dev-dependencies]
// tempfile = "3"

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn parse_config_from_file() -> Result<(), Box<dyn std::error::Error>> {
        // Создаём временный файл, который удаляется автоматически при уничтожении
        let mut file = NamedTempFile::new()?;
        writeln!(file, r#"{{"sku": "ServerNode", "level": "Quick"}}"#)?;

        let config = load_config(file.path().to_str().unwrap())?;
        assert_eq!(config.sku, "ServerNode");
        Ok(())
        // file удаляется здесь — код очистки не нужен
    }
}
```

### Property-based тестирование с `proptest`

Вместо написания конкретных тестовых случаев опишите **свойства**, которые должны выполняться
для всех входных данных. `proptest` генерирует случайные входные данные и находит минимальные падающие примеры:

```rust
// Cargo.toml: [dev-dependencies]
// proptest = "1"

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    fn parse_and_format(n: u32) -> String {
        format!("{n}")
    }

    proptest! {
        #[test]
        fn roundtrip_u32(n: u32) {
            let formatted = parse_and_format(n);
            let parsed: u32 = formatted.parse().unwrap();
            prop_assert_eq!(n, parsed);
        }

        #[test]
        fn string_contains_no_null(s in "[a-zA-Z0-9 ]{0,100}") {
            prop_assert!(!s.contains('\0'));
        }
    }
}
```

### Snapshot-тестирование с `insta`

Для тестов, которые выдают сложный вывод (JSON, отформатированные строки), `insta` автоматически создаёт
и поддерживает эталонные снимки:

```rust
// Cargo.toml: [dev-dependencies]
// insta = { version = "1", features = ["json"] }

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    #[test]
    fn der_entry_format() {
        let entry = DerEntry {
            fault_code: 67956,
            component: "GPU".to_string(),
            message: "ECC error detected".to_string(),
        };
        // Первый запуск: создаёт файл снимка в tests/snapshots/
        // Последующие запуски: сравнивают с сохранённым снимком
        assert_json_snapshot!(entry);
    }
}
```

```bash
cargo insta test              # Запустить тесты и просмотреть новые/изменённые снимки
cargo insta review            # Интерактивный просмотр изменений снимков
```

### Сравнение тестирования C++ и Rust

| **C++ (Google Test)** | **Rust** | **Примечания** |
|----------------------|---------|----------|
| `TEST(Suite, Name) { }` | `#[test] fn name() { }` | Иерархия suite/class не нужна |
| `ASSERT_EQ(a, b)` | `assert_eq!(a, b)` | Встроенный макрос, фреймворк не нужен |
| `ASSERT_NEAR(a, b, eps)` | `assert!((a - b).abs() < eps)` | Или используйте крейт `approx` |
| `EXPECT_THROW(expr, type)` | `#[should_panic(expected = "...")]` | Или `catch_unwind` для тонкого контроля |
| `EXPECT_DEATH(expr, "msg")` | `#[should_panic(expected = "msg")]` | |
| `class Fixture : public ::testing::Test` | Функции-билдеры + `Default` | Наследование не нужно |
| Google Mock `MOCK_METHOD` | Трейт + тестовая реализация | Более явно, без магии макросов |
| `INSTANTIATE_TEST_SUITE_P` (параметризованные) | `proptest!` или тесты, сгенерированные макросом | |
| `SetUp()` / `TearDown()` | RAII через `Drop` — очистка автоматическая | Переменные уничтожаются в конце теста |
| Отдельный тестовый бинарник + CMake | `cargo test` — без настройки | |
| `ctest --output-on-failure` | `cargo test -- --nocapture` | |

----

### Интеграционные тесты: каталог `tests/`

Модульные тесты находятся внутри модулей `#[cfg(test)]` рядом с вашим кодом. **Интеграционные тесты** находятся в отдельном каталоге `tests/` в корне крейта и проверяют публичный API вашей библиотеки так, как это делал бы внешний потребитель:

```
my_crate/
├── src/
│   └── lib.rs          # Код вашей библиотеки
├── tests/
│   ├── smoke.rs        # Каждый .rs-файл — отдельный тестовый бинарник
│   ├── regression.rs
│   └── common/
│       └── mod.rs      # Общие вспомогательные функции для тестов (НЕ сам тест)
└── Cargo.toml
```

```rust
// tests/smoke.rs — тестирует ваш крейт так, как это делал бы внешний пользователь
use my_crate::DiagEngine;  // Доступен только публичный API

#[test]
fn engine_starts_successfully() {
    let engine = DiagEngine::new("test_config.json");
    assert!(engine.is_ok());
}

#[test]
fn engine_rejects_invalid_config() {
    let engine = DiagEngine::new("nonexistent.json");
    assert!(engine.is_err());
}
```

```rust
// tests/common/mod.rs — общие помощники, НЕ компилируются как тестовый бинарник
pub fn setup_test_environment() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.json"), r#"{"log_level": "debug"}"#).unwrap();
    dir
}
```

```rust
// tests/regression.rs — может использовать общие помощники
mod common;

#[test]
fn regression_issue_42() {
    let env = common::setup_test_environment();
    let engine = my_crate::DiagEngine::new(
        env.path().join("config.json").to_str().unwrap()
    );
    assert!(engine.is_ok());
}
```

**Запуск интеграционных тестов:**
```bash
cargo test                          # Запускает и модульные, и интеграционные тесты
cargo test --test smoke             # Запустить только tests/smoke.rs
cargo test --test regression        # Запустить только tests/regression.rs
cargo test --lib                    # Запустить ТОЛЬКО модульные тесты (без интеграционных)
```

> **Ключевое отличие от модульных тестов**: интеграционные тесты не могут обращаться к приватным функциям или элементам `pub(crate)`. Это заставляет проверять, что публичного API достаточно — полезный сигнал для дизайна. В терминах C++ это похоже на тестирование только по публичному заголовку без доступа `friend`.

----

