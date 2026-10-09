## `Vec<T>` против `List<T>`

> **Что вы узнаете:** `Vec<T>` против `List<T>`, `HashMap` против `Dictionary`, безопасные паттерны доступа
> (почему Rust возвращает `Option` вместо генерации исключения), а также последствия владения для коллекций.
>
> **Сложность:** 🟢 Начальный

`Vec<T>` — аналог `List<T>` из C# в Rust, но с семантикой владения.

### `List<T>` в C#
```csharp
// List<T> в C# — ссылочный тип, размещается в куче
var numbers = new List<int>();
numbers.Add(1);
numbers.Add(2);
numbers.Add(3);

// Передача в метод — ссылка копируется
ProcessList(numbers);
Console.WriteLine(numbers.Count);  // По-прежнему доступен

void ProcessList(List<int> list)
{
    list.Add(4);  // Изменяет исходный список
    Console.WriteLine($"Count in method: {list.Count}");
}
```

### `Vec<T>` в Rust
```rust
// Vec<T> в Rust — владеемый тип, размещается в куче
let mut numbers = Vec::new();
numbers.push(1);
numbers.push(2);
numbers.push(3);

// Метод, который забирает владение
process_vec(numbers);
// println!("{:?}", numbers);  // ❌ Ошибка: numbers было перемещено

// Метод, который заимствует
let mut numbers = vec![1, 2, 3];  // Макрос vec! для удобства
process_vec_borrowed(&mut numbers);
println!("{:?}", numbers);  // ✅ По-прежнему доступен

fn process_vec(mut vec: Vec<i32>) {  // Забирает владение
    vec.push(4);
    println!("Count in method: {}", vec.len());
    // vec освобождается здесь
}

fn process_vec_borrowed(vec: &mut Vec<i32>) {  // Заимствует изменяемо
    vec.push(4);
    println!("Count in method: {}", vec.len());
}
```

### Создание и инициализация векторов
```csharp
// Инициализация List в C#
var numbers = new List<int> { 1, 2, 3, 4, 5 };
var empty = new List<int>();
var sized = new List<int>(10);  // Начальная ёмкость

// Из других коллекций
var fromArray = new List<int>(new[] { 1, 2, 3 });
```

```rust
// Инициализация Vec в Rust
let numbers = vec![1, 2, 3, 4, 5];  // Макрос vec!
let empty: Vec<i32> = Vec::new();   // Для пустого нужна аннотация типа
let sized = Vec::with_capacity(10); // Предварительное выделение ёмкости

// Из итератора
let from_range: Vec<i32> = (1..=5).collect();
let from_array = vec![1, 2, 3];
```

### Сравнение распространённых операций
```csharp
// Операции с List в C#
var list = new List<int> { 1, 2, 3 };

list.Add(4);                    // Добавить элемент
list.Insert(0, 0);              // Вставить по индексу
list.Remove(2);                 // Удалить первое вхождение
list.RemoveAt(1);               // Удалить по индексу
list.Clear();                   // Удалить все

int first = list[0];            // Доступ по индексу
int count = list.Count;         // Получить количество
bool contains = list.Contains(3); // Проверка наличия
```

```rust
// Операции с Vec в Rust
let mut vec = vec![1, 2, 3];

vec.push(4);                    // Добавить элемент
vec.insert(0, 0);               // Вставить по индексу
vec.retain(|&x| x != 2);        // Удалить элементы (в функциональном стиле)
vec.remove(1);                  // Удалить по индексу
vec.clear();                    // Удалить все

let first = vec[0];             // Доступ по индексу (паника, если выход за границы)
let safe_first = vec.get(0);    // Безопасный доступ, возвращает Option<&T>
let count = vec.len();          // Получить количество
let contains = vec.contains(&3); // Проверка наличия
```

### Паттерны безопасного доступа
```csharp
// C# — проверка границ через исключения
public int SafeAccess(List<int> list, int index)
{
    try
    {
        return list[index];
    }
    catch (ArgumentOutOfRangeException)
    {
        return -1;  // Значение по умолчанию
    }
}
```

```rust
// Rust — безопасный доступ через Option
fn safe_access(vec: &[i32], index: usize) -> Option<i32> {
    vec.get(index).copied()  // Возвращает Option<i32>
}

fn main() {
    let vec = vec![1, 2, 3];
    
    // Паттерны безопасного доступа
    match vec.get(10) {
        Some(value) => println!("Value: {}", value),
        None => println!("Индекс вне границ"),
    }
    
    // Или через unwrap_or
    let value = vec.get(10).copied().unwrap_or(-1);
    println!("Value: {}", value);
}
```

***

## HashMap против Dictionary

HashMap — аналог `Dictionary<K,V>` из C# в Rust.

### Dictionary в C#
```csharp
// Dictionary<TKey, TValue> в C#
var scores = new Dictionary<string, int>
{
    ["Alice"] = 100,
    ["Bob"] = 85,
    ["Charlie"] = 92
};

// Добавление/обновление
scores["Dave"] = 78;
scores["Alice"] = 105;  // Обновление существующего

// Безопасный доступ
if (scores.TryGetValue("Eve", out int score))
{
    Console.WriteLine($"Eve's score: {score}");
}
else
{
    Console.WriteLine("Eve not found");
}

// Итерация
foreach (var kvp in scores)
{
    Console.WriteLine($"{kvp.Key}: {kvp.Value}");
}
```

### HashMap в Rust
```rust
use std::collections::HashMap;

// Создание и инициализация HashMap
let mut scores = HashMap::new();
scores.insert("Alice".to_string(), 100);
scores.insert("Bob".to_string(), 85);
scores.insert("Charlie".to_string(), 92);

// Или из итератора
let scores: HashMap<String, i32> = [
    ("Alice".to_string(), 100),
    ("Bob".to_string(), 85),
    ("Charlie".to_string(), 92),
].into_iter().collect();

// Добавление/обновление
let mut scores = scores;  // Делаем изменяемой
scores.insert("Dave".to_string(), 78);
scores.insert("Alice".to_string(), 105);  // Обновление существующего

// Безопасный доступ
match scores.get("Eve") {
    Some(score) => println!("Eve's score: {}", score),
    None => println!("Eve not found"),
}

// Итерация
for (name, score) in &scores {
    println!("{}: {}", name, score);
}
```

### Операции с HashMap
```csharp
// Операции с Dictionary в C#
var dict = new Dictionary<string, int>();

dict["key"] = 42;                    // Вставка/обновление
bool exists = dict.ContainsKey("key"); // Проверка наличия
bool removed = dict.Remove("key");    // Удаление
dict.Clear();                        // Очистка

// Получение со значением по умолчанию
int value = dict.GetValueOrDefault("missing", 0);
```

```rust
use std::collections::HashMap;

// Операции с HashMap в Rust
let mut map = HashMap::new();

map.insert("key".to_string(), 42);   // Вставка/обновление
let exists = map.contains_key("key"); // Проверка наличия
let removed = map.remove("key");      // Удаление, возвращает Option<V>
map.clear();                         // Очистка

// Entry API для продвинутых операций
let mut map = HashMap::new();
map.entry("key".to_string()).or_insert(42);  // Вставить, если не существует
map.entry("key".to_string()).and_modify(|v| *v += 1); // Изменить, если существует

// Получение со значением по умолчанию
let value = map.get("missing").copied().unwrap_or(0);
```

### Владение ключами и значениями HashMap
```rust
// Понимание владения в HashMap
fn ownership_example() {
    let mut map = HashMap::new();
    
    // Строковые ключи и значения перемещаются в map
    let key = String::from("name");
    let value = String::from("Alice");
    
    map.insert(key, value);
    // println!("{}", key);   // ❌ Ошибка: key было перемещено
    // println!("{}", value); // ❌ Ошибка: value было перемещено
    
    // Доступ через ссылки
    if let Some(name) = map.get("name") {
        println!("Name: {}", name);  // Заимствуем значение
    }
}

// Использование ключей &str (без передачи владения)
fn string_slice_keys() {
    let mut map = HashMap::new();
    
    map.insert("name", "Alice");     // Ключи и значения типа &str
    map.insert("age", "30");
    
    // Со строковыми литералами проблем с владением нет
    println!("Name exists: {}", map.contains_key("name"));
}
```

***

## Работа с коллекциями

### Паттерны итерации
```csharp
// Паттерны итерации в C#
var numbers = new List<int> { 1, 2, 3, 4, 5 };

// Цикл for с индексом
for (int i = 0; i < numbers.Count; i++)
{
    Console.WriteLine($"Index {i}: {numbers[i]}");
}

// Цикл foreach
foreach (int num in numbers)
{
    Console.WriteLine(num);
}

// Методы LINQ
var doubled = numbers.Select(x => x * 2).ToList();
var evens = numbers.Where(x => x % 2 == 0).ToList();
```

```rust
// Паттерны итерации в Rust
let numbers = vec![1, 2, 3, 4, 5];

// Цикл for с индексом
for (i, num) in numbers.iter().enumerate() {
    println!("Index {}: {}", i, num);
}

// Цикл for по значениям
for num in &numbers {  // Заимствуем каждый элемент
    println!("{}", num);
}

// Методы итераторов (как LINQ)
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
let evens: Vec<i32> = numbers.iter().filter(|&x| x % 2 == 0).cloned().collect();

// Или эффективнее — потребляющий итератор
let doubled: Vec<i32> = numbers.into_iter().map(|x| x * 2).collect();
```

### Iterator против IntoIterator против Iter
```rust
// Понимание различных способов итерации
fn iteration_methods() {
    let vec = vec![1, 2, 3, 4, 5];
    
    // 1. iter() — заимствует элементы (&T)
    for item in vec.iter() {
        println!("{}", item);  // item имеет тип &i32
    }
    // vec здесь по-прежнему доступен
    
    // 2. into_iter() — забирает владение (T)
    for item in vec.into_iter() {
        println!("{}", item);  // item имеет тип i32
    }
    // vec здесь больше недоступен
    
    let mut vec = vec![1, 2, 3, 4, 5];
    
    // 3. iter_mut() — изменяемые заимствования (&mut T)
    for item in vec.iter_mut() {
        *item *= 2;  // item имеет тип &mut i32
    }
    println!("{:?}", vec);  // [2, 4, 6, 8, 10]
}
```

### Сбор результатов
```csharp
// C# — обработка коллекций с возможными ошибками
public List<int> ParseNumbers(List<string> inputs)
{
    var results = new List<int>();
    foreach (string input in inputs)
    {
        if (int.TryParse(input, out int result))
        {
            results.Add(result);
        }
        // Молча пропускаем некорректные значения
    }
    return results;
}
```

```rust
// Rust — явная обработка ошибок через collect
fn parse_numbers(inputs: Vec<String>) -> Result<Vec<i32>, std::num::ParseIntError> {
    inputs.into_iter()
        .map(|s| s.parse::<i32>())  // Возвращает Result<i32, ParseIntError>
        .collect()                  // Собирает в Result<Vec<i32>, ParseIntError>
}

// Альтернатива: отфильтровать ошибки
fn parse_numbers_filter(inputs: Vec<String>) -> Vec<i32> {
    inputs.into_iter()
        .filter_map(|s| s.parse::<i32>().ok())  // Оставляем только значения Ok
        .collect()
}

fn main() {
    let inputs = vec!["1".to_string(), "2".to_string(), "invalid".to_string(), "4".to_string()];
    
    // Вариант, который завершается при первой ошибке
    match parse_numbers(inputs.clone()) {
        Ok(numbers) => println!("All parsed: {:?}", numbers),
        Err(error) => println!("Parse error: {}", error),
    }
    
    // Вариант, который пропускает ошибки
    let numbers = parse_numbers_filter(inputs);
    println!("Successfully parsed: {:?}", numbers);  // [1, 2, 4]
}
```

---

## Упражнения

<details>
<summary><strong>🏋️ Упражнение: из LINQ в итераторы</strong> (нажмите, чтобы раскрыть)</summary>

Переведите этот LINQ-запрос на C# в идиоматичные итераторы Rust:

```csharp
var result = students
    .Where(s => s.Grade >= 90)
    .OrderByDescending(s => s.Grade)
    .Select(s => $"{s.Name}: {s.Grade}")
    .Take(3)
    .ToList();
```

Используйте эту структуру:
```rust
struct Student { name: String, grade: u32 }
```

Верните `Vec<String>` из трёх лучших студентов с оценкой ≥ 90, отформатированных как `"Name: Grade"`.

<details>
<summary>🔑 Решение</summary>

```rust
#[derive(Debug)]
struct Student { name: String, grade: u32 }

fn top_students(students: &mut [Student]) -> Vec<String> {
    students.sort_by(|a, b| b.grade.cmp(&a.grade)); // сортировка по убыванию
    students.iter()
        .filter(|s| s.grade >= 90)
        .take(3)
        .map(|s| format!("{}: {}", s.name, s.grade))
        .collect()
}

fn main() {
    let mut students = vec![
        Student { name: "Alice".into(), grade: 95 },
        Student { name: "Bob".into(), grade: 88 },
        Student { name: "Carol".into(), grade: 92 },
        Student { name: "Dave".into(), grade: 97 },
        Student { name: "Eve".into(), grade: 91 },
    ];
    let result = top_students(&mut students);
    assert_eq!(result, vec!["Dave: 97", "Alice: 95", "Carol: 92"]);
    println!("{result:?}");
}
```

**Ключевое отличие от C#**: итераторы Rust ленивы (как LINQ), но `.sort_by()` — энергичная операция, которая сортирует на месте; ленивого `OrderBy` здесь нет. Сначала сортируете, затем строите цепочку ленивых операций.

</details>
</details>

***


