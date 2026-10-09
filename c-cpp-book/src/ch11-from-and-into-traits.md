# Трейты From и Into в Rust

> **Что вы узнаете:** трейты преобразования типов в Rust — `From<T>` и `Into<T>` для преобразований, которые не могут завершиться ошибкой, и `TryFrom` и `TryInto` для преобразований, которые могут. Реализуйте `From` и получите `Into` бесплатно. Заменяют операторы преобразования и конструкторы C++.

- ```From``` и ```Into``` — взаимодополняющие трейты для преобразования типов
- Обычно типы реализуют трейт ```From```. ```String::from("Rust")``` преобразует ```&str``` в ```String```.
Когда существует ```From<T> for U```, Rust также предоставляет ```Into<U> for T```, поэтому ```let s: String = "Rust".into();``` тоже работает.
```rust
struct Point {x: u32, y: u32}
// Создаём Point из кортежа
impl From<(u32, u32)> for Point {
    fn from(xy : (u32, u32)) -> Self {
        Point {x : xy.0, y: xy.1}       // Создаём Point из элементов кортежа
    }
}
fn main() {
    let s = String::from("Rust");
    let x = u32::from(true);
    let p = Point::from((40, 42));
    // let p : Point = (40,42).into(); // Альтернативная запись выражения выше
    println!("s: {s} x:{x} p.x:{} p.y {}", p.x, p.y);   
}
```

# Упражнение: From и Into
- Реализуйте трейт ```From``` для ```Point``` с преобразованием в тип ```TransposePoint```. ```TransposePoint``` меняет местами элементы ```x``` и ```y``` у ```Point```

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
struct Point { x: u32, y: u32 }
struct TransposePoint { x: u32, y: u32 }

impl From<Point> for TransposePoint {
    fn from(p: Point) -> Self {
        TransposePoint { x: p.y, y: p.x }
    }
}

fn main() {
    let p = Point { x: 10, y: 20 };
    let tp = TransposePoint::from(p);
    println!("Transposed: x={}, y={}", tp.x, tp.y);  // x=20, y=10

    // Через .into() — работает автоматически, когда реализован From
    let p2 = Point { x: 3, y: 7 };
    let tp2: TransposePoint = p2.into();
    println!("Transposed: x={}, y={}", tp2.x, tp2.y);  // x=7, y=3
}
// Вывод:
// Transposed: x=20, y=10
// Transposed: x=7, y=3
```

</details>

# Трейт Default в Rust
- ```Default``` можно использовать, чтобы задать значения по умолчанию для типа
    - Типы могут использовать макрос ```Derive``` с ```Default``` или предоставить собственную реализацию
```rust
#[derive(Default, Debug)]
struct Point {x: u32, y: u32}
#[derive(Debug)]
struct CustomPoint {x: u32, y: u32}
impl Default for CustomPoint {
    fn default() -> Self {
        CustomPoint {x: 42, y: 42}
    }
}
fn main() {
    let x = Point::default();   // Создаёт Point{0, 0}
    println!("{x:?}");
    let y = CustomPoint::default();
    println!("{y:?}");
}
```

### Применение трейта Default
- Трейт ```Default``` применяется в нескольких случаях, в том числе
    - Частичное копирование с инициализацией остальных полей значениями по умолчанию
    - Значение по умолчанию для типов ```Option``` в методах вроде ```unwrap_or_default()```
```rust
#[derive(Debug)]
struct CustomPoint {x: u32, y: u32}
impl Default for CustomPoint {
    fn default() -> Self {
        CustomPoint {x: 42, y: 42}
    }
}
fn main() {
    let x = CustomPoint::default();
    // Переопределяем y, а остальные поля оставляем по умолчанию
    let y = CustomPoint {y: 43, ..CustomPoint::default()};
    println!("{x:?} {y:?}");
    let z : Option<CustomPoint> = None;
    // Попробуйте заменить unwrap_or_default() на unwrap()
    println!("{:?}", z.unwrap_or_default());
}
```

### Другие преобразования типов в Rust
- Rust не поддерживает неявных преобразований типов, а для ```явных``` преобразований используется ```as```
- ```as``` следует использовать с осторожностью, потому что он может приводить к потере данных при сужении и т. п. В общем случае предпочтительнее использовать ```into()``` или ```from()```, где это возможно
```rust
fn main() {
    let f = 42u8;
    // let g : u32 = f;    // Не скомпилируется
    let g = f as u32;      // Работает, но не рекомендуется. Подчиняется правилам сужения
    let g : u32 = f.into(); // Предпочтительная форма; не может завершиться ошибкой, проверяется компилятором
    // let k : u8 = g.into();  // Не скомпилируется; сужение может привести к потере данных
    
    // Для операции сужения нужно использовать try_into
    if let Ok(k) = TryInto::<u8>::try_into(g) {
        println!("{k}");
    }
}
```

