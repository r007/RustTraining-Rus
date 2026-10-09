## Переменные и изменяемость

> **Что вы узнаете:** модель объявления переменных и изменяемости в Rust в сравнении с `var`/`const` в C#,
> соответствие примитивных типов, важнейшее различие между `String` и `&str`, вывод типов,
> а также то, как Rust по-другому обрабатывает приведение и преобразования типов.
>
> **Сложность:** 🟢 Начальный

### Объявление переменных в C#
```csharp
// C# — переменные по умолчанию изменяемы
int count = 0;           // Изменяемая
count = 5;               // ✅ Работает

// readonly-поля (только на уровне класса, не для локальных переменных)
// readonly int maxSize = 100;  // Неизменяемо после инициализации

const int BUFFER_SIZE = 1024; // Константа времени компиляции (работает как локальная переменная или поле)
```

### Объявление переменных в Rust
```rust
// Rust — переменные по умолчанию неизменяемы
let count = 0;           // Неизменяемая по умолчанию
// count = 5;            // ❌ Ошибка компиляции: нельзя дважды присвоить неизменяемой переменной

let mut count = 0;       // Явно изменяемая
count = 5;               // ✅ Работает

const BUFFER_SIZE: usize = 1024; // Константа времени компиляции
```

### Ключевой сдвиг мышления для разработчиков C#
```rust
// Думайте о 'let' как о семантике readonly-поля, применённой ко всем переменным
let name = "John";       // Как readonly-поле: после присваивания изменить нельзя
let mut age = 30;        // Как: int age = 30;

// Затенение переменных (особенность Rust)
let spaces = "   ";      // Строка
let spaces = spaces.len(); // Теперь это число (usize)
// Это не мутация — мы создаём новую переменную
```

### Практический пример: счётчик
```csharp
// Версия на C#
public class Counter
{
    private int value = 0;
    
    public void Increment()
    {
        value++;  // Мутация
    }
    
    public int GetValue() => value;
}
```

```rust
// Версия на Rust
pub struct Counter {
    value: i32,  // По умолчанию приватное
}

impl Counter {
    pub fn new() -> Counter {
        Counter { value: 0 }
    }
    
    pub fn increment(&mut self) {  // &mut нужен для мутации
        self.value += 1;
    }
    
    pub fn get_value(&self) -> i32 {
        self.value
    }
}
```

***

## Сравнение типов данных

### Примитивные типы

| Тип C# | Тип Rust | Размер | Диапазон |
|---------|-----------|------|-------|
| `byte` | `u8` | 8 бит | от 0 до 255 |
| `sbyte` | `i8` | 8 бит | от -128 до 127 |
| `short` | `i16` | 16 бит | от -32 768 до 32 767 |
| `ushort` | `u16` | 16 бит | от 0 до 65 535 |
| `int` | `i32` | 32 бита | от -2³¹ до 2³¹-1 |
| `uint` | `u32` | 32 бита | от 0 до 2³²-1 |
| `long` | `i64` | 64 бита | от -2⁶³ до 2⁶³-1 |
| `ulong` | `u64` | 64 бита | от 0 до 2⁶⁴-1 |
| `float` | `f32` | 32 бита | IEEE 754 |
| `double` | `f64` | 64 бита | IEEE 754 |
| `bool` | `bool` | 1 бит | true/false |
| `char` | `char` | 32 бита | Скалярное значение Unicode |

### Типы размера (важно!)
```csharp
// C# — int всегда 32-битный
int arrayIndex = 0;
long fileSize = file.Length;
```

```rust
// Rust — типы размера совпадают с размером указателя (32 или 64 бита)
let array_index: usize = 0;    // Как size_t в C
let file_size: u64 = file.len(); // Явно 64-битный
```

### Вывод типов
```csharp
// C# — ключевое слово var
var name = "John";        // string
var count = 42;           // int
var price = 29.99;        // double
```

```rust
// Rust — автоматический вывод типов
let name = "John";        // &str (срез строки)
let count = 42;           // i32 (целое по умолчанию)
let price = 29.99;        // f64 (вещественное по умолчанию)

// Явные аннотации типов
let count: u32 = 42;
let price: f32 = 29.99;
```

### Обзор массивов и коллекций
```csharp
// C# — ссылочные типы, размещаются в куче
int[] numbers = new int[5];        // Фиксированный размер
List<int> list = new List<int>();  // Динамический размер
```

```rust
// Rust — несколько вариантов
let numbers: [i32; 5] = [1, 2, 3, 4, 5];  // Массив на стеке, фиксированный размер
let mut list: Vec<i32> = Vec::new();       // Вектор в куче, динамический размер
```

***

## Типы строк: String и &str

Это одна из самых запутанных концепций для разработчиков C#, поэтому разберём её внимательно.

### Работа со строками в C#
```csharp
// C# — простая модель строк
string name = "John";           // Строковый литерал
string greeting = "Hello, " + name;  // Конкатенация строк
string upper = name.ToUpper();  // Вызов метода
```

### Строковые типы Rust
```rust
// Rust — два основных строковых типа

// 1. &str (срез строки) — похож на ReadOnlySpan<char> в C#
let name: &str = "John";        // Строковый литерал (неизменяемый, заимствованный)

// 2. String — похож на StringBuilder или изменяемую строку
let mut greeting = String::new();       // Пустая строка
greeting.push_str("Hello, ");          // Добавление
greeting.push_str(name);               // Добавление

// Или создаём сразу
let greeting = String::from("Hello, John");
let greeting = "Hello, John".to_string();  // Преобразование &str в String
```

### Когда что использовать?

| Сценарий | Использовать | Аналог в C# |
|----------|-----|---------------|
| Строковые литералы | `&str` | Строковый литерал `string` |
| Параметры функций (только для чтения) | `&str` | `string` или `ReadOnlySpan<char>` |
| Владеемые, изменяемые строки | `String` | `StringBuilder` |
| Возврат владеемых строк | `String` | `string` |

### Практические примеры
```rust
// Функция, принимающая любой строковый тип
fn greet(name: &str) {  // Принимает и String, и &str
    println!("Hello, {}!", name);
}

fn main() {
    let literal = "John";                    // &str
    let owned = String::from("Jane");        // String
    
    greet(literal);                          // Работает
    greet(&owned);                           // Работает (заимствуем String как &str)
    greet("Bob");                            // Работает
}

// Функция, возвращающая владеемую строку
fn create_greeting(name: &str) -> String {
    format!("Hello, {}!", name)  // Макрос format! возвращает String
}
```

### Разработчикам C# — смотрите так
```rust
// &str — это как ReadOnlySpan<char>: представление данных строки
// String — это как char[], которым вы владеете и которое можете изменять

let borrowed: &str = "Я не владею этими данными";
let owned: String = String::from("Я владею этими данными");

// Преобразования между ними
let owned_copy: String = borrowed.to_string();  // Копия во владение
let borrowed_view: &str = &owned;               // Заимствование из owned
```

***

## Вывод и форматирование строк

Разработчики C# активно используют `Console.WriteLine` и интерполяцию строк (`$""`). Система форматирования Rust не менее мощная, но основана на макросах и спецификаторах формата.

### Базовый вывод
```csharp
// Вывод в C#
Console.Write("без перевода строки");
Console.WriteLine("с переводом строки");
Console.Error.WriteLine("в stderr");

// Интерполяция строк (C# 6+)
string name = "Alice";
int age = 30;
Console.WriteLine($"{name} is {age} years old");
```

```rust
// Вывод в Rust — всё через макросы (обратите внимание на !)
print!("без перевода строки");              // → stdout, без перевода строки
println!("с переводом строки");              // → stdout + перевод строки
eprint!("в stderr");                          // → stderr, без перевода строки
eprintln!("в stderr с переводом строки"); // → stderr + перевод строки

// Форматирование строк (аналог интерполяции $"")
let name = "Alice";
let age = 30;
println!("{name} is {age} years old");     // Захват переменной прямо в строке (Rust 1.58+)
println!("{} is {} years old", name, age); // Позиционные аргументы

// format! возвращает String вместо вывода
let msg = format!("{name} is {age} years old");
```

### Спецификаторы формата
```csharp
// Спецификаторы формата в C#
Console.WriteLine($"{price:F2}");         // Фиксированная запись: 29.99
Console.WriteLine($"{count:D5}");         // Дополнение нулями: 00042
Console.WriteLine($"{value,10}");         // Выравнивание по правому краю, ширина 10
Console.WriteLine($"{value,-10}");        // Выравнивание по левому краю, ширина 10
Console.WriteLine($"{hex:X}");            // Шестнадцатеричный: FF
Console.WriteLine($"{ratio:P1}");         // Проценты: 85.0%
```

```rust
// Спецификаторы формата в Rust
println!("{price:.2}");          // 2 знака после запятой: 29.99
println!("{count:05}");          // Дополнение нулями, ширина 5: 00042
println!("{value:>10}");         // Выравнивание по правому краю, ширина 10
println!("{value:<10}");         // Выравнивание по левому краю, ширина 10
println!("{value:^10}");         // Выравнивание по центру, ширина 10
println!("{hex:#X}");            // Шестнадцатеричный с префиксом: 0xFF
println!("{hex:08X}");           // Шестнадцатеричный с нулями: 000000FF
println!("{bits:#010b}");        // Двоичный с префиксом: 0b00001010
println!("{big}", big = 1_000_000); // Именованный параметр
```

### Вывод Debug и Display
```rust
// {:?}  — трейт Debug (для разработчиков, реализуется автоматически)
// {:#?} — Debug с красивым форматированием (с отступами, в несколько строк)
// {}    — трейт Display (для пользователей, нужно реализовать вручную)

#[derive(Debug)] // Автоматически генерирует вывод Debug
struct Point { x: f64, y: f64 }

let p = Point { x: 1.5, y: 2.7 };

println!("{:?}", p);   // Point { x: 1.5, y: 2.7 }   — компактный debug
println!("{:#?}", p);  // Point {                     — debug с форматированием
                        //     x: 1.5,
                        //     y: 2.7,
                        // }
// println!("{}", p);  // ❌ ОШИБКА: Point не реализует Display

// Реализуем Display для вывода, понятного пользователю:
use std::fmt;

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
println!("{}", p);    // (1.5, 2.7)  — удобно для пользователя
```

```csharp
// Эквивалент в C#:
// {:?}  ≈ object.GetType().ToString() или дамп через рефлексию
// {}    ≈ object.ToString()
// В C# вы переопределяете ToString(); в Rust вы реализуете Display
```

### Краткая справка

| C# | Rust | Вывод |
|----|------|--------|
| `Console.WriteLine(x)` | `println!("{x}")` | Форматирование Display |
| `$"{x}"` (интерполяция) | `format!("{x}")` | Возвращает `String` |
| `x.ToString()` | `x.to_string()` | Требует трейт `Display` |
| Переопределение `ToString()` | `impl Display` | Вывод для пользователя |
| Просмотр в отладчике | `{:?}` или `dbg!(x)` | Вывод для разработчика |
| `String.Format("{0:F2}", x)` | `format!("{x:.2}")` | Форматированная `String` |
| `Console.Error.WriteLine` | `eprintln!()` | Запись в stderr |

***

## Приведение типов и преобразования

В C# есть неявные преобразования, явные приведения `(int)x` и `Convert.To*()`. Rust строже — неявных числовых преобразований здесь нет.

### Числовые преобразования
```csharp
// C# — неявные и явные преобразования
int small = 42;
long big = small;              // Неявное расширение: OK
double d = small;              // Неявное расширение: OK
int truncated = (int)3.14;     // Явное сужение: 3
byte b = (byte)300;            // Тихое переполнение: 44

// Безопасное преобразование
if (int.TryParse("42", out int parsed)) { /* ... */ }
```

```rust
// Rust — ВСЕ числовые преобразования явные
let small: i32 = 42;
let big: i64 = small as i64;       // Расширение: явно через 'as'
let d: f64 = small as f64;         // Целое в вещественное: явно
let truncated: i32 = 3.14_f64 as i32; // Сужение: 3 (отбрасывается дробная часть)
let b: u8 = 300_u16 as u8;        // Переполнение: оборачивается до 44 (как unchecked в C#)

// Безопасное преобразование через TryFrom
use std::convert::TryFrom;
let safe: Result<u8, _> = u8::try_from(300_u16); // Err — вне диапазона
let ok: Result<u8, _>   = u8::try_from(42_u16);  // Ok(42)

// Разбор строки — возвращает Result, а не bool + out-параметр
let parsed: Result<i32, _> = "42".parse::<i32>();   // Ok(42)
let bad: Result<i32, _>    = "abc".parse::<i32>();  // Err(ParseIntError)

// С синтаксисом turbofish:
let n = "42".parse::<f64>().unwrap(); // 42.0
```

### Преобразования строк
```csharp
// C#
int n = 42;
string s = n.ToString();          // "42"
string formatted = $"{n:X}";
int back = int.Parse(s);          // 42 или исключение
bool ok = int.TryParse(s, out int result);
```

```rust
// Rust — to_string() через Display, parse() через FromStr
let n: i32 = 42;
let s: String = n.to_string();            // "42" (использует трейт Display)
let formatted = format!("{n:X}");         // "2A"
let back: i32 = s.parse().unwrap();       // 42 или panic
let result: Result<i32, _> = s.parse();   // Ok(42) — безопасная версия

// Преобразования &str ↔ String (самое частое преобразование в Rust)
let owned: String = "hello".to_string();    // &str → String
let owned2: String = String::from("hello"); // &str → String (то же самое)
let borrowed: &str = &owned;               // String → &str (бесплатно, это просто заимствование)
```

### Преобразования ссылочных типов (никакого приведения по наследованию!)
```csharp
// C# — приведение вверх и вниз по иерархии
Animal a = new Dog();              // Приведение вверх (неявное)
Dog d = (Dog)a;                    // Приведение вниз (явное, может бросить исключение)
if (a is Dog dog) { /* ... */ }    // Безопасное приведение вниз через сопоставление с образцом
```

```rust
// Rust — нет наследования, нет приведения вверх и вниз
// Для полиморфизма используйте трейт-объекты:
let animal: Box<dyn Animal> = Box::new(Dog);

// «Приведение вниз» требует трейта Any (нужно редко):
use std::any::Any;
if let Some(dog) = animal_any.downcast_ref::<Dog>() {
    // Используем dog
}
// На практике вместо приведения вниз используйте перечисления:
enum Animal {
    Dog(Dog),
    Cat(Cat),
}
match animal {
    Animal::Dog(d) => { /* используем d */ }
    Animal::Cat(c) => { /* используем c */ }
}
```

### Краткая справка

| C# | Rust | Примечания |
|----|------|-------|
| `(int)x` | `x as i32` | Усечение/оборачивание при приведении |
| Неявное расширение | Нужно использовать `as` | Неявных числовых преобразований нет |
| `Convert.ToInt32(x)` | `i32::try_from(x)` | Безопасно, возвращает `Result` |
| `int.Parse(s)` | `s.parse::<i32>().unwrap()` | Паника при ошибке |
| `int.TryParse(s, out n)` | `s.parse::<i32>()` | Возвращает `Result<i32, _>` |
| `(Dog)animal` | Недоступно | Используйте перечисления или `Any` |
| `as Dog` / `is Dog` | `downcast_ref::<Dog>()` | Через трейт `Any`; предпочтительны перечисления |

***

## Комментарии и документация

### Обычные комментарии
```csharp
// Комментарии в C#
// Однострочный комментарий
/* Многострочный
   комментарий */

/// <summary>
/// XML-комментарий документации
/// </summary>
/// <param name="name">Имя пользователя</param>
/// <returns>Строка приветствия</returns>
public string Greet(string name)
{
    return $"Hello, {name}!";
}
```

```rust
// Комментарии в Rust
// Однострочный комментарий
/* Многострочный
   комментарий */

/// Комментарий документации (аналог C# ///)
/// Эта функция приветствует пользователя по имени.
/// 
/// # Аргументы
/// 
/// * `name` - Имя пользователя в виде среза строки
/// 
/// # Возвращает
/// 
/// `String`, содержащую приветствие
/// 
/// # Примеры
/// 
/// ```
/// let greeting = greet("Alice");
/// assert_eq!(greeting, "Hello, Alice!");
/// ```
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

### Генерация документации
```bash
# Генерация документации (аналог XML-документации в C#)
cargo doc --open

# Запуск тестов документации
cargo test --doc
```

---

## Упражнения

<details>
<summary><strong>🏋️ Упражнение: температура с безопасными типами</strong> (нажмите, чтобы раскрыть)</summary>

Напишите программу на Rust, которая:
1. Объявляет `const` для абсолютного нуля в градусах Цельсия (`-273.15`)
2. Объявляет `static`-счётчик количества выполненных преобразований (используйте `AtomicU32`)
3. Пишет функцию `celsius_to_fahrenheit(c: f64) -> f64`, которая отвергает температуры ниже абсолютного нуля, возвращая `f64::NAN`
4. Демонстрирует затенение: разбирает строку `"98.6"` в `f64`, а затем преобразует её

<details>
<summary>🔑 Решение</summary>

```rust
use std::sync::atomic::{AtomicU32, Ordering};

const ABSOLUTE_ZERO_C: f64 = -273.15;
static CONVERSION_COUNT: AtomicU32 = AtomicU32::new(0);

fn celsius_to_fahrenheit(c: f64) -> f64 {
    if c < ABSOLUTE_ZERO_C {
        return f64::NAN;
    }
    CONVERSION_COUNT.fetch_add(1, Ordering::Relaxed);
    c * 9.0 / 5.0 + 32.0
}

fn main() {
    let temp = "98.6";           // &str
    let temp: f64 = temp.parse().unwrap(); // затеняем как f64
    let temp = celsius_to_fahrenheit(temp); // затеняем как Фаренгейт
    println!("{temp:.1}°F");
    println!("Преобразований: {}", CONVERSION_COUNT.load(Ordering::Relaxed));
}
```

</details>
</details>

***

