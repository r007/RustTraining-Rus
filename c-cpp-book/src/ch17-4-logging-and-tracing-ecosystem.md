## Экосистема логирования и трассировки: syslog/printf → `log` + `tracing`

> **Что вы узнаете:** двухуровневую архитектуру логирования в Rust (фасад + бэкенд), крейты `log` и `tracing`, структурированное логирование со спанами и то, чем это заменяет отладку через `printf`/`syslog`.

Диагностический код на C++ обычно использует `printf`, `syslog` или собственные фреймворки логирования.
В Rust есть стандартизированная двухуровневая архитектура логирования: крейт-**фасад** (`log` или
`tracing`) и **бэкенд** (собственно реализация логгера).

### Фасад `log` — универсальный API логирования в Rust

Крейт `log` предоставляет макросы, которые повторяют уровни серьёзности syslog. Библиотеки используют
макросы `log`, а бинарные крейты выбирают бэкенд:

```rust
// Cargo.toml
// [dependencies]
// log = "0.4"
// env_logger = "0.11"    # Один из многих бэкендов

use log::{info, warn, error, debug, trace};

fn check_sensor(id: u32, temp: f64) {
    trace!("Reading sensor {id}");           // Самая мелкая детализация
    debug!("Sensor {id} raw value: {temp}"); // Подробности для разработки
    
    if temp > 85.0 {
        warn!("Sensor {id} high temperature: {temp}°C");
    }
    if temp > 95.0 {
        error!("Sensor {id} CRITICAL: {temp}°C — initiating shutdown");
    }
    info!("Sensor {id} check complete");     // Штатная работа
}

fn main() {
    // Инициализация бэкенда — обычно делается один раз в main()
    env_logger::init();  // Управляется переменной окружения RUST_LOG

    check_sensor(0, 72.5);
    check_sensor(1, 91.0);
}
```

```bash
# Управление уровнем логирования через переменную окружения
RUST_LOG=debug cargo run          # Показать debug и выше
RUST_LOG=warn cargo run           # Показать только warn и error
RUST_LOG=my_crate=trace cargo run # Фильтрация по модулям
RUST_LOG=my_crate::gpu=debug,warn cargo run  # Разные уровни для разных частей
```

### Сравнение с C++

| C++ | Rust (`log`) | Примечания |
|-----|-------------|-------|
| `printf("DEBUG: %s\n", msg)` | `debug!("{msg}")` | Формат проверяется на этапе компиляции |
| `syslog(LOG_ERR, "...")` | `error!("...")` | Бэкенд решает, куда попадёт вывод |
| `#ifdef DEBUG` вокруг вызовов логирования | `trace!` / `debug!` удаляются при max_level | Нулевая стоимость, когда отключено |
| Собственный `Logger::log(level, msg)` | `log::info!("...")` — все крейты используют один API | Универсальный фасад, заменяемый бэкенд |
| Детализация логов для каждого файла | `RUST_LOG=crate::module=level` | Настраивается через окружение, без перекомпиляции |

### Крейт `tracing` — структурированное логирование со спанами

`tracing` расширяет `log` **структурированными полями** и **спанами** (ограниченными по времени областями).
Это особенно полезно в диагностическом коде, где нужно отслеживать контекст:

```rust
// Cargo.toml
// [dependencies]
// tracing = "0.1"
// tracing-subscriber = { version = "0.3", features = ["env-filter"] }

use tracing::{info, warn, error, instrument, info_span};

#[instrument(skip(data), fields(gpu_id = gpu_id, data_len = data.len()))]
fn run_gpu_test(gpu_id: u32, data: &[u8]) -> Result<(), String> {
    info!("Starting GPU test");

    let span = info_span!("ecc_check", gpu_id);
    let _guard = span.enter();  // Все логи внутри этой области включают gpu_id

    if data.is_empty() {
        error!(gpu_id, "No test data provided");
        return Err("empty data".to_string());
    }

    // Структурированные поля — пригодны для машинного разбора, а не просто подстановка строк
    info!(
        gpu_id,
        temp_celsius = 72.5,
        ecc_errors = 0,
        "ECC check passed"
    );

    Ok(())
}

fn main() {
    // Инициализация подписчика tracing
    tracing_subscriber::fmt()
        .with_env_filter("debug")  // Или используйте переменную окружения RUST_LOG
        .with_target(true)          // Показывать путь модуля
        .with_thread_ids(true)      // Показывать ID потоков
        .init();

    let _ = run_gpu_test(0, &[1, 2, 3]);
}
```

Вывод `tracing-subscriber`:
```rust
2026-02-15T10:30:00.123Z DEBUG ThreadId(01) run_gpu_test{gpu_id=0 data_len=3}: my_crate: Starting GPU test
2026-02-15T10:30:00.124Z  INFO ThreadId(01) run_gpu_test{gpu_id=0 data_len=3}:ecc_check{gpu_id=0}: my_crate: ECC check passed gpu_id=0 temp_celsius=72.5 ecc_errors=0
```

### `#[instrument]` — автоматическое создание спанов

Атрибут `#[instrument]` автоматически создаёт спан с именем функции
и её аргументами:

```rust
use tracing::instrument;

#[instrument]
fn parse_sel_record(record_id: u16, sensor_type: u8, data: &[u8]) -> Result<(), String> {
    // Каждый лог внутри этой функции автоматически включает:
    // record_id, sensor_type и data (если реализован Debug)
    tracing::debug!("Parsing SEL record");
    Ok(())
}

// skip: исключить из спана большие/чувствительные аргументы
// fields: добавить вычисляемые поля
#[instrument(skip(raw_buffer), fields(buf_len = raw_buffer.len()))]
fn decode_ipmi_response(raw_buffer: &[u8]) -> Result<Vec<u8>, String> {
    tracing::trace!("Decoding {} bytes", raw_buffer.len());
    Ok(raw_buffer.to_vec())
}
```

### `log` или `tracing` — что выбрать

| Аспект | `log` | `tracing` |
|--------|-------|-----------|
| **Сложность** | Простой — 5 макросов | Богаче — спаны, поля, instrument |
| **Структурированные данные** | Только подстановка строк | Поля ключ-значение: `info!(gpu_id = 0, "msg")` |
| **Время / спаны** | Нет | Да — `#[instrument]`, `span.enter()` |
| **Поддержка async** | Базовая | Полноценная — спаны передаются через `.await` |
| **Совместимость** | Универсальный фасад | Совместим с `log` (есть мост `log`) |
| **Когда использовать** | Простые приложения, библиотеки | Диагностические инструменты, async-код, наблюдаемость |

> **Рекомендация**: используйте `tracing` для продакшн-проектов диагностического характера
> (диагностические инструменты со структурированным выводом). Используйте `log` для простых библиотек, где
> нужно минимум зависимостей. `tracing` включает слой совместимости, так что библиотеки,
> использующие макросы `log`, продолжат работать с подписчиком `tracing`.

### Варианты бэкендов

| Крейт бэкенда | Вывод | Сценарий использования |
|--------------|--------|----------|
| `env_logger` | stderr, цветной | Разработка, простые CLI-инструменты |
| `tracing-subscriber` | stderr, форматированный | Продакшн с `tracing` |
| `syslog` | Системный syslog | Системные службы Linux |
| `tracing-journald` | Журнал systemd | Службы под управлением systemd |
| `tracing-appender` | Ротируемые файлы логов | Долгоживущие демоны |
| `tracing-opentelemetry` | Коллектор OpenTelemetry | Распределённая трассировка |

----

