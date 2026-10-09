## Замыкания в Rust

> **Что вы узнаете:** замыкания с захватом с учётом владения (`Fn`/`FnMut`/`FnOnce`) в сравнении с лямбдами C#,
> итераторы Rust как замену LINQ с нулевой стоимостью, ленивые и энергичные вычисления,
> а также параллельную итерацию с `rayon`.
>
> **Сложность:** 🟡 Средний

Замыкания в Rust похожи на лямбды и делегаты C#, но захватывают переменные с учётом владения.

### Лямбды и делегаты C#
```csharp
// C# — лямбды захватывают по ссылке
Func<int, int> doubler = x => x * 2;
Action<string> printer = msg => Console.WriteLine(msg);

// Замыкание, захватывающее внешние переменные
int multiplier = 3;
Func<int, int> multiply = x => x * multiplier;
Console.WriteLine(multiply(5)); // 15

// LINQ активно использует лямбды
var evens = numbers.Where(n => n % 2 == 0).ToList();
```

### Замыкания в Rust
```rust
// Замыкания Rust — с учётом владения
let doubler = |x: i32| x * 2;
let printer = |msg: &str| println!("{}", msg);

// Замыкание, захватывающее по ссылке (по умолчанию для неизменяемых)
let multiplier = 3;
let multiply = |x: i32| x * multiplier; // заимствует multiplier
println!("{}", multiply(5)); // 15
println!("{}", multiplier); // по-прежнему доступен

// Замыкание, захватывающее перемещением
let data = vec![1, 2, 3];
let owns_data = move || {
    println!("{:?}", data); // data перемещается в замыкание
};
owns_data();
// println!("{:?}", data); // ОШИБКА: data было перемещено

// Использование замыканий с итераторами
let numbers = vec![1, 2, 3, 4, 5];
let evens: Vec<&i32> = numbers.iter().filter(|&&n| n % 2 == 0).collect();
```

### Виды замыканий
```rust
// Fn — заимствует захваченные значения неизменяемо
fn apply_fn(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

// FnMut — заимствует захваченные значения изменяемо
fn apply_fn_mut(mut f: impl FnMut(i32), values: &[i32]) {
    for &v in values {
        f(v);
    }
}

// FnOnce — забирает владение захваченными значениями
fn apply_fn_once(f: impl FnOnce() -> Vec<i32>) -> Vec<i32> {
    f() // можно вызвать только один раз
}

fn main() {
    // Пример Fn
    let multiplier = 3;
    let result = apply_fn(|x| x * multiplier, 5);
    
    // Пример FnMut
    let mut sum = 0;
    apply_fn_mut(|x| sum += x, &[1, 2, 3, 4, 5]);
    println!("Sum: {}", sum); // 15
    
    // Пример FnOnce
    let data = vec![1, 2, 3];
    let result = apply_fn_once(move || data); // перемещает data
}
```

***

## LINQ против итераторов Rust

### LINQ в C# (Language Integrated Query)
```csharp
// LINQ в C# — декларативная обработка данных
var numbers = new[] { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 };

var result = numbers
    .Where(n => n % 2 == 0)           // Отбираем чётные числа
    .Select(n => n * n)               // Возводим в квадрат
    .Where(n => n > 10)               // Отбираем > 10
    .OrderByDescending(n => n)        // Сортируем по убыванию
    .Take(3)                          // Берём первые 3
    .ToList();                        // Материализуем

// LINQ со сложными объектами
var users = GetUsers();
var activeAdults = users
    .Where(u => u.IsActive && u.Age >= 18)
    .GroupBy(u => u.Department)
    .Select(g => new {
        Department = g.Key,
        Count = g.Count(),
        AverageAge = g.Average(u => u.Age)
    })
    .OrderBy(x => x.Department)
    .ToList();

// Асинхронный LINQ (с дополнительными библиотеками)
var results = await users
    .ToAsyncEnumerable()
    .WhereAwait(async u => await IsActiveAsync(u.Id))
    .SelectAwait(async u => await EnrichUserAsync(u))
    .ToListAsync();
```

### Итераторы Rust
```rust
// Итераторы Rust — ленивые абстракции с нулевой стоимостью
let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

let result: Vec<i32> = numbers
    .iter()
    .filter(|&&n| n % 2 == 0)        // Отбираем чётные числа
    .map(|&n| n * n)                 // Возводим в квадрат
    .filter(|&n| n > 10)             // Отбираем > 10
    .collect::<Vec<_>>()             // Собираем в Vec
    .into_iter()
    .rev()                           // Меняем порядок обхода
    .take(3)                         // Берём первые 3
    .collect();                      // Материализуем

// Сложные цепочки итераторов
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    department: String,
    is_active: bool,
}

fn process_users(users: Vec<User>) -> HashMap<String, (usize, f64)> {
    users
        .into_iter()
        .filter(|u| u.is_active && u.age >= 18)
        .fold(HashMap::new(), |mut acc, user| {
            let entry = acc.entry(user.department.clone()).or_insert((0, 0.0));
            entry.0 += 1;  // количество
            entry.1 += user.age as f64;  // сумма возрастов
            acc
        })
        .into_iter()
        .map(|(dept, (count, sum))| (dept, (count, sum / count as f64)))  // среднее
        .collect()
}

// Параллельная обработка с rayon
use rayon::prelude::*;

fn parallel_processing(numbers: Vec<i32>) -> Vec<i32> {
    numbers
        .par_iter()                  // Параллельный итератор
        .filter(|&&n| n % 2 == 0)
        .map(|&n| expensive_computation(n))
        .collect()
}

fn expensive_computation(n: i32) -> i32 {
    // Имитация тяжёлых вычислений
    (0..1000).fold(n, |acc, _| acc + 1)
}
```

```mermaid
graph TD
    subgraph "Особенности LINQ в C#"
        CS_LINQ["Выражение LINQ"]
        CS_EAGER["Часто энергичные вычисления<br/>(ToList(), ToArray())"]
        CS_REFLECTION["[ERROR] Часть рефлексии во время выполнения<br/>Деревья выражений"]
        CS_ALLOCATIONS["[ERROR] Промежуточные коллекции<br/>Нагрузка на сборщик мусора"]
        CS_ASYNC["[OK] Поддержка async<br/>(с дополнительными библиотеками)"]
        CS_SQL["[OK] Интеграция LINQ to SQL/EF"]
        
        CS_LINQ --> CS_EAGER
        CS_LINQ --> CS_REFLECTION
        CS_LINQ --> CS_ALLOCATIONS
        CS_LINQ --> CS_ASYNC
        CS_LINQ --> CS_SQL
    end
    
    subgraph "Особенности итераторов Rust"
        RUST_ITER["Цепочка итераторов"]
        RUST_LAZY["[OK] Ленивые вычисления<br/>Никакой работы до .collect()"]
        RUST_ZERO["[OK] Абстракции с нулевой стоимостью<br/>Компилируются в оптимальные циклы"]
        RUST_NO_ALLOC["[OK] Без промежуточных выделений<br/>Обработка на стеке"]
        RUST_PARALLEL["[OK] Простая параллелизация<br/>(крейт rayon)"]
        RUST_FUNCTIONAL["[OK] Функциональное программирование<br/>Неизменяемость по умолчанию"]
        
        RUST_ITER --> RUST_LAZY
        RUST_ITER --> RUST_ZERO
        RUST_ITER --> RUST_NO_ALLOC
        RUST_ITER --> RUST_PARALLEL
        RUST_ITER --> RUST_FUNCTIONAL
    end
    
    subgraph "Сравнение производительности"
        CS_PERF["Производительность LINQ в C#<br/>[ERROR] Накладные расходы на выделение памяти<br/>[ERROR] Виртуальная диспетчеризация<br/>[OK] Достаточно для большинства случаев"]
        RUST_PERF["Производительность итераторов Rust<br/>[OK] Скорость уровня ручной оптимизации<br/>[OK] Без выделений памяти<br/>[OK] Оптимизация на этапе компиляции"]
    end
    
    style CS_REFLECTION fill:#ffcdd2,color:#000
    style CS_ALLOCATIONS fill:#fff3e0,color:#000
    style RUST_ZERO fill:#c8e6c9,color:#000
    style RUST_LAZY fill:#c8e6c9,color:#000
    style RUST_NO_ALLOC fill:#c8e6c9,color:#000
    style CS_PERF fill:#fff3e0,color:#000
    style RUST_PERF fill:#c8e6c9,color:#000
```

***


<details>
<summary><strong>🏋️ Упражнение: перевод из LINQ в итераторы</strong> (нажмите, чтобы раскрыть)</summary>

**Задача**: переведите этот конвейер LINQ на C# в идиоматичные итераторы Rust.

```csharp
// C# — переведите на Rust
record Employee(string Name, string Dept, int Salary);

var result = employees
    .Where(e => e.Salary > 50_000)
    .GroupBy(e => e.Dept)
    .Select(g => new {
        Department = g.Key,
        Count = g.Count(),
        AvgSalary = g.Average(e => e.Salary)
    })
    .OrderByDescending(x => x.AvgSalary)
    .ToList();
```

<details>
<summary>🔑 Решение</summary>

```rust
use std::collections::HashMap;

struct Employee { name: String, dept: String, salary: u32 }

#[derive(Debug)]
struct DeptStats { department: String, count: usize, avg_salary: f64 }

fn department_stats(employees: &[Employee]) -> Vec<DeptStats> {
    let mut by_dept: HashMap<&str, Vec<u32>> = HashMap::new();
    for e in employees.iter().filter(|e| e.salary > 50_000) {
        by_dept.entry(&e.dept).or_default().push(e.salary);
    }

    let mut stats: Vec<DeptStats> = by_dept
        .into_iter()
        .map(|(dept, salaries)| {
            let count = salaries.len();
            let avg = salaries.iter().sum::<u32>() as f64 / count as f64;
            DeptStats { department: dept.to_string(), count, avg_salary: avg }
        })
        .collect();

    stats.sort_by(|a, b| b.avg_salary.partial_cmp(&a.avg_salary).unwrap());
    stats
}
```

**Ключевые выводы**:
- В Rust нет встроенного `group_by` для итераторов — идиоматичный паттерн это `HashMap` + `fold`/`for`
- Крейт `itertools` добавляет `.group_by()` для синтаксиса, более похожего на LINQ
- Цепочки итераторов не имеют накладных расходов — компилятор превращает их в простые циклы

</details>
</details>


<!-- ch12.0a: itertools — LINQ Power Tools -->
## itertools: недостающие операции LINQ

Стандартные итераторы Rust покрывают `map`, `filter`, `fold`, `take` и `collect`. Но разработчики C#, привыкшие к `GroupBy`, `Zip`, `Chunk`, `SelectMany` и `Distinct`, сразу заметят пробелы. Крейт **`itertools`** их восполняет.

```toml
# Cargo.toml
[dependencies]
itertools = "0.12"
```

### Бок о бок: LINQ и itertools

```csharp
// C# — GroupBy
var byDept = employees.GroupBy(e => e.Department)
    .Select(g => new { Dept = g.Key, Count = g.Count() });

// C# — Chunk (пакетирование)
var batches = items.Chunk(100);  // IEnumerable<T[]>

// C# — Distinct / DistinctBy
var unique = users.DistinctBy(u => u.Email);

// C# — SelectMany (уплощение)
var allTags = posts.SelectMany(p => p.Tags);

// C# — Zip
var pairs = names.Zip(scores, (n, s) => new { Name = n, Score = s });

// C# — скользящее окно
var windows = data.Zip(data.Skip(1), data.Skip(2))
    .Select(triple => (triple.First + triple.Second + triple.Third) / 3.0);
```

```rust
use itertools::Itertools;

// Rust — group_by (требует отсортированный ввод)
let by_dept = employees.iter()
    .sorted_by_key(|e| &e.department)
    .group_by(|e| &e.department);
for (dept, group) in &by_dept {
    println!("{}: {} employees", dept, group.count());
}

// Rust — chunks (пакетирование)
let batches = items.iter().chunks(100);
for batch in &batches {
    process_batch(batch.collect::<Vec<_>>());
}

// Rust — unique / unique_by
let unique: Vec<_> = users.iter().unique_by(|u| &u.email).collect();

// Rust — flat_map (аналог SelectMany — встроен в std!)
let all_tags: Vec<&str> = posts.iter().flat_map(|p| &p.tags).collect();

// Rust — zip (встроен в std!)
let pairs: Vec<_> = names.iter().zip(scores.iter()).collect();

// Rust — tuple_windows (скользящее окно)
let moving_avg: Vec<f64> = data.iter()
    .tuple_windows::<(_, _, _)>()
    .map(|(a, b, c)| (*a + *b + *c) as f64 / 3.0)
    .collect();
```

### Краткая справка по itertools

| Метод LINQ | Аналог в itertools | Примечания |
|------------|---------------------|-------|
| `GroupBy(key)` | `.sorted_by_key().group_by()` | Требует отсортированный ввод (в отличие от LINQ) |
| `Chunk(n)` | `.chunks(n)` | Возвращает итератор итераторов |
| `Distinct()` | `.unique()` | Требует `Eq + Hash` |
| `DistinctBy(key)` | `.unique_by(key)` | |
| `SelectMany()` | `.flat_map()` | Встроен в std — крейт не нужен |
| `Zip()` | `.zip()` | Встроен в std |
| `Aggregate()` | `.fold()` | Встроен в std |
| `Any()` / `All()` | `.any()` / `.all()` | Встроены в std |
| `First()` / `Last()` | `.next()` / `.last()` | Встроены в std |
| `Skip(n)` / `Take(n)` | `.skip(n)` / `.take(n)` | Встроены в std |
| `OrderBy()` | `.sorted()` / `.sorted_by()` | `itertools` (в std сортировки нет) |
| `ThenBy()` | `.sorted_by(\|a,b\| a.x.cmp(&b.x).then(a.y.cmp(&b.y)))` | Цепочка через `Ordering::then` |
| `Intersect()` | Пересечение `HashSet` | Прямого метода итератора нет |
| `Concat()` | `.chain()` | Встроен в std |
| Скользящее окно | `.tuple_windows()` | Кортежи фиксированного размера |
| Декартово произведение | `.cartesian_product()` | `itertools` |
| Чередование | `.interleave()` | `itertools` |
| Перестановки | `.permutations(k)` | `itertools` |

### Пример из практики: конвейер анализа логов

```rust
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug)]
struct LogEntry { level: String, module: String, message: String }

fn analyze_logs(entries: &[LogEntry]) {
    // Топ-5 самых «шумных» модулей (как LINQ GroupBy + OrderByDescending + Take)
    let noisy: Vec<_> = entries.iter()
        .into_group_map_by(|e| &e.module) // itertools: сразу группируем в HashMap
        .into_iter()
        .sorted_by(|a, b| b.1.len().cmp(&a.1.len()))
        .take(5)
        .collect();

    for (module, entries) in &noisy {
        println!("{}: {} entries", module, entries.len());
    }

    // Доля ошибок в окне из 100 записей (скользящее окно)
    let error_rates: Vec<f64> = entries.iter()
        .map(|e| if e.level == "ERROR" { 1.0 } else { 0.0 })
        .collect::<Vec<_>>()
        .windows(100)  // метод среза из std
        .map(|w| w.iter().sum::<f64>() / 100.0)
        .collect();

    // Удаляем подряд идущие одинаковые сообщения
    let deduped: Vec<_> = entries.iter().dedup_by(|a, b| a.message == b.message).collect();
    println!("Deduped {} → {} entries", entries.len(), deduped.len());
}
```

***


