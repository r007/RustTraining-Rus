## Справочник по инструментам для работы с итераторами

> **Что вы узнаете:** продвинутые комбинаторы итераторов помимо `filter`/`map`/`collect` — `enumerate`, `zip`, `chain`, `flat_map`, `scan`, `windows` и `chunks`. Они необходимы, чтобы заменить индексные циклы `for` в стиле C безопасными и выразительными итераторами Rust.

Базовой цепочки `filter`/`map`/`collect` хватает для многих случаев, но библиотека итераторов Rust
гораздо богаче. Этот раздел описывает инструменты, которыми вы будете пользоваться каждый день — особенно при
переводе циклов C, которые вручную отслеживают индексы, накапливают результаты или обрабатывают
данные фиксированными порциями.

### Краткая таблица

| Метод | Аналог в C | Что делает | Возвращает |
|-------|-------------|-------------|---------|
| `enumerate()` | `for (int i=0; ...)` | Сопоставляет каждый элемент с его индексом | `(usize, T)` |
| `zip(other)` | Параллельные массивы с одним индексом | Попарно объединяет элементы двух итераторов | `(A, B)` |
| `chain(other)` | Обработать массив1, затем массив2 | Конкатенирует два итератора | `T` |
| `flat_map(f)` | Вложенные циклы | Отображает и выравнивает на один уровень | `U` |
| `windows(n)` | `for (int i=0; i<len-n+1; i++) &arr[i..i+n]` | Перекрывающиеся срезы размера `n` | `&[T]` |
| `chunks(n)` | Обрабатывать по `n` элементов за раз | Непересекающиеся срезы размера `n` | `&[T]` |
| `fold(init, f)` | `int acc = init; for (...) acc = f(acc, x);` | Свести к одному значению | `Acc` |
| `scan(init, f)` | Накапливающий аккумулятор с выводом | Как `fold`, но выдаёт промежуточные результаты | `Option<B>` |
| `take(n)` / `skip(n)` | Начать цикл со смещения / ограничить | Первые `n` / пропустить первые `n` элементов | `T` |
| `take_while(f)` / `skip_while(f)` | `while (pred) {...}` | Брать/пропускать, пока предикат истинен | `T` |
| `peekable()` | Заглядывание вперёд через `arr[i+1]` | Позволяет `.peek()` без потребления элемента | `T` |
| `step_by(n)` | `for (i=0; i<len; i+=n)` | Брать каждый n-й элемент | `T` |
| `unzip()` | Разбиение параллельных массивов | Собрать пары в две коллекции | `(A, B)` |
| `sum()` / `product()` | Накопление суммы/произведения | Свести через `+` или `*` | `T` |
| `min()` / `max()` | Поиск экстремумов | Возвращают `Option<T>` | `Option<T>` |
| `any(f)` / `all(f)` | `bool found = false; for (...) ...` | Булев поиск с коротким замыканием | `bool` |
| `position(f)` | `for (i=0; ...) if (pred) return i;` | Индекс первого совпадения | `Option<usize>` |

### `enumerate` — индекс и значение (заменяет индексные циклы C)

```rust
fn main() {
    let sensors = ["GPU_TEMP", "CPU_TEMP", "FAN_RPM", "PSU_WATT"];

    // В стиле C: for (int i = 0; i < 4; i++) printf("[%d] %s\n", i, sensors[i]);
    for (i, name) in sensors.iter().enumerate() {
        println!("[{i}] {name}");
    }

    // Найти индекс конкретного датчика
    let gpu_idx = sensors.iter().position(|&s| s == "GPU_TEMP");
    println!("GPU sensor at index: {gpu_idx:?}");  // Some(0)
}
```

### `zip` — параллельный перебор (заменяет циклы по параллельным массивам)

```rust
fn main() {
    let names = ["accel_diag", "nic_diag", "cpu_diag"];
    let statuses = [true, false, true];
    let durations_ms = [1200, 850, 3400];

    // C: for (int i=0; i<3; i++) printf("%s: %s (%d ms)\n", names[i], ...);
    for ((name, passed), ms) in names.iter().zip(&statuses).zip(&durations_ms) {
        let status = if *passed { "PASS" } else { "FAIL" };
        println!("{name}: {status} ({ms} ms)");
    }
}
```

### `chain` — конкатенация итераторов

```rust
fn main() {
    let critical = vec!["ECC error", "Thermal shutdown"];
    let warnings = vec!["Link degraded", "Fan slow"];

    // Обрабатываем все события в порядке приоритета
    let all_events: Vec<_> = critical.iter().chain(warnings.iter()).collect();
    println!("{all_events:?}");
    // ["ECC error", "Thermal shutdown", "Link degraded", "Fan slow"]
}
```

### `flat_map` — выравнивание вложенных результатов

```rust
fn main() {
    let lines = vec!["gpu:42:ok", "nic:99:fail", "cpu:7:ok"];

    // Извлекаем все числовые значения из строк, разделённых двоеточиями
    let numbers: Vec<u32> = lines.iter()
        .flat_map(|line| line.split(':'))
        .filter_map(|token| token.parse::<u32>().ok())
        .collect();
    println!("{numbers:?}");  // [42, 99, 7]
}
```

### `windows` и `chunks` — скользящие и фиксированные группы

```rust
fn main() {
    let temps = [65, 68, 72, 71, 75, 80, 78, 76];

    // windows(3): перекрывающиеся группы по 3 (как скользящее среднее)
    // C: for (int i = 0; i <= len-3; i++) avg(arr[i], arr[i+1], arr[i+2]);
    let moving_avg: Vec<f64> = temps.windows(3)
        .map(|w| w.iter().sum::<i32>() as f64 / 3.0)
        .collect();
    println!("Moving avg: {moving_avg:.1?}");

    // chunks(2): непересекающиеся группы по 2
    // C: for (int i = 0; i < len; i += 2) process(arr[i], arr[i+1]);
    for pair in temps.chunks(2) {
        println!("Chunk: {pair:?}");
    }

    // chunks_exact(2): то же самое, но возникает panic при наличии остатка
    // Также: .remainder() возвращает оставшиеся элементы
}
```

### `fold` и `scan` — накопление

```rust
fn main() {
    let values = [10, 20, 30, 40, 50];

    // fold: единственный итоговый результат (как накопительный цикл в C)
    let sum = values.iter().fold(0, |acc, &x| acc + x);
    println!("Sum: {sum}");  // 150

    // Собираем строку через fold
    let csv = values.iter()
        .fold(String::new(), |acc, x| {
            if acc.is_empty() { format!("{x}") }
            else { format!("{acc},{x}") }
        });
    println!("CSV: {csv}");  // "10,20,30,40,50"

    // scan: как fold, но выдаёт промежуточные результаты
    let running_sum: Vec<i32> = values.iter()
        .scan(0, |state, &x| {
            *state += x;
            Some(*state)
        })
        .collect();
    println!("Running sum: {running_sum:?}");  // [10, 30, 60, 100, 150]
}
```

### Упражнение: конвейер данных датчиков

Даны сырые показания датчиков (по одному на строку, формат `"sensor_name:value:unit"`). Напишите
конвейер итераторов, который:
1. Разбирает каждую строку в `(name, f64, unit)`
2. Отбрасывает показания ниже порога
3. Группирует по имени датчика с помощью `fold` в `HashMap`
4. Выводит среднее показание для каждого датчика

```rust
// Стартовый код
fn main() {
    let raw_data = vec![
        "gpu_temp:72.5:C",
        "cpu_temp:65.0:C",
        "gpu_temp:74.2:C",
        "fan_rpm:1200.0:RPM",
        "cpu_temp:63.8:C",
        "gpu_temp:80.1:C",
        "fan_rpm:1150.0:RPM",
    ];
    let threshold = 70.0;
    // TODO: Разобрать, отфильтровать значения >= threshold, сгруппировать по имени, посчитать средние
}
```

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
use std::collections::HashMap;

fn main() {
    let raw_data = vec![
        "gpu_temp:72.5:C",
        "cpu_temp:65.0:C",
        "gpu_temp:74.2:C",
        "fan_rpm:1200.0:RPM",
        "cpu_temp:63.8:C",
        "gpu_temp:80.1:C",
        "fan_rpm:1150.0:RPM",
    ];
    let threshold = 70.0;

    // Разбор → фильтрация → группировка → среднее
    let grouped = raw_data.iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, ':').collect();
            if parts.len() == 3 {
                let value: f64 = parts[1].parse().ok()?;
                Some((parts[0], value, parts[2]))
            } else {
                None
            }
        })
        .filter(|(_, value, _)| *value >= threshold)
        .fold(HashMap::<&str, Vec<f64>>::new(), |mut acc, (name, value, _)| {
            acc.entry(name).or_default().push(value);
            acc
        });

    for (name, values) in &grouped {
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        println!("{name}: avg={avg:.1} ({} readings)", values.len());
    }
}
// Вывод (порядок может отличаться):
// gpu_temp: avg=75.6 (3 readings)
// fan_rpm: avg=1175.0 (2 readings)
```

</details>


# Итераторы в Rust: собственные типы
- Трейт ```Iterator``` используется для реализации итерации по пользовательским типам (https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)
    - В примере мы реализуем итератор для последовательности Фибоначчи, которая начинается с 1, 1, 2, ..., а каждый следующий элемент — это сумма двух предыдущих
    - Ассоциированный тип в ```Iterator``` (```type Item = u32;```) задаёт тип значений, которые выдаёт наш итератор (```u32```)
    - Метод ```next()``` содержит саму логику итератора. В данном случае всё состояние хранится в структуре ```Fibonacci```
    - Можно было бы реализовать ещё один трейт, ```IntoIterator```, с методом ```into_iter()``` для более специализированных итераторов
    - https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=ab367dc2611e1b5a0bf98f1185b38f3f

