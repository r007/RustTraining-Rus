## Ключевые слова Rust для разработчиков C#

> **Что вы узнаете:** краткое соответствие ключевых слов Rust их аналогам в C# —
> модификаторы видимости, ключевые слова владения, управление потоком, определения типов и синтаксис сопоставления с образцом.
>
> **Сложность:** 🟢 Начальный

Понимание ключевых слов Rust и их назначения помогает разработчикам C# эффективнее осваивать язык.

### Ключевые слова видимости и контроля доступа

#### Модификаторы доступа C#
```csharp
public class Example
{
    public int PublicField;           // Доступно везде
    private int privateField;        // Только внутри этого класса
    protected int protectedField;    // Этот класс и его наследники
    internal int internalField;      // Внутри этой сборки
    protected internal int protectedInternalField; // Комбинация
}
```

#### Ключевые слова видимости Rust
```rust
// pub - делает элементы публичными (как public в C#)
pub struct PublicStruct {
    pub public_field: i32,           // Публичное поле
    private_field: i32,              // По умолчанию приватное (без ключевого слова)
}

pub mod my_module {
    pub(crate) fn crate_public() {}     // Публично внутри текущего крейта (как internal)
    pub(super) fn parent_public() {}    // Публично для родительского модуля
    pub(self) fn self_public() {}       // Публично внутри текущего модуля (то же, что и приватное)
    
    pub use super::PublicStruct;        // Реэкспорт (как псевдоним через using)
}

// Прямого аналога protected в C# нет — используйте композицию
```

### Ключевые слова памяти и владения

#### Ключевые слова памяти C#
```csharp
// ref - передача по ссылке
public void Method(ref int value) { value = 10; }

// out - выходной параметр
public bool TryParse(string input, out int result) { /* */ }

// in - ссылка только для чтения (C# 7.2+)
public void ReadOnly(in LargeStruct data) { /* Нельзя изменить data */ }
```

#### Ключевые слова владения Rust
```rust
// & - неизменяемая ссылка (как in-параметр в C#)
fn read_only(data: &Vec<i32>) {
    println!("Длина: {}", data.len()); // Можно читать, нельзя изменять
}

// &mut - изменяемая ссылка (как ref-параметр в C#)
fn modify(data: &mut Vec<i32>) {
    data.push(42); // Можно изменять
}

// move - принудительный захват перемещением в замыканиях
let data = vec![1, 2, 3];
let closure = move || {
    println!("{:?}", data); // data перемещается в замыкание
};
// data здесь больше недоступен

// Box - выделение в куче (как new для ссылочных типов в C#)
let boxed_data = Box::new(42); // Выделяем в куче
```

### Ключевые слова управления потоком

#### Управление потоком C#
```csharp
// return - выход из функции со значением
public int GetValue() { return 42; }

// yield return - паттерн итератора
public IEnumerable<int> GetNumbers()
{
    yield return 1;
    yield return 2;
}

// break/continue - управление циклом
foreach (var item in items)
{
    if (item == null) continue;
    if (item.Stop) break;
}
```

#### Ключевые слова управления потоком Rust
```rust
// return - явный возврат (обычно не нужен)
fn get_value() -> i32 {
    return 42; // Явный возврат
    // ИЛИ просто: 42 (неявный возврат)
}

// break/continue - управление циклом с необязательными значениями
fn find_value() -> Option<i32> {
    loop {
        let value = get_next();
        if value < 0 { continue; }
        if value > 100 { break None; }      // Выход с значением
        if value == 42 { break Some(value); } // Выход с успешным результатом
    }
}

// loop - бесконечный цикл (как while(true))
loop {
    if condition { break; }
}

// while - условный цикл
while condition {
    // код
}

// for - цикл по итератору
for item in collection {
    // код
}
```

### Ключевые слова определения типов

#### Ключевые слова типов C#
```csharp
// class - ссылочный тип
public class MyClass { }

// struct - значимый тип
public struct MyStruct { }

// interface - определение контракта
public interface IMyInterface { }

// enum - перечисление
public enum MyEnum { Value1, Value2 }

// delegate - указатель на функцию
public delegate void MyDelegate(int value);
```

#### Ключевые слова типов Rust
```rust
// struct - структура данных (объединяет class и struct из C#)
struct MyStruct {
    field: i32,
}

// enum - алгебраический тип данных (гораздо мощнее enum из C#)
enum MyEnum {
    Variant1,
    Variant2(i32),              // Может хранить данные
    Variant3 { x: i32, y: i32 }, // Вариант в виде структуры
}

// trait - определение интерфейса (как interface в C#, но мощнее)
trait MyTrait {
    fn method(&self);
    
    // Реализация по умолчанию (как default-методы интерфейсов в C# 8+)
    fn default_method(&self) {
        println!("Реализация по умолчанию");
    }
}

// type - псевдоним типа (как псевдоним using в C#)
type UserId = u32;
type Result<T> = std::result::Result<T, MyError>;

// impl - блок реализации (аналога в C# нет — методы определяются отдельно)
impl MyStruct {
    fn new() -> MyStruct {
        MyStruct { field: 0 }
    }
}

impl MyTrait for MyStruct {
    fn method(&self) {
        println!("Реализация");
    }
}
```

### Ключевые слова определения функций

#### Ключевые слова функций C#
```csharp
// static - метод класса
public static void StaticMethod() { }

// virtual - может быть переопределён
public virtual void VirtualMethod() { }

// override - переопределение базового метода
public override void VirtualMethod() { }

// abstract - обязан быть реализован
public abstract void AbstractMethod();

// async - асинхронный метод
public async Task<int> AsyncMethod() { return await SomeTask(); }
```

#### Ключевые слова функций Rust
```rust
// fn - определение функции (как метод в C#, но существует самостоятельно)
fn regular_function() {
    println!("Привет");
}

// const fn - функция времени компиляции (как const в C#, но для функций)
const fn compile_time_function() -> i32 {
    42 // Может быть вычислена на этапе компиляции
}

// async fn - асинхронная функция (как async в C#)
async fn async_function() -> i32 {
    some_async_operation().await
}

// unsafe fn - функция, которая может нарушить безопасность памяти
unsafe fn unsafe_function() {
    // Может выполнять небезопасные операции
}

// extern fn - интерфейс внешних функций (FFI)
extern "C" fn c_compatible_function() {
    // Может вызываться из C
}
```

### Ключевые слова объявления переменных

#### Ключевые слова переменных C#
```csharp
// var - вывод типа
var name = "John"; // Выводится как string

// const - константа времени компиляции
const int MaxSize = 100;

// readonly - константа времени выполнения (только для полей, не для локальных переменных)
// readonly DateTime createdAt = DateTime.Now;

// static - переменная уровня класса
static int instanceCount = 0;
```

#### Ключевые слова переменных Rust
```rust
// let - привязка переменной (как var в C#)
let name = "John"; // По умолчанию неизменяемая

// let mut - изменяемая привязка переменной
let mut count = 0; // Может быть изменена
count += 1;

// const - константа времени компиляции (как const в C#)
const MAX_SIZE: usize = 100;

// static - глобальная переменная (как static в C#)
static INSTANCE_COUNT: std::sync::atomic::AtomicUsize = 
    std::sync::atomic::AtomicUsize::new(0);
```

### Ключевые слова сопоставления с образцом

#### Сопоставление с образцом C# (C# 8+)
```csharp
// switch-выражение
string result = value switch
{
    1 => "Один",
    2 => "Два",
    _ => "Другое"
};

// паттерн is
if (obj is string str)
{
    Console.WriteLine(str.Length);
}
```

#### Ключевые слова сопоставления с образцом Rust
```rust
// match - сопоставление с образцом (как switch в C#, но гораздо мощнее)
let result = match value {
    1 => "Один",
    2 => "Два",
    3..=10 => "От 3 до 10", // Диапазонные паттерны
    _ => "Другое", // Подстановочный паттерн (как _ в C#)
};

// if let - условное сопоставление с образцом
if let Some(value) = optional {
    println!("Получено значение: {}", value);
}

// while let - цикл с сопоставлением с образцом
while let Some(item) = iterator.next() {
    println!("Элемент: {}", item);
}

// let с паттернами - деструктуризация
let (x, y) = point; // Деструктуризация кортежа
let Some(value) = optional else {
    return; // Ранний выход, если паттерн не совпал
};
```

### Ключевые слова безопасности памяти

#### Ключевые слова памяти C#
```csharp
// unsafe - отключение проверок безопасности
unsafe
{
    int* ptr = &variable;
    *ptr = 42;
}

// fixed - закрепление управляемой памяти
unsafe
{
    fixed (byte* ptr = array)
    {
        // Использовать ptr
    }
}
```

#### Ключевые слова безопасности Rust
```rust
// unsafe - отключение проверок заимствований (используйте осторожно!)
unsafe {
    let ptr = &variable as *const i32;
    let value = *ptr; // Разыменование сырого указателя
}

// Типы сырых указателей (аналога в C# нет — обычно не нужны)
let ptr: *const i32 = &42;  // Неизменяемый сырой указатель
let ptr: *mut i32 = &mut 42; // Изменяемый сырой указатель
```

### Распространённые ключевые слова Rust, которых нет в C#

```rust
// where - ограничения обобщений (гибче, чем where в C#)
fn generic_function<T>() 
where 
    T: Clone + Send + Sync,
{
    // T должен реализовывать трейты Clone, Send и Sync
}

// dyn - динамические трейт-объекты (как object в C#, но с проверкой типов)
let drawable: Box<dyn Draw> = Box::new(Circle::new());

// Self - ссылка на реализующий тип (как this в C#, но для типов)
impl MyStruct {
    fn new() -> Self { // Self = MyStruct
        Self { field: 0 }
    }
}

// self - получатель метода
impl MyStruct {
    fn method(&self) { }        // Неизменяемое заимствование
    fn method_mut(&mut self) { } // Изменяемое заимствование
    fn consume(self) { }        // Принять владение
}

// crate - ссылка на корень текущего крейта
use crate::models::User; // Абсолютный путь от корня крейта

// super - ссылка на родительский модуль
use super::utils; // Импорт из родительского модуля
```

### Сводка ключевых слов для разработчиков C#

| Назначение | C# | Rust | Ключевое отличие |
|---------|----|----|----------------|
| Видимость | `public`, `private`, `internal` | `pub`, по умолчанию приватное | Более тонкая настройка через `pub(crate)` |
| Переменные | `var`, `readonly`, `const` | `let`, `let mut`, `const` | Неизменяемость по умолчанию |
| Функции | `method()` | `fn` | Функции могут существовать самостоятельно |
| Типы | `class`, `struct`, `interface` | `struct`, `enum`, `trait` | Перечисления — алгебраические типы |
| Обобщения | `<T> where T : IFoo` | `<T> where T: Foo` | Более гибкие ограничения |
| Ссылки | `ref`, `out`, `in` | `&`, `&mut` | Проверка заимствований на этапе компиляции |
| Паттерны | `switch`, `is` | `match`, `if let` | Требуется исчерпывающее сопоставление |

***

