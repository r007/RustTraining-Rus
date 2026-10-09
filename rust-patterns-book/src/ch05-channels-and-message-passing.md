# 5. Каналы и передача сообщений 🟢

> **Что вы узнаете:**
> - Основы `std::sync::mpsc` и когда стоит перейти на crossbeam-channel
> - Выбор канала через `select!` для обработки сообщений из нескольких источников
> - Ограниченные и неограниченные каналы и стратегии обратного давления (backpressure)
> - Паттерн актора для инкапсуляции конкурентного состояния

## std::sync::mpsc: стандартный канал

Стандартная библиотека Rust предоставляет канал с несколькими отправителями и одним получателем (multi-producer, single-consumer):

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Создаём канал: tx (отправитель) и rx (получатель)
    let (tx, rx) = mpsc::channel();

    // Запускаем поток-производитель
    let tx1 = tx.clone(); // Клонируем для нескольких производителей
    thread::spawn(move || {
        for i in 0..5 {
            tx1.send(format!("производитель-1: сообщение {i}")).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Второй производитель
    thread::spawn(move || {
        for i in 0..5 {
            tx.send(format!("производитель-2: сообщение {i}")).unwrap();
            thread::sleep(Duration::from_millis(150));
        }
    });

    // Потребитель: получает все сообщения
    for msg in rx {
        // Итератор rx завершается, когда уничтожены ВСЕ отправители
        println!("Получено: {msg}");
    }
    println!("Все производители завершили работу.");
}
```

**Ключевые свойства**:
- **Неограниченный** по умолчанию: может заполнить память, если потребитель медленный
- `mpsc::sync_channel(N)` создаёт **ограниченный** канал с обратным давлением
- `rx.recv()` блокирует текущий поток, пока не придёт сообщение
- `rx.try_recv()` сразу возвращает `Err(TryRecvError::Empty)`, если ничего не готово
- Канал закрывается, когда уничтожены все `Sender`

```rust
// Ограниченный канал с обратным давлением:
let (tx, rx) = mpsc::sync_channel(10); // Буфер на 10 сообщений

thread::spawn(move || {
    for i in 0..1000 {
        tx.send(i).unwrap(); // БЛОКИРУЕТСЯ, если буфер полон: естественное обратное давление
    }
});
```

### crossbeam-channel: рабочая лошадка для продакшена

`crossbeam-channel` — де-факто стандарт для каналов в продакшене. Он быстрее `std::sync::mpsc` и поддерживает нескольких получателей (`mpmc`):

```rust,ignore
// Cargo.toml:
//   [dependencies]
//   crossbeam-channel = "0.5"
use crossbeam_channel::{bounded, unbounded, select, Sender, Receiver};
use std::thread;
use std::time::Duration;

fn main() {
    // Ограниченный канал MPMC
    let (tx, rx) = bounded::<String>(100);

    // Несколько производителей
    for id in 0..4 {
        let tx = tx.clone();
        thread::spawn(move || {
            for i in 0..10 {
                tx.send(format!("рабочий-{id}: элемент-{i}")).unwrap();
            }
        });
    }
    drop(tx); // Отбрасываем исходный отправитель, чтобы канал мог закрыться

    // Несколько потребителей (с std::sync::mpsc так нельзя!)
    let rx2 = rx.clone();
    let consumer1 = thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            println!("[потребитель-1] {msg}");
        }
    });
    let consumer2 = thread::spawn(move || {
        while let Ok(msg) = rx2.recv() {
            println!("[потребитель-2] {msg}");
        }
    });

    consumer1.join().unwrap();
    consumer2.join().unwrap();
}
```

### Выбор канала (select!)

Слушайте несколько каналов одновременно, как `select` в Go:

```rust,ignore
use crossbeam_channel::{bounded, tick, after, select};
use std::time::Duration;

fn main() {
    let (work_tx, work_rx) = bounded::<String>(10);
    let ticker = tick(Duration::from_secs(1));        // Периодический тик
    let deadline = after(Duration::from_secs(10));     // Одноразовый таймаут

    // Производитель
    let tx = work_tx.clone();
    std::thread::spawn(move || {
        for i in 0..100 {
            tx.send(format!("задача-{i}")).unwrap();
            std::thread::sleep(Duration::from_millis(500));
        }
    });
    drop(work_tx);

    loop {
        select! {
            recv(work_rx) -> msg => {
                match msg {
                    Ok(job) => println!("Обработка: {job}"),
                    Err(_) => {
                        println!("Канал задач закрыт");
                        break;
                    }
                }
            },
            recv(ticker) -> _ => {
                println!("Тик — сигнал жизни");
            },
            recv(deadline) -> _ => {
                println!("Достигнут дедлайн — завершение работы");
                break;
            },
        }
    }
}
```

> **Сравнение с Go**: это в точности похоже на оператор `select` для каналов в Go. Макрос `select!` из crossbeam, как и в Go, случайным образом выбирает порядок веток, чтобы избежать голодания.

### Ограниченные и неограниченные каналы и обратное давление

| Тип | Поведение при заполнении | Память | Сценарий использования |
|-----|--------------------------|--------|------------------------|
| **Неограниченный** | Никогда не блокирует (растёт в куче) | Не ограничена ⚠️ | Редко: только когда производитель медленнее потребителя |
| **Ограниченный** | `send()` блокируется, пока не появится место | Фиксирована | Выбор по умолчанию в продакшене: защита от OOM |
| **Рандеву** (bounded(0)) | `send()` блокируется, пока получатель не готов | Нет | Синхронизация и передача управления |

```rust
// Канал рандеву: нулевая ёмкость, прямая передача
let (tx, rx) = crossbeam_channel::bounded(0);
// tx.send(x) блокируется, пока не вызван rx.recv(), и наоборот.
// Это точно синхронизирует два потока.
```

**Правило**: в продакшене всегда используйте ограниченные каналы, если только вы не можете доказать, что производитель никогда не обгонит потребителя.

### Паттерн актора на каналах

Паттерн актора использует каналы, чтобы сериализовать доступ к изменяемому состоянию: никакие мьютексы не нужны.

```rust
use std::sync::mpsc;
use std::thread;

// Сообщения, которые может получить актор
enum CounterMsg {
    Increment,
    Decrement,
    Get(mpsc::Sender<i64>), // Канал для ответа
}

struct CounterActor {
    count: i64,
    rx: mpsc::Receiver<CounterMsg>,
}

impl CounterActor {
    fn new(rx: mpsc::Receiver<CounterMsg>) -> Self {
        CounterActor { count: 0, rx }
    }

    fn run(mut self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                CounterMsg::Increment => self.count += 1,
                CounterMsg::Decrement => self.count -= 1,
                CounterMsg::Get(reply) => {
                    let _ = reply.send(self.count);
                }
            }
        }
    }
}

// Дескриптор актора: дёшево клонируется, реализует Send + Sync
#[derive(Clone)]
struct Counter {
    tx: mpsc::Sender<CounterMsg>,
}

impl Counter {
    fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || CounterActor::new(rx).run());
        Counter { tx }
    }

    fn increment(&self) { let _ = self.tx.send(CounterMsg::Increment); }
    fn decrement(&self) { let _ = self.tx.send(CounterMsg::Decrement); }

    fn get(&self) -> i64 {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.tx.send(CounterMsg::Get(reply_tx)).unwrap();
        reply_rx.recv().unwrap()
    }
}

fn main() {
    let counter = Counter::spawn();

    // Несколько потоков могут безопасно использовать счётчик: никакого мьютекса!
    let handles: Vec<_> = (0..10).map(|_| {
        let counter = counter.clone();
        thread::spawn(move || {
            for _ in 0..1000 {
                counter.increment();
            }
        })
    }).collect();

    for h in handles { h.join().unwrap(); }
    println!("Итоговое значение: {}", counter.get()); // 10000
}
```

> **Когда использовать акторы, а когда мьютексы**: акторы хороши, когда у состояния сложные инварианты, операции занимают много времени или вы хотите сериализовать доступ, не думая о порядке захвата блокировок. Для коротких критических секций мьютексы проще.

> **Ключевые выводы: каналы**
> - `crossbeam-channel` — рабочая лошадка для продакшена: быстрее и богаче по возможностям, чем `std::sync::mpsc`
> - `select!` заменяет сложный опрос нескольких источников декларативным выбором канала
> - Ограниченные каналы дают естественное обратное давление; неограниченные каналы рискуют привести к OOM

> **См. также:** [гл. 6 — Конкурентность](ch06-concurrency-vs-parallelism-vs-threads.md) о потоках, Mutex и разделяемом состоянии. [гл. 16 — Async](ch16-asyncawait-essentials.md) о каналах в async (`tokio::sync::mpsc`).

---

### Упражнение: пул воркеров на каналах ★★★ (~45 минут)

Постройте пул воркеров на каналах, где:
- диспетчер отправляет структуры `Job` через канал
- N воркеров забирают задачи и отправляют результаты обратно
- используйте `std::sync::mpsc` с `Arc<Mutex<Receiver>>` для распределения работы между воркерами (work-stealing)

<details>
<summary>🔑 Решение</summary>

```rust
use std::sync::mpsc;
use std::thread;

struct Job {
    id: u64,
    data: String,
}

struct JobResult {
    job_id: u64,
    output: String,
    worker_id: usize,
}

fn worker_pool(jobs: Vec<Job>, num_workers: usize) -> Vec<JobResult> {
    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let (result_tx, result_rx) = mpsc::channel::<JobResult>();

    let job_rx = std::sync::Arc::new(std::sync::Mutex::new(job_rx));

    let mut handles = Vec::new();
    for worker_id in 0..num_workers {
        let job_rx = job_rx.clone();
        let result_tx = result_tx.clone();
        handles.push(thread::spawn(move || {
            loop {
                let job = {
                    let rx = job_rx.lock().unwrap();
                    rx.recv()
                };
                match job {
                    Ok(job) => {
                        let output = format!("обработано '{}' воркером {worker_id}", job.data);
                        result_tx.send(JobResult {
                            job_id: job.id, output, worker_id,
                        }).unwrap();
                    }
                    Err(_) => break,
                }
            }
        }));
    }
    drop(result_tx);

    let num_jobs = jobs.len();
    for job in jobs {
        job_tx.send(job).unwrap();
    }
    drop(job_tx);

    let results: Vec<_> = result_rx.into_iter().collect();
    assert_eq!(results.len(), num_jobs);

    for h in handles { h.join().unwrap(); }
    results
}

fn main() {
    let jobs: Vec<Job> = (0..20).map(|i| Job {
        id: i, data: format!("задача-{i}"),
    }).collect();

    let results = worker_pool(jobs, 4);
    for r in &results {
        println!("[воркер {}] задача {}: {}", r.worker_id, r.job_id, r.output);
    }
}
```

</details>

***
