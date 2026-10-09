## Асинхронное программирование: Task в C# против Future в Rust

> **Что вы узнаете:** ленивый `Future` Rust против энергичного `Task` в C#, модель исполнителя (tokio),
> отмену через `Drop` + `select!` против `CancellationToken`, а также практические паттерны для конкурентных запросов.
>
> **Сложность:** 🔴 Продвинутый

Разработчики C# хорошо знакомы с `async`/`await`. Rust использует те же ключевые слова, но модель выполнения принципиально иная.

### Модель исполнителя

```csharp
// C# — рантайм предоставляет встроенный пул потоков и планировщик задач
// async/await «просто работает» из коробки
public async Task<string> FetchDataAsync(string url)
{
    using var client = new HttpClient();
    return await client.GetStringAsync(url);  // Планируется пулом потоков .NET
}
// .NET управляет пулом потоков, планированием задач и контекстом синхронизации
```

```rust
// Rust — встроенного асинхронного рантайма нет. Исполнитель выбираете вы.
// Самый популярный — tokio.
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}

// Чтобы выполнить асинхронный код, нужен рантайм:
#[tokio::main]  // Этот макрос настраивает рантайм tokio
async fn main() {
    let data = fetch_data("https://example.com").await.unwrap();
    println!("{}", &data[..100]);
}
```

### Future против Task

| | C# `Task<T>` | Rust `Future<Output = T>` |
|---|---|---|
| **Выполнение** | Запускается сразу при создании | **Ленивый** — ничего не делает, пока его не `await`-ят |
| **Рантайм** | Встроенный (пул потоков CLR) | Внешний (tokio, async-std и т. д.) |
| **Отмена** | `CancellationToken` | Уничтожить `Future` (или `tokio::select!`) |
| **Конечный автомат** | Генерируется компилятором | Генерируется компилятором |
| **Размещение** | В куче | На стеке, пока не упаковано в `Box` |

```rust
// ВАЖНО: Future в Rust ленивые!
async fn compute() -> i32 { println!("Computing!"); 42 }

let future = compute();  // Ничего не выведено! Future ещё не опрошен.
let result = future.await; // ТЕПЕРЬ выводится "Computing!"
```

```csharp
// Task в C# запускаются сразу!
var task = ComputeAsync();  // "Computing!" выводится немедленно
var result = await task;    // Просто ждёт завершения
```

### Отмена: CancellationToken против Drop / select!

```csharp
// C# — кооперативная отмена через CancellationToken
public async Task ProcessAsync(CancellationToken ct)
{
    while (!ct.IsCancellationRequested)
    {
        await Task.Delay(1000, ct);  // Бросает исключение, если отменено
        DoWork();
    }
}

var cts = new CancellationTokenSource(TimeSpan.FromSeconds(5));
await ProcessAsync(cts.Token);
```

```rust
// Rust — отмена через уничтожение future или через tokio::select!
use tokio::time::{sleep, Duration};

async fn process() {
    loop {
        sleep(Duration::from_secs(1)).await;
        do_work();
    }
}

// Паттерн таймаута с select!
async fn run_with_timeout() {
    tokio::select! {
        _ = process() => { println!("Completed"); }
        _ = sleep(Duration::from_secs(5)) => { println!("Timed out!"); }
    }
    // Когда select! выбирает ветку таймаута, future process() УНИЧТОЖАЕТСЯ
    // — автоматическая очистка, CancellationToken не нужен
}
```

### Практический паттерн: конкурентные запросы с таймаутом

```csharp
// C# — конкурентные HTTP-запросы с таймаутом
public async Task<string[]> FetchAllAsync(string[] urls, CancellationToken ct)
{
    var tasks = urls.Select(url => httpClient.GetStringAsync(url, ct));
    return await Task.WhenAll(tasks);
}
```

```rust
// Rust — конкурентные запросы через tokio::join! или futures::join_all
use futures::future::join_all;

async fn fetch_all(urls: &[&str]) -> Vec<Result<String, reqwest::Error>> {
    let futures = urls.iter().map(|url| reqwest::get(*url));
    let responses = join_all(futures).await;

    let mut results = Vec::new();
    for resp in responses {
        results.push(resp?.text().await);
    }
    results
}

// С таймаутом:
async fn fetch_all_with_timeout(urls: &[&str]) -> Result<Vec<String>, &'static str> {
    tokio::time::timeout(
        Duration::from_secs(10),
        async {
            let futures: Vec<_> = urls.iter()
                .map(|url| async { reqwest::get(*url).await?.text().await })
                .collect();
            let results = join_all(futures).await;
            results.into_iter().collect::<Result<Vec<_>, _>>()
        }
    )
    .await
    .map_err(|_| "Request timed out")?
    .map_err(|_| "Request failed")
}
```

<details>
<summary><strong>🏋️ Упражнение: паттерн асинхронного таймаута</strong> (нажмите, чтобы раскрыть)</summary>

**Задача**: напишите асинхронную функцию, которая запрашивает два URL конкурентно, возвращает тот результат, который пришёл первым, а второй запрос отменяет. (В C# это `Task.WhenAny`.)

<details>
<summary>🔑 Решение</summary>

```rust
use tokio::time::{sleep, Duration};

// Имитация асинхронного запроса
async fn fetch(url: &str, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("Response from {url}")
}

async fn fetch_first(url1: &str, url2: &str) -> String {
    tokio::select! {
        result = fetch(url1, 200) => {
            println!("URL 1 won");
            result
        }
        result = fetch(url2, 500) => {
            println!("URL 2 won");
            result
        }
    }
    // Future проигравшей ветки автоматически уничтожается (отменяется)
}

#[tokio::main]
async fn main() {
    let result = fetch_first("https://fast.api", "https://slow.api").await;
    println!("{result}");
}
```

**Ключевая мысль**: `tokio::select!` — это аналог `Task.WhenAny` в Rust. Он гонит несколько future, завершается, когда первое из них готово, и уничтожает (отменяет) остальные.

</details>
</details>

### Запуск независимых задач через `tokio::spawn`

В C# `Task.Run` запускает работу, которая выполняется независимо от вызывающего кода. Аналог в Rust — `tokio::spawn`:

```rust
use tokio::task;

async fn background_work() {
    // Выполняется независимо — даже если future вызывающего кода уничтожен
    let handle = task::spawn(async {
        tokio::time::sleep(Duration::from_secs(2)).await;
        42
    });

    // Делаем другую работу, пока порождённая задача выполняется...
    println!("Doing other work");

    // Ждём результат, когда он понадобится
    let result = handle.await.unwrap(); // 42
}
```

```csharp
// Аналог в C#
var task = Task.Run(async () => {
    await Task.Delay(2000);
    return 42;
});
// Делаем другую работу...
var result = await task;
```

**Ключевое отличие**: обычный блок `async {}` ленивый — он ничего не делает, пока его не `await`-ят. `tokio::spawn` сразу запускает его в рантайме, как `Task.Run` в C#.

### Pin: понятие, которого в C# нет

Разработчики C# никогда не сталкиваются с `Pin` — сборщик мусора CLR свободно перемещает объекты и автоматически обновляет все ссылки. В Rust сборщика мусора нет. Когда компилятор превращает `async fn` в конечный автомат, эта структура может содержать внутренние указатели на собственные поля. Перемещение структуры сделало бы эти указатели недействительными.

`Pin<T>` — это обёртка, которая говорит: **«это значение не будет перемещено в памяти».**

```rust
// Pin встречается в таких контекстах:
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
    //           ^^^^^^^^^^^^^^ закреплено — внутренние ссылки остаются действительными
}

// Возврат future в упаковке Box из трейта:
fn make_future() -> Pin<Box<dyn Future<Output = i32> + Send>> {
    Box::pin(async { 42 })
}
```

**На практике вы почти никогда не пишете `Pin` сами.** Синтаксис `async fn` и `.await` справляется с этим. С `Pin` вы столкнётесь только в таких случаях:
- Сообщения об ошибках компилятора (следуйте подсказке)
- `tokio::select!` (используйте макрос `pin!()`)
- Методы трейтов, возвращающие `dyn Future` (используйте `Box::pin(async { ... })`)

> **Хотите глубже?** Сопутствующее руководство [Async Rust Training](../../async-book/src/ch04-pin-and-unpin.md) подробно рассматривает Pin, Unpin, самоссылающиеся структуры и структурное закрепление.

***


