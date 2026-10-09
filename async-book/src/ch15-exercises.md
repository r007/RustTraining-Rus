## Упражнения

### Упражнение 1: асинхронный эхо-сервер

Напишите TCP-эхо-сервер, который обрабатывает нескольких клиентов одновременно.

**Требования**:
- Слушать `127.0.0.1:8080`
- Принимать соединения и возвращать обратно каждую строку
- Корректно обрабатывать отключения клиентов
- Выводить лог при подключении и отключении клиентов

<details>
<summary>🔑 Решение</summary>

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Эхо-сервер слушает :8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("[{addr}] Подключён");

        tokio::spawn(async move {
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        println!("[{addr}] Отключён");
                        break;
                    }
                    Ok(_) => {
                        print!("[{addr}] Эхо: {line}");
                        if writer.write_all(line.as_bytes()).await.is_err() {
                            println!("[{addr}] Ошибка записи, отключаем");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("[{addr}] Ошибка чтения: {e}");
                        break;
                    }
                }
            }
        });
    }
}
```

</details>

---

### Упражнение 2: конкурентный загрузчик URL с ограничением частоты

Загрузите список URL одновременно, но не более 5 запросов одновременно.

<details>
<summary>🔑 Решение</summary>

```rust
use futures::stream::{self, StreamExt};
use tokio::time::{sleep, Duration};

async fn fetch_urls(urls: Vec<String>) -> Vec<Result<String, String>> {
    // buffer_unordered(5) гарантирует, что опрашивается не более 5 future
    // одновременно — отдельный Semaphore здесь не нужен.
    let results: Vec<_> = stream::iter(urls)
        .map(|url| {
            async move {
                println!("Загружаем: {url}");

                match reqwest::get(&url).await {
                    Ok(resp) => match resp.text().await {
                        Ok(body) => Ok(body),
                        Err(e) => Err(format!("{url}: {e}")),
                    },
                    Err(e) => Err(format!("{url}: {e}")),
                }
            }
        })
        .buffer_unordered(5) // ← Этого достаточно, чтобы ограничить конкурентность 5
        .collect()
        .await;

    results
}

// ПРИМЕЧАНИЕ: используйте Semaphore, когда нужно ограничить конкурентность
// для независимо порождённых задач (tokio::spawn). Используйте buffer_unordered,
// когда обрабатываете поток. Не комбинируйте оба механизма для одного и того же лимита.
```

</details>

---

### Упражнение 3: graceful shutdown с пулом воркеров

Постройте обработчик задач с:
- очередью работы на каналах
- N задачами-воркерами, которые читают из очереди
- graceful shutdown по Ctrl+C: прекратить приём, доделать текущую работу

<details>
<summary>🔑 Решение</summary>

```rust
use tokio::sync::{mpsc, watch};
use tokio::time::{sleep, Duration};

struct WorkItem {
    id: u64,
    payload: String,
}

#[tokio::main]
async fn main() {
    let (work_tx, work_rx) = mpsc::channel::<WorkItem>(100);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Запускаем 4 воркера
    let mut worker_handles = Vec::new();
    let work_rx = std::sync::Arc::new(tokio::sync::Mutex::new(work_rx));

    for id in 0..4 {
        let rx = work_rx.clone();
        let mut shutdown = shutdown_rx.clone();
        let handle = tokio::spawn(async move {
            loop {
                let item = {
                    let mut rx = rx.lock().await;
                    tokio::select! {
                        item = rx.recv() => item,
                        _ = shutdown.changed() => {
                            if *shutdown.borrow() { None } else { continue }
                        }
                    }
                };

                match item {
                    Some(work) => {
                        println!("Воркер {id}: обрабатываю элемент {}", work.id);
                        sleep(Duration::from_millis(200)).await; // Имитация работы
                        println!("Воркер {id}: закончил элемент {}", work.id);
                    }
                    None => {
                        println!("Воркер {id}: канал закрыт, выходим");
                        break;
                    }
                }
            }
        });
        worker_handles.push(handle);
    }

    // Производитель: отправляет работу
    let producer = tokio::spawn(async move {
        for i in 0..20 {
            let _ = work_tx.send(WorkItem {
                id: i,
                payload: format!("task-{i}"),
            }).await;
            sleep(Duration::from_millis(50)).await;
        }
    });

    // Ждём Ctrl+C
    tokio::signal::ctrl_c().await.unwrap();
    println!("\nПолучен сигнал остановки!");
    shutdown_tx.send(true).unwrap();
    producer.abort(); // Отменяем производителя

    // Ждём, пока воркеры завершатся
    for handle in worker_handles {
        let _ = handle.await;
    }
    println!("Все воркеры остановлены. До свидания!");
}
```

</details>

---

### Упражнение 4: простой асинхронный мьютекс с нуля

Реализуйте асинхронный мьютекс на основе каналов (без `tokio::sync::Mutex`).

*Подсказка*: используйте `tokio::sync::Semaphore` с одним разрешением, чтобы сериализовать доступ.

<details>
<summary>🔑 Решение</summary>

```rust
use std::cell::UnsafeCell;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub struct SimpleAsyncMutex<T> {
    data: Arc<UnsafeCell<T>>,
    semaphore: Arc<Semaphore>,
}

// SAFETY: доступ к T сериализуется семафором (не более одного разрешения).
unsafe impl<T: Send> Send for SimpleAsyncMutex<T> {}
unsafe impl<T: Send> Sync for SimpleAsyncMutex<T> {}

pub struct SimpleGuard<T> {
    data: Arc<UnsafeCell<T>>,
    _permit: OwnedSemaphorePermit, // Уничтожается вместе с guard → освобождает блокировку
}

impl<T> SimpleAsyncMutex<T> {
    pub fn new(value: T) -> Self {
        SimpleAsyncMutex {
            data: Arc::new(UnsafeCell::new(value)),
            semaphore: Arc::new(Semaphore::new(1)),
        }
    }

    pub async fn lock(&self) -> SimpleGuard<T> {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        SimpleGuard {
            data: self.data.clone(),
            _permit: permit,
        }
    }
}

impl<T> std::ops::Deref for SimpleGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: у нас единственное разрешение семафора, поэтому других
        // SimpleGuard не существует → эксклюзивный доступ гарантирован.
        unsafe { &*self.data.get() }
    }
}

impl<T> std::ops::DerefMut for SimpleGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: то же рассуждение — единственное разрешение гарантирует эксклюзивность.
        unsafe { &mut *self.data.get() }
    }
}

// Когда SimpleGuard уничтожается, уничтожается и _permit,
// что освобождает разрешение семафора — следующий lock() может продолжить.

// Использование:
// let mutex = SimpleAsyncMutex::new(vec![1, 2, 3]);
// {
//     let mut guard = mutex.lock().await;
//     guard.push(4);
// } // разрешение освобождено здесь
```

**Ключевой вывод**: асинхронные мьютексы обычно строятся поверх семафоров. Семафор обеспечивает асинхронное ожидание: когда блокировка занята, `acquire()` приостанавливает задачу до освобождения разрешения. Именно так `tokio::sync::Mutex` работает внутри.

> **Почему `UnsafeCell`, а не `std::sync::Mutex`?** В предыдущей версии этого
> упражнения использовался `Arc<Mutex<T>>` с `Deref`/`DerefMut`, которые вызывали `.lock().unwrap()`.
> Это не компилируется — возвращаемая `&T` заимствует данные у временного `MutexGuard`,
> который уничтожается сразу. `UnsafeCell` обходит промежуточный guard, а сериализация
> через семафор делает `unsafe` корректным.

</details>

---

### Упражнение 5: конвейер потоков

Постройте конвейер обработки данных на потоках:
1. Сгенерируйте числа 1..=100
2. Отфильтруйте чётные
3. Возведите каждое в квадрат
4. Обрабатывайте по 10 штук одновременно (имитация через sleep)
5. Соберите результаты

<details>
<summary>🔑 Решение</summary>

```rust
use futures::stream::{self, StreamExt};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let results: Vec<u64> = stream::iter(1u64..=100)
        // Шаг 2: отфильтровать чётные
        .filter(|x| futures::future::ready(x % 2 == 0))
        // Шаг 3: возвести каждое в квадрат
        .map(|x| x * x)
        // Шаг 4: обрабатывать одновременно (имитация асинхронной работы)
        .map(|x| async move {
            sleep(Duration::from_millis(50)).await;
            println!("Обработано: {x}");
            x
        })
        .buffer_unordered(10) // 10 одновременно
        // Шаг 5: собрать
        .collect()
        .await;

    println!("Получено результатов: {}", results.len());
    println!("Сумма: {}", results.iter().sum::<u64>());
}
```

</details>

---

### Упражнение 6: реализуйте select с таймаутом

Не используя `tokio::select!` и `tokio::time::timeout`, реализуйте функцию, которая состязает future с дедлайном и возвращает `Either::Left(result)` или `Either::Right(())` при таймауте.

*Подсказка*: опирайтесь на комбинатор `Select` из главы 6 и `TimerFuture` из той же главы.

<details>
<summary>🔑 Решение</summary>

```rust,ignore
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

pub enum Either<A, B> {
    Left(A),
    Right(B),
}

pub struct Timeout<F> {
    future: F,
    timer: TimerFuture, // Из главы 6
}

impl<F: Future + Unpin> Timeout<F> {
    pub fn new(future: F, duration: Duration) -> Self {
        Timeout {
            future,
            timer: TimerFuture::new(duration),
        }
    }
}

impl<F: Future + Unpin> Future for Timeout<F> {
    type Output = Either<F::Output, ()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Проверяем, завершился ли основной future
        if let Poll::Ready(val) = Pin::new(&mut self.future).poll(cx) {
            return Poll::Ready(Either::Left(val));
        }

        // Проверяем, не истёк ли таймер
        if let Poll::Ready(()) = Pin::new(&mut self.timer).poll(cx) {
            return Poll::Ready(Either::Right(()));
        }

        Poll::Pending
    }
}

// Использование:
// match Timeout::new(fetch_data(), Duration::from_secs(5)).await {
//     Either::Left(data) => println!("Получены данные: {data}"),
//     Either::Right(()) => println!("Время вышло!"),
// }
```

**Ключевой вывод**: `select`/`timeout` — это просто опрос двух future и выяснение, какой завершится первым. Вся асинхронная экосистема построена на этом простом примитиве: poll, Pending/Ready, Waker.

</details>

***
