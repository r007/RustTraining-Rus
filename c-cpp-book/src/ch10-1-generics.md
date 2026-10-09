# Обобщения в Rust

> **Что вы узнаете:** обобщённые параметры типов, мономорфизацию (обобщения без накладных расходов), ограничения трейтами и то, как обобщения Rust соотносятся с шаблонами C++ — с более понятными сообщениями об ошибках и без SFINAE.

- Обобщения позволяют повторно использовать один и тот же алгоритм или структуру данных для разных типов данных
    - Обобщённый параметр записывается как идентификатор в ```<>```, например: ```<T>```. Параметр может иметь любое допустимое имя, но обычно его делают коротким
    - Компилятор выполняет мономорфизацию на этапе компиляции, то есть генерирует новый тип для каждого варианта ```T```, который встречается в коде
```rust
// Возвращает кортеж типа <T>, составленный из left и right типа <T>
fn pick<T>(x: u32, left: T, right: T) -> (T, T) {
   if x == 42 {
    (left, right) 
   } else {
    (right, left)
   }
}
fn main() {
    let a = pick(42, true, false);
    let b = pick(42, "hello", "world");
    println!("{a:?}, {b:?}");
}
```

# Обобщения в Rust: специализация
- Обобщения можно применять и к типам данных, и к ассоциированным методам. Реализацию можно специализировать для конкретного ```<T>``` (например, ```f32``` и ```u32```)
```rust
#[derive(Debug)] // Мы обсудим это позже
struct Point<T> {
    x : T,
    y : T,
}
impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point {x, y}
    }
    fn set_x(&mut self, x: T) {
         self.x = x;       
    }
    fn set_y(&mut self, y: T) {
         self.y = y;       
    }
}
impl Point<f32> {
    fn is_secret(&self) -> bool {
        self.x == 42.0
    }    
}
fn main() {
    let mut p = Point::new(2, 4); // i32
    let q = Point::new(2.0, 4.0); // f32
    p.set_x(42);
    p.set_y(43);
    println!("{p:?} {q:?} {}", q.is_secret());
}
```

# Упражнение: обобщения

🟢 **Начальный уровень**
- Измените тип ```Point```, чтобы для x и y использовались два разных типа (```T``` и ```U```)

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
#[derive(Debug)]
struct Point<T, U> {
    x: T,
    y: U,
}

impl<T, U> Point<T, U> {
    fn new(x: T, y: U) -> Self {
        Point { x, y }
    }
}

fn main() {
    let p1 = Point::new(42, 3.14);        // Point<i32, f64>
    let p2 = Point::new("hello", true);   // Point<&str, bool>
    let p3 = Point::new(1u8, 1000u64);    // Point<u8, u64>
    println!("{p1:?}");
    println!("{p2:?}");
    println!("{p3:?}");
}
// Вывод:
// Point { x: 42, y: 3.14 }
// Point { x: "hello", y: true }
// Point { x: 1, y: 1000 }
```

</details>

### Комбинирование трейтов и обобщений в Rust
- Трейты можно использовать для наложения ограничений на обобщённые типы (constraints)
- Ограничение задаётся через ```:``` после обобщённого параметра типа или через ```where```. Ниже определена обобщённая функция ```get_area```, которая принимает любой тип ```T```, если он реализует трейт ```ComputeArea```
```rust
    trait ComputeArea {
        fn area(&self) -> u64;
    }
    fn get_area<T: ComputeArea>(t: &T) -> u64 {
        t.area()
    }
```
- [▶ Попробуйте в Rust Playground](https://play.rust-lang.org/)

### Комбинирование трейтов и обобщений в Rust: несколько ограничений
- Можно задать несколько ограничений трейтами
```rust
trait Fish {}
trait Mammal {}
struct Shark;
struct Whale;
impl Fish for Shark {}
impl Fish for Whale {}
impl Mammal for Whale {}
fn only_fish_and_mammals<T: Fish + Mammal>(_t: &T) {}
fn main() {
    let w = Whale {};
    only_fish_and_mammals(&w);
    let _s = Shark {};
    // Не скомпилируется
    only_fish_and_mammals(&_s);
}
```

### Ограничения трейтов в типах данных
- Ограничения трейтами можно комбинировать с обобщениями в типах данных
- В следующем примере мы определяем трейт ```PrintDescription``` и обобщённую структуру ```Shape``` с полем, ограниченным этим трейтом
```rust
trait PrintDescription {
    fn print_description(&self);
}
struct Shape<S: PrintDescription> {
    shape: S,
}
// Обобщённая реализация Shape для любого типа, который реализует PrintDescription
impl<S: PrintDescription> Shape<S> {
    fn print(&self) {
        self.shape.print_description();
    }
}
```
- [▶ Попробуйте в Rust Playground](https://play.rust-lang.org/)

# Упражнение: ограничения трейтов и обобщения

🟡 **Средний уровень**
- Реализуйте структуру с обобщённым полем ```cipher```, которое реализует ```CipherText```
```rust
trait CipherText {
    fn encrypt(&self);
}
// TO DO
//struct Cipher<>

```
- Затем реализуйте метод ```encrypt``` для структуры ```impl```, который вызывает ```encrypt``` у ```cipher```
```rust
// TO DO
impl for Cipher<> {}
```
- Далее реализуйте ```CipherText``` для двух структур, ```CipherOne``` и ```CipherTwo``` (достаточно ```println()```). Создайте ```CipherOne``` и ```CipherTwo``` и используйте ```Cipher``` для их вызова

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
trait CipherText {
    fn encrypt(&self);
}

struct Cipher<T: CipherText> {
    cipher: T,
}

impl<T: CipherText> Cipher<T> {
    fn encrypt(&self) {
        self.cipher.encrypt();
    }
}

struct CipherOne;
struct CipherTwo;

impl CipherText for CipherOne {
    fn encrypt(&self) {
        println!("CipherOne encryption applied");
    }
}

impl CipherText for CipherTwo {
    fn encrypt(&self) {
        println!("CipherTwo encryption applied");
    }
}

fn main() {
    let c1 = Cipher { cipher: CipherOne };
    let c2 = Cipher { cipher: CipherTwo };
    c1.encrypt();
    c2.encrypt();
}
// Вывод:
// CipherOne encryption applied
// CipherTwo encryption applied
```

</details>

### Паттерн состояний (type state) и обобщения в Rust
- Типы Rust можно использовать, чтобы обеспечить переходы конечного автомата на этапе *компиляции*
    - Рассмотрим ```Drone``` с двумя состояниями: ```Idle``` и ```Flying```. В состоянии ```Idle``` единственный допустимый метод — ```takeoff()```. В состоянии ```Flying``` разрешён ```land()```
    
- Один из подходов — смоделировать конечный автомат примерно так
```rust
enum DroneState {
    Idle,
    Flying
}
struct Drone {x: u64, y: u64, z: u64, state: DroneState}  // x, y, z — координаты
```
- Это требует множества проверок во время выполнения, чтобы обеспечить семантику конечного автомата — [▶ попробуйте](https://play.rust-lang.org/), чтобы увидеть почему

### Паттерн состояний на обобщениях
- Обобщения позволяют обеспечить конечный автомат на этапе *компиляции*. Для этого нужен специальный обобщённый тип ```PhantomData<T>```
- ```PhantomData<T>``` — это маркерный тип данных ```нулевого размера```. В данном случае мы используем его для представления состояний ```Idle``` и ```Flying```, но во время выполнения он занимает ```ноль``` байт
- Обратите внимание, что методы ```takeoff``` и ```land``` принимают ```self``` как параметр. Это называется ```поглощением``` (в отличие от ```&self```, которое использует заимствование). По сути, после вызова ```takeoff()``` для ```Drone<Idle>``` мы можем получить только ```Drone<Flying>``` и наоборот
```rust
struct Drone<T> {x: u64, y: u64, z: u64, state: PhantomData<T> }
impl Drone<Idle> {
    fn takeoff(self) -> Drone<Flying> {...}
}
impl Drone<Flying> {
    fn land(self) -> Drone<Idle> { ...}
}
```
    - [▶ Попробуйте в Rust Playground](https://play.rust-lang.org/)

### Паттерн состояний: ключевые выводы
- Основные выводы:
    - Состояния можно представить структурами (нулевого размера)
    - Состояние ```T``` можно комбинировать с ```PhantomData<T>``` (нулевого размера)
    - Реализация методов для конкретного этапа конечного автомата — это просто ```impl State<T>```
    - Для перехода из одного состояния в другое используйте метод, который поглощает ```self```
    - Это даёт абстракции ```без накладных расходов```. Компилятор может обеспечить конечный автомат на этапе компиляции, и невозможно вызвать методы, если состояние неподходящее

### Паттерн-строитель в Rust
- Поглощение ```self``` может быть полезным для паттерна-строителя
- Рассмотрим конфигурацию GPIO с несколькими десятками выводов. Каждый вывод может быть установлен в высокий или низкий уровень (по умолчанию — низкий)
```rust
#[derive(default)]
enum PinState {
    #[default]
    Low,
    High,
} 
#[derive(default)]
struct GPIOConfig {
    pin0: PinState,
    pin1: PinState
    ... 
}
```
- Паттерн-строитель можно использовать, чтобы собрать конфигурацию GPIO через цепочку вызовов — [▶ Попробуйте](https://play.rust-lang.org/)

