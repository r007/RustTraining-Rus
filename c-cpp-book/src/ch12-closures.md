## Замыкания в Rust

> **Что вы узнаете:** замыкания как анонимные функции, три трейта захвата (`Fn`, `FnMut`, `FnOnce`), замыкания `move` и то, как замыкания Rust соотносятся с лямбдами C++ — с автоматическим анализом захвата вместо ручных спецификаций `[&]`/`[=]`.

- Замыкания — это анонимные функции, которые могут захватывать своё окружение
    - Аналог в C++: лямбды (`[&](int x) { return x + 1; }`)
    - Ключевое отличие: у замыканий Rust есть **три** трейта захвата (`Fn`, `FnMut`, `FnOnce`), которые компилятор выбирает автоматически
    - Режимы захвата в C++ (`[=]`, `[&]`, `[this]`) задаются вручную и подвержены ошибкам (висячий `[&]`!)
    - Проверка заимствований Rust предотвращает висячие захваты на этапе компиляции
- Замыкания обозначаются символом `||`. Параметры размещаются внутри `||` и могут использовать вывод типов
- Замыкания часто используются вместе с итераторами (следующая тема)
```rust
fn add_one(x: u32) -> u32 {
    x + 1
}
fn main() {
    let add_one_v1 = |x : u32| {x + 1}; // Тип указан явно
    let add_one_v2 = |x| {x + 1};   // Тип выводится из места вызова
    let add_one_v3 = |x| x+1;   // Допустимо для однострочных функций
    println!("{} {} {} {}", add_one(42), add_one_v1(42), add_one_v2(42), add_one_v3(42) );
}
```


# Упражнение: замыкания и захват переменных

🟡 **Средний уровень**

- Создайте замыкание, которое захватывает `String` из внешней области видимости и дописывает к ней текст (подсказка: используйте `move`)
- Создайте вектор замыканий `Vec<Box<dyn Fn(i32) -> i32>>`, содержащий замыкания, которые прибавляют 1, умножают на 2 и возводят входное значение в квадрат. Переберите вектор и примените каждое замыкание к числу 5

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
fn main() {
    // Часть 1: замыкание, которое захватывает String и дописывает к ней
    let mut greeting = String::from("Hello");
    let mut append = |suffix: &str| {
        greeting.push_str(suffix);
    };
    append(", world");
    append("!");
    println!("{greeting}");  // "Hello, world!"

    // Часть 2: вектор замыканий
    let operations: Vec<Box<dyn Fn(i32) -> i32>> = vec![
        Box::new(|x| x + 1),      // прибавить 1
        Box::new(|x| x * 2),      // умножить на 2
        Box::new(|x| x * x),      // возвести в квадрат
    ];

    let input = 5;
    for (i, op) in operations.iter().enumerate() {
        println!("Operation {i} on {input}: {}", op(input));
    }
}
// Вывод:
// Hello, world!
// Operation 0 on 5: 6
// Operation 1 on 5: 10
// Operation 2 on 5: 25
```

</details>

# Итераторы в Rust
- Итераторы — одна из самых мощных возможностей Rust. Они позволяют очень изящно выполнять операции над коллекциями, включая фильтрацию (```filter()```), преобразование (```map()```), фильтрацию с преобразованием (```filter_map()```), поиск (```find()```) и многое другое
- В примере ниже ```|&x| *x >= 42``` — замыкание, которое выполняет то же сравнение. ```|x| println!("{x}")``` — ещё одно замыкание
```rust
fn main() {
    let a = [0, 1, 2, 3, 42, 43];
    for x in &a {
        if *x >= 42 {
            println!("{x}");
        }
    }
    // То же, что и выше
    a.iter().filter(|&x| *x >= 42).for_each(|x| println!("{x}"))
}
```

# Итераторы в Rust: ленивость
- Ключевая особенность итераторов в том, что большинство из них ```ленивые```, то есть ничего не делают, пока их не вычислят. Например, ```a.iter().filter(|&x| *x >= 42);``` без ```for_each``` не сделало бы *ничего*. Компилятор Rust выдаёт явное предупреждение, когда замечает такую ситуацию
```rust
fn main() {
    let a = [0, 1, 2, 3, 42, 43];
    // Прибавляем единицу к каждому элементу и выводим его
    let _ = a.iter().map(|x|x + 1).for_each(|x|println!("{x}"));
    let found = a.iter().find(|&x|*x == 42);
    println!("{found:?}");
    // Считаем элементы
    let count = a.iter().count();
    println!("{count}");
}
```

# Итераторы в Rust: сбор результатов
- Метод ```collect()``` позволяет собрать результаты в отдельную коллекцию
    - В примере ниже ```_``` в ```Vec<_>``` — это подстановочный символ для типа, который возвращает ```map```. Например, из ```map``` можно даже вернуть ```String```
```rust
fn main() {
    let a = [0, 1, 2, 3, 42, 43];
    let squared_a : Vec<_> = a.iter().map(|x|x*x).collect();
    for x in &squared_a {
        println!("{x}");
    }
    let squared_a_strings : Vec<_> = a.iter().map(|x|(x*x).to_string()).collect();
    // Это уже строковые представления
    for x in &squared_a_strings {
        println!("{x}");
    }
}
```

# Упражнение: итераторы в Rust

🟢 **Начальный уровень**
- Создайте целочисленный массив из нечётных и чётных элементов. Переберите массив и разделите его на два разных вектора: в одном — чётные элементы, в другом — нечётные
- Можно ли сделать это за один проход (подсказка: используйте ```partition()```)?

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
fn main() {
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Подход 1: ручная итерация
    let mut evens = Vec::new();
    let mut odds = Vec::new();
    for n in numbers {
        if n % 2 == 0 {
            evens.push(n);
        } else {
            odds.push(n);
        }
    }
    println!("Evens: {evens:?}");
    println!("Odds:  {odds:?}");

    // Подход 2: один проход с partition()
    let (evens, odds): (Vec<i32>, Vec<i32>) = numbers
        .into_iter()
        .partition(|n| *n % 2 == 0);
    println!("Evens (partition): {evens:?}");
    println!("Odds  (partition): {odds:?}");
}
// Вывод:
// Evens: [2, 4, 6, 8, 10]
// Odds:  [1, 3, 5, 7, 9]
// Evens (partition): [2, 4, 6, 8, 10]
// Odds  (partition): [1, 3, 5, 7, 9]
```

</details>

> **Промышленные шаблоны**: реальные цепочки итераторов (`.map().collect()`, `.filter().collect()`, `.find_map()`) из продакшн-кода на Rust см. в разделе [Схлопывание пирамид присваиваний с помощью замыканий](ch17-3-collapsing-assignment-pyramids.md#схлопывание-пирамид-присваиваний-с-помощью-замыканий).

### Инструменты для работы с итераторами: методы, заменяющие циклы C++

Следующие адаптеры итераторов *широко* используются в продакшн-коде на Rust. В C++ есть `<algorithm>` и диапазоны C++20, но цепочки итераторов Rust более компонуемы и используются чаще.

#### `enumerate` — индекс и значение (заменяет `for (int i = 0; ...)`)

```rust
let sensors = vec!["temp0", "temp1", "temp2"];
for (idx, name) in sensors.iter().enumerate() {
    println!("Sensor {idx}: {name}");
}
// Sensor 0: temp0
// Sensor 1: temp1
// Sensor 2: temp2
```

Аналог в C++: `for (size_t i = 0; i < sensors.size(); ++i) { auto& name = sensors[i]; ... }`

#### `zip` — попарное объединение элементов двух итераторов (заменяет параллельные циклы по индексу)

```rust
let names = ["gpu0", "gpu1", "gpu2"];
let temps = [72.5, 68.0, 75.3];

let report: Vec<String> = names.iter()
    .zip(temps.iter())
    .map(|(name, temp)| format!("{name}: {temp}°C"))
    .collect();
println!("{report:?}");
// ["gpu0: 72.5°C", "gpu1: 68.0°C", "gpu2: 75.3°C"]

// Останавливается на более коротком итераторе — риска выхода за границы нет
```

Аналог в C++: `for (size_t i = 0; i < std::min(names.size(), temps.size()); ++i) { ... }`

#### `flat_map` — map и flatten для вложенных коллекций

```rust
// У каждого GPU несколько BDF на PCIe; собираем все BDF со всех GPU
let gpu_bdfs = vec![
    vec!["0000:01:00.0", "0000:02:00.0"],
    vec!["0000:41:00.0"],
    vec!["0000:81:00.0", "0000:82:00.0"],
];

let all_bdfs: Vec<&str> = gpu_bdfs.iter()
    .flat_map(|bdfs| bdfs.iter().copied())
    .collect();
println!("{all_bdfs:?}");
// ["0000:01:00.0", "0000:02:00.0", "0000:41:00.0", "0000:81:00.0", "0000:82:00.0"]
```

Аналог в C++: вложенный цикл `for`, который добавляет элементы в один вектор.

#### `chain` — конкатенация двух итераторов

```rust
let critical_gpus = vec!["gpu0", "gpu3"];
let warning_gpus = vec!["gpu1", "gpu5"];

// Обрабатываем все отмеченные GPU, сначала критичные
for gpu in critical_gpus.iter().chain(warning_gpus.iter()) {
    println!("Flagged: {gpu}");
}
```

#### `windows` и `chunks` — скользящие и фиксированные окна над срезами

```rust
let temps = [70, 72, 75, 73, 71, 68, 65];

// windows(3): скользящее окно размера 3 — выявляем тренды
let rising = temps.windows(3)
    .any(|w| w[0] < w[1] && w[1] < w[2]);
println!("Rising trend detected: {rising}"); // true (70 < 72 < 75)

// chunks(2): группы фиксированного размера — обрабатываем парами
for pair in temps.chunks(2) {
    println!("Pair: {pair:?}");
}
// Pair: [70, 72]
// Pair: [75, 73]
// Pair: [71, 68]
// Pair: [65]       ← последний фрагмент может быть меньше
```

Аналог в C++: ручная арифметика индексов с `i` и `i+1`/`i+2`.

#### `fold` — накопление в одно значение (заменяет `std::accumulate`)

```rust
let errors = vec![
    ("gpu0", 3u32),
    ("gpu1", 0),
    ("gpu2", 7),
    ("gpu3", 1),
];

// Считаем общее число ошибок и формируем сводку за один проход
let (total, summary) = errors.iter().fold(
    (0u32, String::new()),
    |(count, mut s), (name, errs)| {
        if *errs > 0 {
            s.push_str(&format!("{name}:{errs} "));
        }
        (count + errs, s)
    },
);
println!("Total errors: {total}, details: {summary}");
// Total errors: 11, details: gpu0:3 gpu2:7 gpu3:1
```

#### `scan` — преобразование с состоянием (накопленная сумма, отслеживание разностей)

```rust
let readings = [100, 105, 103, 110, 108];

// Вычисляем разности между соседними показаниями
let deltas: Vec<i32> = readings.iter()
    .scan(None::<i32>, |prev, &val| {
        let delta = prev.map(|p| val - p);
        *prev = Some(val);
        Some(delta)
    })
    .flatten()  // Убираем начальный None
    .collect();
println!("Deltas: {deltas:?}"); // [5, -2, 7, -2]
```

#### Краткий справочник: цикл C++ → итератор Rust

| **Шаблон C++** | **Итератор Rust** | **Пример** |
|----------------|------------------|------------|
| `for (int i = 0; i < v.size(); i++)` | `.enumerate()` | `v.iter().enumerate()` |
| Параллельный перебор с индексом | `.zip()` | `a.iter().zip(b.iter())` |
| Вложенный цикл → плоский результат | `.flat_map()` | `vecs.iter().flat_map(\|v\| v.iter())` |
| Конкатенация двух контейнеров | `.chain()` | `a.iter().chain(b.iter())` |
| Скользящее окно `v[i..i+n]` | `.windows(n)` | `v.windows(3)` |
| Обработка группами фиксированного размера | `.chunks(n)` | `v.chunks(4)` |
| `std::accumulate` / ручной аккумулятор | `.fold()` | `.fold(init, \|acc, x\| ...)` |
| Накопленная сумма / отслеживание разностей | `.scan()` | `.scan(state, \|s, x\| ...)` |
| `while (it != end && count < n) { ++it; ++count; }` | `.take(n)` | `.iter().take(5)` |
| `while (it != end && !pred(*it)) { ++it; }` | `.skip_while()` | `.skip_while(\|x\| x < &threshold)` |
| `std::any_of` | `.any()` | `.iter().any(\|x\| x > &limit)` |
| `std::all_of` | `.all()` | `.iter().all(\|x\| x.is_valid())` |
| `std::none_of` | `!.any()` | `!iter.any(\|x\| x.failed())` |
| `std::count_if` | `.filter().count()` | `.filter(\|x\| x > &0).count()` |
| `std::min_element` / `std::max_element` | `.min()` / `.max()` | `.iter().max()` → `Option<&T>` |
| `std::unique` | `.dedup()` (на отсортированном) | `v.dedup()` (на месте в Vec) |

### Упражнение: цепочки итераторов

Дано: данные датчиков в виде `Vec<(String, f64)>` (имя, температура). Напишите **одну
цепочку итераторов**, которая:
1. Отфильтровывает датчики с температурой > 80.0
2. Сортирует их по температуре (по убыванию)
3. Форматирует каждый как `"{name}: {temp}°C [ALARM]"`
4. Собирает результат в `Vec<String>`

Подсказка: нужно вызвать `.collect()` перед `.sort_by()`, поскольку для сортировки нужен `Vec`.

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
fn alarm_report(sensors: &[(String, f64)]) -> Vec<String> {
    let mut hot: Vec<_> = sensors.iter()
        .filter(|(_, temp)| *temp > 80.0)
        .collect();
    hot.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    hot.iter()
        .map(|(name, temp)| format!("{name}: {temp}°C [ALARM]"))
        .collect()
}

fn main() {
    let sensors = vec![
        ("gpu0".to_string(), 72.5),
        ("gpu1".to_string(), 85.3),
        ("gpu2".to_string(), 91.0),
        ("gpu3".to_string(), 78.0),
        ("gpu4".to_string(), 88.7),
    ];
    for line in alarm_report(&sensors) {
        println!("{line}");
    }
}
// Вывод:
// gpu2: 91°C [ALARM]
// gpu4: 88.7°C [ALARM]
// gpu1: 85.3°C [ALARM]
```

</details>

----

# Итераторы в Rust: собственные типы
- Трейт ```Iterator``` используется для реализации итерации по пользовательским типам (https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)
    - В примере мы реализуем итератор для последовательности Фибоначчи, которая начинается с 1, 1, 2, ..., а каждый следующий элемент — это сумма двух предыдущих
    - Ассоциированный тип в ```Iterator``` (```type Item = u32;```) задаёт тип значений, которые выдаёт наш итератор (```u32```)
    - Метод ```next()``` содержит саму логику итератора. В данном случае всё состояние хранится в структуре ```Fibonacci```
    - Можно было бы реализовать ещё один трейт, ```IntoIterator```, с методом ```into_iter()``` для более специализированных итераторов
    - [▶ Попробуйте в Rust Playground](https://play.rust-lang.org/)

