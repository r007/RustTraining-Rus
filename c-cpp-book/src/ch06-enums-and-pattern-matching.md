# Перечисления в Rust

> **Что вы узнаете:** перечисления Rust как размеченные объединения (tagged unions, сделанные правильно), `match` для исчерпывающего сопоставления с образцом и то, как перечисления заменяют иерархии классов C++ и размеченные объединения C с безопасностью, которую обеспечивает компилятор.

- Перечисления в Rust — это размеченные объединения, то есть суммовые типы из нескольких возможных различных типов с меткой, которая определяет конкретный вариант
    - Для разработчиков на C: перечисления в Rust могут содержать данные (размеченные объединения, сделанные правильно — компилятор отслеживает, какой вариант активен)
    - Для разработчиков на C++: перечисления Rust похожи на `std::variant`, но с исчерпывающим сопоставлением с образцом, без исключений `std::get` и без шаблонного кода `std::visit`
    - Размер `enum` равен размеру самого большого возможного варианта. Отдельные варианты не связаны друг с другом и могут иметь совершенно разные типы
    - Типы `enum` — одна из самых мощных возможностей языка: они заменяют целые иерархии классов в C++ (подробнее об этом в разборах кейсов)
```rust
fn main() {
    enum Numbers {
        Zero,
        SmallNumber(u8),
        BiggerNumber(u32),
        EvenBiggerNumber(u64),
    }
    let a = Numbers::Zero;
    let b = Numbers::SmallNumber(42);
    let c : Numbers = a; // Ок — тип a это Numbers
    let d : Numbers = b; // Ок — тип b это Numbers
}
```
----
# Оператор match в Rust
- ```match``` в Rust — это аналог C-шного "switch", но на стероидах
    - ```match``` можно использовать для сопоставления с образцом простых типов данных, ```struct```, ```enum```
    - Оператор ```match``` должен быть исчерпывающим, то есть покрывать все возможные случаи для данного ```типа```. Символ ```_``` используется как подстановочный шаблон для случая «все остальные»
    - ```match``` может возвращать значение, но все ветки (```=>```) должны возвращать значение одного и того же типа

```rust
fn main() {
    let x = 42;
    // В этом случае _ покрывает все числа, кроме явно перечисленных
    let is_secret_of_life = match x {
        42 => true, // Возвращаемый тип — булево значение
        _ => false, // Возвращаемый тип — булево значение
        // Это не скомпилируется, потому что возвращаемый тип не булев
        // _ => 0  
    };
    println!("{is_secret_of_life}");
}
```

# Оператор match в Rust
- ```match``` поддерживает диапазоны, булевы фильтры и охранные условия ```if```
```rust
fn main() {
    let x = 42;
    match x {
        // Обратите внимание: =41 обеспечивает включение верхней границы диапазона
        0..=41 => println!("Less than the secret of life"),
        42 => println!("Secret of life"),
        _ => println!("More than the secret of life"),
    }
    let y = 100;
    match y {
        100 if x == 43 => println!("y is 100% not secret of life"),
        100 if x == 42 => println!("y is 100% secret of life"),
        _ => (),    // Ничего не делать
    }
}
```

# Оператор match в Rust
- ```match``` и ```enum``` часто используются вместе
    - Оператор match может «связать» содержащееся значение с переменной. Используйте ```_```, если значение не важно
    - Макрос ```matches!``` можно использовать для проверки на конкретный вариант
```rust
fn main() {
    enum Numbers {
        Zero,
        SmallNumber(u8),
        BiggerNumber(u32),
        EvenBiggerNumber(u64),
    }
    let b = Numbers::SmallNumber(42);
    match b {
        Numbers::Zero => println!("Zero"),
        Numbers::SmallNumber(value) => println!("Small number {value}"),
        Numbers::BiggerNumber(_) | Numbers::EvenBiggerNumber(_) => println!("Some BiggerNumber or EvenBiggerNumber"),
    }
    
    // Булева проверка на конкретные варианты
    if matches!(b, Numbers::Zero | Numbers::SmallNumber(_)) {
        println!("Matched Zero or small number");
    }
}
```

# Оператор match в Rust
- ```match``` также может выполнять сопоставление с помощью деструктуризации и срезов
```rust
fn main() {
    struct Foo {
        x: (u32, bool),
        y: u32
    }
    let f = Foo {x: (42, true), y: 100};
    match f {
        // Захватываем значение x в переменную tuple
        Foo{y: 100, x : tuple} => println!("Matched x: {tuple:?}"),
        _ => ()
    }
    let a = [40, 41, 42];
    match a {
        // Последний элемент среза должен быть 42. @ используется для связывания с образцом
        [rest @ .., 42] => println!("{rest:?}"),
        // Первый элемент среза должен быть 42. @ используется для связывания с образцом
        [42, rest @ ..] => println!("{rest:?}"),
        _ => (),
    }
}
```

# Упражнение: сложение и вычитание с помощью match и enum

🟢 **Начальный уровень**

- Напишите функцию, которая реализует арифметические операции над беззнаковыми 64-битными числами
- **Шаг 1**: определите перечисление для операций:
```rust
enum Operation {
    Add(u64, u64),
    Subtract(u64, u64),
}
```
- **Шаг 2**: определите перечисление результата:
```rust
enum CalcResult {
    Ok(u64),                    // Успешный результат
    Invalid(String),            // Сообщение об ошибке для недопустимых операций
}
```
- **Шаг 3**: реализуйте `calculate(op: Operation) -> CalcResult`
    - Для Add: вернуть Ok(sum)
    - Для Subtract: вернуть Ok(difference), если первое число >= второго, иначе Invalid("Underflow")
- **Подсказка**: используйте сопоставление с образцом в функции:
```rust
match op {
    Operation::Add(a, b) => { /* ваш код */ },
    Operation::Subtract(a, b) => { /* ваш код */ },
}
```

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
enum Operation {
    Add(u64, u64),
    Subtract(u64, u64),
}

enum CalcResult {
    Ok(u64),
    Invalid(String),
}

fn calculate(op: Operation) -> CalcResult {
    match op {
        Operation::Add(a, b) => CalcResult::Ok(a + b),
        Operation::Subtract(a, b) => {
            if a >= b {
                CalcResult::Ok(a - b)
            } else {
                CalcResult::Invalid("Underflow".to_string())
            }
        }
    }
}

fn main() {
    match calculate(Operation::Add(10, 20)) {
        CalcResult::Ok(result) => println!("10 + 20 = {result}"),
        CalcResult::Invalid(msg) => println!("Error: {msg}"),
    }
    match calculate(Operation::Subtract(5, 10)) {
        CalcResult::Ok(result) => println!("5 - 10 = {result}"),
        CalcResult::Invalid(msg) => println!("Error: {msg}"),
    }
}
// Вывод:
// 10 + 20 = 30
// Error: Underflow
```

</details>

# Ассоциированные методы в Rust
- ```impl``` может определять методы, ассоциированные с типами вроде ```struct```, ```enum``` и т. д.
    - Методы могут необязательно принимать ```self``` как параметр. ```self``` концептуально похож на передачу указателя на структуру первым параметром в C или на ```this``` в C++
    - Ссылка на ```self``` может быть неизменяемой (по умолчанию: ```&self```), изменяемой (```&mut self```) или ```self``` (передача владения)
    - Ключевое слово ```Self``` можно использовать как сокращение, подразумевающее тип
```rust
struct Point {x: u32, y: u32}
impl Point {
    fn new(x: u32, y: u32) -> Self {
        Point {x, y}
    }
    fn increment_x(&mut self) {
        self.x += 1;
    }
}
fn main() {
    let mut p = Point::new(10, 20);
    p.increment_x();
}
```

# Упражнение: сложение и преобразование точки

🟡 **Средний уровень** — требует понимания разницы между перемещением и заимствованием по сигнатурам методов
- Реализуйте следующие ассоциированные методы для ```Point```
    - ```add()``` принимает другую ```Point``` и увеличивает значения x и y на месте (подсказка: используйте ```&mut self```)
    - ```transform()``` поглощает существующую ```Point``` (подсказка: используйте ```self```) и возвращает новую ```Point```, возводя x и y в квадрат

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
struct Point { x: u32, y: u32 }

impl Point {
    fn new(x: u32, y: u32) -> Self {
        Point { x, y }
    }
    fn add(&mut self, other: &Point) {
        self.x += other.x;
        self.y += other.y;
    }
    fn transform(self) -> Point {
        Point { x: self.x * self.x, y: self.y * self.y }
    }
}

fn main() {
    let mut p1 = Point::new(2, 3);
    let p2 = Point::new(10, 20);
    p1.add(&p2);
    println!("After add: x={}, y={}", p1.x, p1.y);           // x=12, y=23
    let p3 = p1.transform();
    println!("After transform: x={}, y={}", p3.x, p3.y);     // x=144, y=529
    // p1 больше недоступен — transform() его поглотил
}
```

</details>

----
