## Путь обучения и следующие шаги

> **Что вы узнаете:** структурированную дорожную карту обучения (недели 1–2, месяцы 1–3+), рекомендуемые книги и ресурсы,
> типичные ловушки для разработчиков C# (путаница с владением, борьба с проверщиком заимствований),
> а также структурированную наблюдаемость с `tracing` против `ILogger`.
>
> **Сложность:** 🟢 Начальный

### Ближайшие шаги (недели 1–2)
1. **Настройте окружение**
   - Установите Rust через [rustup.rs](https://rustup.rs/)
   - Настройте VS Code с расширением rust-analyzer
   - Создайте первый проект `cargo new hello_world`

2. **Освойте основы**
   - Практикуйте владение на простых упражнениях
   - Пишите функции с разными типами параметров (`&str`, `String`, `&mut`)
   - Реализуйте простые структуры и методы

3. **Практика обработки ошибок**
   - Переведите код C# с try-catch на паттерны на основе Result
   - Практикуйтесь с оператором `?` и выражениями `match`
   - Реализуйте собственные типы ошибок

### Промежуточные цели (месяцы 1–2)
1. **Коллекции и итераторы**
   - Освойте `Vec<T>`, `HashMap<K,V>` и `HashSet<T>`
   - Изучите методы итераторов: `map`, `filter`, `collect`, `fold`
   - Практикуйтесь: циклы `for` против цепочек итераторов

2. **Трейты и обобщения**
   - Реализуйте распространённые трейты: `Debug`, `Clone`, `PartialEq`
   - Пишите обобщённые функции и структуры
   - Разберитесь с границами трейтов и предложениями where

3. **Структура проекта**
   - Организуйте код в модули
   - Разберитесь с видимостью `pub`
   - Работайте с внешними крейтами с crates.io

### Продвинутые темы (месяцы 3 и далее)
1. **Конкурентность**
   - Изучите трейты `Send` и `Sync`
   - Используйте `std::thread` для базового параллелизма
   - Изучите `tokio` для асинхронного программирования

2. **Управление памятью**
   - Разберитесь с `Rc<T>` и `Arc<T>` для разделяемого владения
   - Изучите, когда использовать `Box<T>` для выделения в куче
   - Освойте времена жизни для сложных сценариев

3. **Реальные проекты**
   - Напишите CLI-утилиту с `clap`
   - Создайте веб-API с `axum` или `warp`
   - Напишите библиотеку и опубликуйте её на crates.io

### Рекомендуемые ресурсы для обучения

#### Книги
- **«The Rust Programming Language»** (бесплатно онлайн) — официальная книга
- **«Rust by Example»** (бесплатно онлайн) — примеры для практики
- **«Programming Rust»** Джима Блэнди — глубокое техническое изложение

#### Онлайн-ресурсы
- [Rust Playground](https://play.rust-lang.org/) — попробовать код в браузере
- [Rustlings](https://github.com/rust-lang/rustlings) — интерактивные упражнения
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — практические примеры

#### Практические проекты
1. **Калькулятор командной строки** — практика с перечислениями и сопоставлением с образцом
2. **Организатор файлов** — работа с файловой системой и обработкой ошибок
3. **Обработчик JSON** — изучение serde и преобразования данных
4. **HTTP-сервер** — понимание асинхронного программирования и сетей
5. **Библиотека для работы с базой данных** — освоение трейтов, обобщений и обработки ошибок

### Типичные ловушки для разработчиков C#

#### Путаница с владением
```rust
// НЕ НАДО: пытаться использовать перемещённые значения
fn wrong_way() {
    let s = String::from("hello");
    takes_ownership(s);
    // println!("{}", s); // ОШИБКА: s было перемещено
}

// НАДО: используйте ссылки или клонируйте при необходимости
fn right_way() {
    let s = String::from("hello");
    borrows_string(&s);
    println!("{}", s); // OK: s здесь по-прежнему принадлежит нам
}

fn takes_ownership(s: String) { /* s перемещается сюда */ }
fn borrows_string(s: &str) { /* s заимствуется сюда */ }
```

#### Борьба с проверщиком заимствований
```rust
// НЕ НАДО: несколько изменяемых ссылок одновременно
fn wrong_borrowing() {
    let mut v = vec![1, 2, 3];
    let r1 = &mut v;
    // let r2 = &mut v; // ОШИБКА: нельзя заимствовать как изменяемое больше одного раза
}

// НАДО: ограничивайте область видимости изменяемых заимствований
fn right_borrowing() {
    let mut v = vec![1, 2, 3];
    {
        let r1 = &mut v;
        r1.push(4);
    } // r1 выходит из области видимости здесь
    
    let r2 = &mut v; // OK: других изменяемых заимствований нет
    r2.push(5);
}
```

#### Ожидание значений null
```rust
// НЕ НАДО: ожидать поведения, похожего на null
fn no_null_in_rust() {
    // let s: String = null; // В Rust НЕТ null!
}

// НАДО: явно используйте Option<T>
fn use_option_instead() {
    let maybe_string: Option<String> = None;
    
    match maybe_string {
        Some(s) => println!("Got string: {}", s),
        None => println!("No string available"),
    }
}
```

### Финальные советы

1. **Прислушивайтесь к компилятору** — ошибки компилятора Rust полезны, а не враждебны
2. **Начинайте с малого** — начните с простых программ и постепенно усложняйте
3. **Читайте чужой код** — изучайте популярные крейты на GitHub
4. **Просите помощи** — сообщество Rust дружелюбно и отзывчиво
5. **Практикуйтесь регулярно** — концепции Rust становятся естественными с практикой

Помните: у Rust крутая кривая обучения, но она окупается безопасностью памяти, производительностью и конкурентностью без страха. Система владения, которая сначала кажется ограничивающей, становится мощным инструментом для написания корректных и эффективных программ.

---

**Поздравляем!** Теперь у вас есть прочная основа для перехода с C# на Rust. Начинайте с простых проектов, будьте терпеливы в процессе обучения и постепенно переходите к более сложным приложениям. Преимущества Rust в безопасности и производительности делают первоначальные инвестиции в обучение оправданными.


<!-- ch16.2a: Structured Observability with tracing -->
## Структурированная наблюдаемость: `tracing` против ILogger и Serilog

Разработчики C# привыкли к **структурированному логированию** через `ILogger`, **Serilog** или **NLog** — где сообщения журнала несут типизированные пары ключ-значение. Крейт `log` в Rust даёт базовое логирование с уровнями, но **`tracing`** — производственный стандарт структурированной наблюдаемости со спанами, поддержкой асинхронности и распределённой трассировкой.

### Почему `tracing`, а не `log`

| Возможность | Крейт `log` | Крейт `tracing` | Аналог в C# |
|---------|------------|-----------------|----------------|
| Сообщения с уровнями | ✅ `info!()`, `error!()` | ✅ `info!()`, `error!()` | `ILogger.LogInformation()` |
| Структурированные поля | ❌ Только интерполяция строк | ✅ Типизированные пары ключ-значение | Serilog `Log.Information("{User}", user)` |
| Спаны (ограниченный контекст) | ❌ | ✅ `#[instrument]`, `span!()` | `ILogger.BeginScope()` |
| Поддержка асинхронности | ❌ Контекст теряется на `.await` | ✅ Спаны сохраняются через `.await` | `Activity` / `DiagnosticSource` |
| Распределённая трассировка | ❌ | ✅ Интеграция с OpenTelemetry | `System.Diagnostics.Activity` |
| Несколько форматов вывода | Базовый | JSON, pretty, compact, OTLP | Приёмники (sinks) Serilog |

### Начало работы
```toml
# Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

### Базовое использование: структурированное логирование
```csharp
// Serilog в C#
Log.Information("Processing order {OrderId} for {Customer}, total {Total:C}",
    orderId, customer.Name, order.Total);
// Вывод: Processing order 12345 for Alice, total $99.95
// JSON:  {"OrderId": 12345, "Customer": "Alice", "Total": 99.95, ...}
```

```rust
use tracing::{info, warn, error, debug, instrument};

// Структурированные поля — типизированные, а не интерполированные в строку
info!(order_id = 12345, customer = "Alice", total = 99.95,
      "Processing order");
// Вывод: INFO Processing order order_id=12345 customer="Alice" total=99.95
// JSON:  {"order_id": 12345, "customer": "Alice", "total": 99.95, ...}

// Динамические значения
let order_id = 12345;
info!(order_id, "Order received");  // сокращение: имя поля = имя переменной

// Условные поля
if let Some(promo) = promo_code {
    info!(order_id, promo_code = %promo, "Promo applied");
    //                        ^ % означает использовать форматирование Display
    //                        ? использовал бы форматирование Debug
}
```

### Спаны: ключевая возможность для асинхронного кода

Спаны — это ограниченные контексты, которые переносят поля через вызовы функций и точки `.await`. Это как `ILogger.BeginScope()`, но безопасно для асинхронного кода.

```csharp
// C# — Activity / BeginScope
using var activity = new Activity("ProcessOrder").Start();
activity.SetTag("order_id", orderId);

using (_logger.BeginScope(new Dictionary<string, object> { ["OrderId"] = orderId }))
{
    _logger.LogInformation("Starting processing");
    await ProcessPaymentAsync();
    _logger.LogInformation("Payment complete");  // OrderId всё ещё в области видимости
}
```

```rust
use tracing::{info, instrument, Instrument};

// #[instrument] автоматически создаёт спан с аргументами функции в качестве полей
#[instrument(skip(db), fields(customer_name))]
async fn process_order(order_id: u64, db: &Database) -> Result<(), AppError> {
    let order = db.get_order(order_id).await?;
    
    // Динамически добавляем поле в текущий спан
    tracing::Span::current().record("customer_name", &order.customer_name.as_str());
    
    info!("Starting processing");
    process_payment(&order).await?;        // контекст спана сохраняется через .await!
    info!(items = order.items.len(), "Payment complete");
    Ok(())
}
// Каждое сообщение журнала внутри этой функции автоматически содержит:
//   order_id=12345 customer_name="Alice"
// Даже во вложенных асинхронных вызовах!

// Ручное создание спана (как BeginScope)
async fn batch_process(orders: Vec<u64>, db: &Database) {
    for order_id in orders {
        let span = tracing::info_span!("process_order", order_id);
        
        // .instrument(span) прикрепляет спан к future
        process_order(order_id, db)
            .instrument(span)
            .await
            .unwrap_or_else(|e| error!("Failed: {e}"));
    }
}
```

### Настройка подписчика (аналог приёмников Serilog)

```rust
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() {
    // Разработка: читаемый человеком цветной вывод
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "my_app=debug,tower_http=info".into()))
        .with(fmt::layer().pretty())  // Цветные, с отступами спаны
        .init();
}

fn init_tracing_production() {
    // Продакшен: вывод в JSON для агрегации логов (как JSON-приёмник Serilog)
    tracing_subscriber::registry()
        .with(EnvFilter::new("my_app=info"))
        .with(fmt::layer().json())  // Структурированный JSON
        .init();
    // Вывод: {"timestamp":"...","level":"INFO","fields":{"order_id":123},...}
}
```

```bash
# Управление уровнями логирования через переменную окружения (как MinimumLevel в Serilog)
RUST_LOG=my_app=debug,hyper=warn cargo run
RUST_LOG=trace cargo run  # всё подряд
```

### Шпаргалка по переходу с Serilog на tracing

| Serilog / ILogger | tracing | Примечания |
|-------------------|---------|-------|
| `Log.Information("{Key}", val)` | `info!(key = val, "message")` | Поля типизированы, не интерполируются |
| `Log.ForContext("Key", val)` | `span.record("key", val)` | Добавляет поля в текущий спан |
| `using BeginScope(...)` | `#[instrument]` или `info_span!()` | Автоматически с `#[instrument]` |
| `.WriteTo.Console()` | `fmt::layer()` | Читаемый человеком вывод |
| `.WriteTo.Seq()` / `.File()` | `fmt::layer().json()` + перенаправление в файл | Или используйте `tracing-appender` |
| `.Enrich.WithProperty()` | `span!(Level::INFO, "name", key = val)` | Поля спана |
| `LogEventLevel.Debug` | `tracing::Level::DEBUG` | Та же концепция |
| Деструктуризация `{@Object}` | `field = ?value` (Debug) или `%value` (Display) | `?` = Debug, `%` = Display |

### Интеграция с OpenTelemetry
```toml
# Для распределённой трассировки (как System.Diagnostics + экспортёр OTLP)
[dependencies]
tracing-opentelemetry = "0.22"
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"
```

```rust
// Добавляем слой OpenTelemetry вместе с выводом в консоль
use tracing_opentelemetry::OpenTelemetryLayer;

fn init_otel() {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to create OTLP tracer");

    tracing_subscriber::registry()
        .with(OpenTelemetryLayer::new(tracer))  // Отправляем спаны в Jaeger/Tempo
        .with(fmt::layer())                      // Также выводим в консоль
        .init();
}
// Теперь спаны от #[instrument] автоматически становятся распределёнными трассами!
```

***


