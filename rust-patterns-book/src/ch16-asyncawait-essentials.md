# 16. Основы async/await 🔴

> **Что вы узнаете:**
> - Чем трейт `Future` в Rust отличается от горутин Go и asyncio в Python
> - Быстрый старт с Tokio: запуск задач, `join!` и настройка рантайма
> - Распространённые ошибки в async-коде и способы их исправления
> - Когда выносить блокирующую работу в `spawn_blocking`

## Future, рантаймы и `async fn`

Модель async в Rust *принципиально отличается* от горутин Go или `asyncio` в Python. Для начала достаточно понимать три концепции:

1. **`Future` это ленивый конечный автомат**: вызов `async fn` ничего не выполняет, он возвращает `Future`, который нужно опросить (poll).
2. **Для опроса `Future` нужен рантайм**: `tokio`, `async-std` или `smol`. Стандартная библиотека определяет `Future`, но рантайма не предоставляет.
3. **`async fn` это синтаксический сахар**: компилятор преобразует её в конечный автомат, который реализует `Future`.

```rust
// Future — это просто трейт:
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

// async fn раскрывается в:
// fn fetch_data(url: &str) -> impl Future<Output = Result<Vec<u8>, Error>>
async fn fetch_data(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let response = reqwest::get(url).await?;  // .await отдаёт управление, пока результат не готов
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}
```

### Быстрый старт с Tokio

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust,ignore
use tokio::time::{sleep, Duration};
use tokio::task;

#[tokio::main]
async fn main() {
    // Запускаем конкурентные задачи (как лёгкие потоки):
    let handle_a = task::spawn(async {
        sleep(Duration::from_millis(100)).await;
        "задача A выполнена"
    });

    let handle_b = task::spawn(async {
        sleep(Duration::from_millis(50)).await;
        "задача B выполнена"
    });

    // .await обе: они выполняются конкурентно, а не последовательно:
    let (a, b) = tokio::join!(handle_a, handle_b);
    println!("{}, {}", a.unwrap(), b.unwrap());
}
```

### Распространённые ошибки в async

| Ошибка | Почему возникает | Решение |
|--------|------------------|---------|
| Блокировка в async | `std::thread::sleep` или вычисления блокируют исполнитель | Используйте `tokio::task::spawn_blocking` или `rayon` |
| Ошибки границы `Send` | Future удерживает через `.await` значение `!Send` (например, `Rc`, `MutexGuard`) | Перестройте код так, чтобы не-Send значения уничтожались до `.await` |
| Future не опрошен | Вызов `async fn` без `.await` или запуска: ничего не происходит | Всегда делайте `.await` или `tokio::spawn` для возвращённого future |
| Удержание `MutexGuard` через `.await` | `std::sync::MutexGuard` это `!Send`; async-задача может продолжиться в другом потоке | Используйте `tokio::sync::Mutex` или уничтожьте защитника до `.await` |
| Непреднамеренно последовательное выполнение | `let a = foo().await; let b = bar().await;` выполняется последовательно | Используйте `tokio::join!` или `tokio::spawn` для конкурентности |

```rust
// ❌ Блокировка async-исполнителя:
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(5)); // Блокирует весь поток!
}

// ✅ Выносим блокирующую работу:
async fn good() {
    tokio::task::spawn_blocking(|| {
        std::thread::sleep(std::time::Duration::from_secs(5)); // Выполняется в пуле для блокирующих задач
    }).await.unwrap();
}
```

> **Полное описание async**: о `Stream`, `select!`, безопасности отмены, структурированной конкурентности и middleware `tower` см. в нашем отдельном руководстве **Async Rust Training**. Этот раздел охватывает лишь то, что нужно для чтения и написания базового async-кода.

### Порождение задач и структурированная конкурентность

`spawn` в Tokio создаёт новую асинхронную задачу: это похоже на `thread::spawn`, но намного легче:

```rust,ignore
use tokio::task;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Запускаем три конкурентные задачи
    let h1 = task::spawn(async {
        sleep(Duration::from_millis(200)).await;
        "профиль пользователя получен"
    });

    let h2 = task::spawn(async {
        sleep(Duration::from_millis(100)).await;
        "история заказов получена"
    });

    let h3 = task::spawn(async {
        sleep(Duration::from_millis(150)).await;
        "рекомендации получены"
    });

    // Ждём все три конкурентно (не последовательно!)
    let (r1, r2, r3) = tokio::join!(h1, h2, h3);
    println!("{}", r1.unwrap());
    println!("{}", r2.unwrap());
    println!("{}", r3.unwrap());
}
```

**`join!` против `try_join!` против `select!`**:

| Макрос | Поведение | Когда использовать |
|--------|-----------|--------------------|
| `join!` | Ждёт ВСЕ future | Все задачи должны завершиться |
| `try_join!` | Ждёт все, прерывается на первом `Err` | Задачи возвращают `Result` |
| `select!` | Возвращает управление, когда ПЕРВЫЙ future завершился | Таймауты, отмена |

```rust,ignore
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout() -> Result<String, Box<dyn std::error::Error>> {
    let result = timeout(Duration::from_secs(5), async {
        // Имитация медленного сетевого вызова
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok::<_, Box<dyn std::error::Error>>("data".to_string())
    }).await??; // Первый ? разворачивает Elapsed, второй ? разворачивает внутренний Result

    Ok(result)
}
```

### `Send` и почему future должны быть `Send`

Когда вы делаете `tokio::spawn` для future, он может продолжиться в другом потоке ОС. Поэтому future должен быть `Send`. Распространённые ошибки:

```rust,ignore
use std::rc::Rc;

async fn not_send() {
    let rc = Rc::new(42); // Rc является !Send
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    println!("{}", rc); // rc удерживается через .await: future является !Send
}

// Исправление 1: уничтожить до .await
async fn fixed_drop() {
    let data = {
        let rc = Rc::new(42);
        *rc // Копируем значение наружу
    }; // rc уничтожается здесь
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    println!("{}", data); // Просто i32, который является Send
}

// Исправление 2: использовать Arc вместо Rc
async fn fixed_arc() {
    let arc = std::sync::Arc::new(42); // Arc является Send
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    println!("{}", arc); // ✅ Future является Send
}
```

> **Полное описание async**: о `Stream`, `select!`, безопасности отмены, структурированной конкурентности и middleware `tower` см. в нашем отдельном руководстве **Async Rust Training**. Этот раздел охватывает лишь то, что нужно для чтения и написания базового async-кода.

> **См. также:** [гл. 5 — Каналы](ch05-channels-and-message-passing.md) о синхронных каналах. [гл. 6 — Конкурентность](ch06-concurrency-vs-parallelism-vs-threads.md) о потоках ОС и async-задачах.

> **Ключевые выводы: async**
> - `async fn` возвращает ленивый `Future`: ничего не выполняется, пока вы не сделаете `.await` или не запустите его
> - Используйте `tokio::task::spawn_blocking` для тяжёлых вычислений или блокирующей работы внутри async-контекста
> - Не держите `std::sync::MutexGuard` через `.await`: используйте `tokio::sync::Mutex`
> - Future должны быть `Send` при запуске: уничтожайте `!Send`-значения до точек `.await`

---

### Упражнение: конкурентный загрузчик с таймаутом ★★ (~25 минут)

Напишите асинхронную функцию `fetch_all`, которая запускает три задачи `tokio::spawn`, каждая из которых имитирует сетевой вызов через `tokio::time::sleep`. Объедините все три через `tokio::try_join!`, обёрнутый в `tokio::time::timeout(Duration::from_secs(5), ...)`. Верните `Result<Vec<String>, ...>` или ошибку, если какая-либо задача завершилась неудачно или истёк срок.

<details>
<summary>🔑 Решение</summary>

```rust,ignore
use tokio::time::{sleep, timeout, Duration};

async fn fake_fetch(name: &'static str, delay_ms: u64) -> Result<String, String> {
    sleep(Duration::from_millis(delay_ms)).await;
    Ok(format!("{name}: OK"))
}

async fn fetch_all() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let deadline = Duration::from_secs(5);

    let (a, b, c) = timeout(deadline, async {
        let h1 = tokio::spawn(fake_fetch("svc-a", 100));
        let h2 = tokio::spawn(fake_fetch("svc-b", 200));
        let h3 = tokio::spawn(fake_fetch("svc-c", 150));
        tokio::try_join!(h1, h2, h3)
    })
    .await??;

    Ok(vec![a?, b?, c?])
}

#[tokio::main]
async fn main() {
    let results = fetch_all().await.unwrap();
    for r in &results {
        println!("{r}");
    }
}
```

</details>

***
