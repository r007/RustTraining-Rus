# Конкурентность в Rust

> **Что вы узнаете:** модель конкурентности Rust — потоки, маркерные трейты `Send`/`Sync`, `Mutex<T>`, `Arc<T>`, каналы и то, как компилятор предотвращает гонки данных на этапе компиляции. Никаких накладных расходов на потокобезопасность, которой вы не пользуетесь.

- В Rust есть встроенная поддержка конкурентности, похожая на `std::thread` в C++
    - Ключевое отличие: Rust **предотвращает гонки данных на этапе компиляции** через маркерные трейты `Send` и `Sync`
    - В C++ передача `std::vector` между потоками без мьютекса — это неопределённое поведение, но код компилируется без проблем. В Rust такой код не скомпилируется.
    - `Mutex<T>` в Rust оборачивает **данные**, а не только доступ к ним: прочитать данные, не взяв блокировку, физически невозможно
- `thread::spawn()` создаёт отдельный поток, который параллельно выполняет замыкание `||`
```rust
use std::thread;
use std::time::Duration;
fn main() {
    let handle = thread::spawn(|| {
        for i in 0..10 {
            println!("Count in thread: {i}!");
            thread::sleep(Duration::from_millis(5));
        }
    });

    for i in 0..5 {
        println!("Main thread: {i}");
        thread::sleep(Duration::from_millis(5));
    }

    handle.join().unwrap(); // handle.join() гарантирует, что порождённый поток завершится
}
```

# Конкурентность в Rust: области видимости потоков
- ```thread::scope()``` можно использовать, когда нужно заимствовать данные из окружения. Это работает, потому что ```thread::scope``` ждёт завершения внутренних потоков
- Попробуйте выполнить это упражнение без ```thread::scope```, чтобы увидеть проблему
```rust
use std::thread;
fn main() {
  let a = [0, 1, 2];
  thread::scope(|scope| {
      scope.spawn(|| {
          for x in &a {
            println!("{x}");
          }
      });
  });
}
```
----
# Конкурентность в Rust: перемещение владения в поток
- Можно также использовать ```move```, чтобы передать владение потоку. Для типов `Copy`, таких как `[i32; 3]`, ключевое слово `move` копирует данные в замыкание, и исходная переменная остаётся пригодной для использования
```rust
use std::thread;
fn main() {
  let mut a = [0, 1, 2];
  let handle = thread::spawn(move || {
      for x in a {
        println!("{x}");
      }
  });
  a[0] = 42;    // Не влияет на копию, переданную в поток
  handle.join().unwrap();
}
```

# Конкурентность в Rust: Arc
- ```Arc<T>``` можно использовать, чтобы разделить *доступные только для чтения* ссылки между несколькими потоками
    - ```Arc``` расшифровывается как Atomic Reference Counted (атомарный подсчёт ссылок). Ссылка не освобождается, пока счётчик не достигнет 0
    - ```Arc::clone()``` просто увеличивает счётчик ссылок, не копируя данные
```rust
use std::sync::Arc;
use std::thread;
fn main() {
    let a = Arc::new([0, 1, 2]);
    let mut handles = Vec::new();
    for i in 0..2 {
        let arc = Arc::clone(&a);
        handles.push(thread::spawn(move || {
            println!("Thread: {i} {arc:?}");
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
}
```

# Конкурентность в Rust: Mutex
- ```Arc<T>``` можно комбинировать с ```Mutex<T>```, чтобы получить изменяемые ссылки
    - ```Mutex``` охраняет защищаемые данные и гарантирует, что доступ имеет только поток, который держит блокировку.
    - `MutexGuard` автоматически освобождается, когда выходит из области видимости (RAII). Примечание: `std::mem::forget` всё же может «утечь» защитник — поэтому точнее сказать «невозможно забыть разблокировать», а не «невозможно допустить утечку».
```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
            // MutexGuard уничтожается здесь — блокировка снимается автоматически
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
    // Вывод: Final count: 5
}
```

# Конкурентность в Rust: RwLock
- `RwLock<T>` допускает **несколько одновременных читателей** или **одного эксклюзивного писателя** — шаблон блокировки для чтения/записи из C++ (`std::shared_mutex`)
    - Используйте `RwLock`, когда чтений намного больше, чем записей (например, конфигурация, кэши)
    - Используйте `Mutex`, когда частота чтения и записи примерно одинакова или критические секции короткие
```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let config = Arc::new(RwLock::new(String::from("v1.0")));
    let mut handles = Vec::new();

    // Запускаем 5 читателей — все могут работать одновременно
    for i in 0..5 {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            let val = config.read().unwrap();  // Несколько читателей — это нормально
            println!("Reader {i}: {val}");
        }));
    }

    // Один писатель — блокируется, пока все читатели не закончат
    {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            let mut val = config.write().unwrap();  // Эксклюзивный доступ
            *val = String::from("v2.0");
            println!("Writer: updated to {val}");
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

# Конкурентность в Rust: отравление Mutex
- Если поток **падает** (panic), удерживая `Mutex` или `RwLock`, блокировка становится **отравленной** (poisoned)
    - Последующие вызовы `.lock()` возвращают `Err(PoisonError)` — данные могут находиться в несогласованном состоянии
    - Можно восстановиться с помощью `.into_inner()`, если вы уверены, что данные по-прежнему корректны
    - В C++ аналога нет — `std::mutex` не знает понятия отравления; упавший поток просто оставляет блокировку захваченной
```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));

    let data2 = Arc::clone(&data);
    let handle = thread::spawn(move || {
        let mut guard = data2.lock().unwrap();
        guard.push(4);
        panic!("oops!");  // Блокировка теперь отравлена
    });

    let _ = handle.join();  // Поток упал с panic

    // Последующие попытки захватить блокировку возвращают Err(PoisonError)
    match data.lock() {
        Ok(guard) => println!("Data: {guard:?}"),
        Err(poisoned) => {
            println!("Lock was poisoned! Recovering...");
            let guard = poisoned.into_inner();  // Всё равно получаем доступ к данным
            println!("Recovered data: {guard:?}");  // [1, 2, 3, 4] — push выполнился до panic
        }
    }
}
```

# Конкурентность в Rust: атомарные типы
- Для простых счётчиков и флагов типы `std::sync::atomic` позволяют избежать накладных расходов `Mutex`
    - `AtomicBool`, `AtomicI32`, `AtomicU64`, `AtomicUsize` и т. д.
    - Аналог `std::atomic<T>` из C++ — та же модель порядка доступа к памяти (`Relaxed`, `Acquire`, `Release`, `SeqCst`)
```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::new();

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Counter: {}", counter.load(Ordering::SeqCst));
    // Вывод: Counter: 10000
}
```

| Примитив | Когда использовать | Аналог в C++ |
|-----------|-------------|----------------|
| `Mutex<T>` | Общее изменяемое состояние общего назначения | `std::mutex` + ручная привязка данных |
| `RwLock<T>` | Нагрузка с преобладанием чтения | `std::shared_mutex` |
| `Atomic*` | Простые счётчики, флаги, lock-free-шаблоны | `std::atomic<T>` |
| `Condvar` | Ожидание, пока условие не станет истинным | `std::condition_variable` |

# Конкурентность в Rust: Condvar
- `Condvar` (условная переменная) позволяет потоку **заснуть до тех пор, пока другой поток не сигнализирует**, что условие изменилось
    - Всегда используется вместе с `Mutex` — шаблон такой: захватить блокировку, проверить условие, ждать, если оно не выполнено, действовать, когда оно выполнено
    - Эквивалент `std::condition_variable` / `std::condition_variable::wait` в C++
    - Учитывает **ложные пробуждения** (spurious wakeups) — всегда перепроверяйте условие в цикле (или используйте `wait_while`/`wait_until`)
```rust
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

fn main() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));

    // Порождаем рабочий поток, который ждёт сигнала
    let pair2 = Arc::clone(&pair);
    let worker = thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        let mut ready = lock.lock().unwrap();
        // wait: засыпает до сигнала (всегда перепроверяйте в цикле из-за ложных пробуждений)
        while !*ready {
            ready = cvar.wait(ready).unwrap();
        }
        println!("Worker: condition met, proceeding!");
    });

    // Главный поток что-то делает, затем сигнализирует рабочему потоку
    thread::sleep(std::time::Duration::from_millis(100));
    {
        let (lock, cvar) = &*pair;
        let mut ready = lock.lock().unwrap();
        *ready = true;
        cvar.notify_one();  // Будим один ожидающий поток (notify_all() будит все)
    }

    worker.join().unwrap();
}
```

> **Когда использовать Condvar, а когда каналы:** используйте `Condvar`, когда потоки разделяют изменяемое состояние и должны ждать условия на этом состоянии (например, «буфер не пуст»). Используйте каналы (`mpsc`), когда потокам нужно передавать *сообщения*. Каналы обычно проще для рассуждений.

# Конкурентность в Rust: каналы
- Каналы Rust позволяют обмениваться сообщениями между ```Sender``` и ```Receiver```
    - Используется парадигма под названием ```mpsc``` или ```Multi-producer, Single-Consumer``` (много производителей, один потребитель)
    - И ```send()```, и ```recv()``` могут блокировать поток
```rust
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    
    tx.send(10).unwrap();
    tx.send(20).unwrap();
    
    println!("Received: {:?}", rx.recv());
    println!("Received: {:?}", rx.recv());

    let tx2 = tx.clone();
    tx2.send(30).unwrap();
    println!("Received: {:?}", rx.recv());
}
```

# Конкурентность в Rust: каналы и потоки
- Каналы можно комбинировать с потоками
```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();
    for _ in 0..2 {
        let tx2 = tx.clone();
        thread::spawn(move || {
            let thread_id = thread::current().id();
            for i in 0..10 {
                tx2.send(format!("Message {i}")).unwrap();
                println!("{thread_id:?}: sent Message {i}");
            }
            println!("{thread_id:?}: done");
        });
    }

        // Удаляем исходного отправителя, чтобы rx.iter() завершился, когда все клонированные отправители будут уничтожены
    drop(tx);

    thread::sleep(Duration::from_millis(100));

    for msg in rx.iter() {
        println!("Main: got {msg}");
    }
}
```



## Почему Rust предотвращает гонки данных: Send и Sync

- Rust использует два маркерных трейта, чтобы обеспечивать потокобезопасность на этапе компиляции:
    - `Send`: тип является `Send`, если его можно безопасно **передать** в другой поток
    - `Sync`: тип является `Sync`, если его можно безопасно **разделять** (через `&T`) между потоками
- Большинство типов автоматически являются `Send + Sync`. Заметные исключения:
    - `Rc<T>` **не** является ни Send, ни Sync (для потоков используйте `Arc<T>`)
    - `Cell<T>` и `RefCell<T>` **не** являются Sync (используйте `Mutex<T>` или `RwLock<T>`)
    - Сырые указатели (`*const T`, `*mut T`) **не** являются ни Send, ни Sync
- Именно поэтому компилятор не даёт использовать `Rc<T>` между потоками — он просто не реализует `Send`
- `Arc<Mutex<T>>` — потокобезопасный аналог `Rc<RefCell<T>>`

> **Интуиция** *(Jon Gjengset)*: представьте значения как игрушки.
> **`Send`** = вы можете **отдать свою игрушку** другому ребёнку (потоку) — передача владения безопасна.
> **`Sync`** = вы можете **позволить другим играть с вашей игрушкой одновременно** — совместное использование ссылки безопасно.
> У `Rc<T>` хрупкий (неатомарный) счётчик ссылок; передача или совместное использование повредили бы счётчик, поэтому он не является ни `Send`, ни `Sync`.


# Упражнение: подсчёт слов в многопоточном режиме

🔴 **Сложный уровень** — объединяет потоки, Arc, Mutex и HashMap

- Дан `Vec<String>` со строками текста. Запустите по одному потоку на каждую строку, чтобы подсчитать слова в ней
- Используйте `Arc<Mutex<HashMap<String, usize>>>`, чтобы собрать результаты
- Выведите общее количество слов по всем строкам
- **Бонус**: попробуйте реализовать то же самое с каналами (`mpsc`) вместо разделяемого состояния

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let lines = vec![
        "the quick brown fox".to_string(),
        "jumps over the lazy dog".to_string(),
        "the fox is quick".to_string(),
    ];

    let word_counts: Arc<Mutex<HashMap<String, usize>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut handles = vec![];
    for line in &lines {
        let line = line.clone();
        let counts = Arc::clone(&word_counts);
        handles.push(thread::spawn(move || {
            for word in line.split_whitespace() {
                let mut map = counts.lock().unwrap();
                *map.entry(word.to_lowercase()).or_insert(0) += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let counts = word_counts.lock().unwrap();
    let total: usize = counts.values().sum();
    println!("Word frequencies: {counts:#?}");
    println!("Total words: {total}");
}
// Вывод (порядок может отличаться):
// Word frequencies: {
//     "the": 3,
//     "quick": 2,
//     "brown": 1,
//     "fox": 2,
//     "jumps": 1,
//     "over": 1,
//     "lazy": 1,
//     "dog": 1,
//     "is": 1,
// }
// Total words: 13
```

</details>

