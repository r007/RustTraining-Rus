### Небезопасный Rust (unsafe)

> **Что вы узнаете:** когда и как использовать `unsafe` — разыменование сырых указателей, FFI (Foreign Function Interface) для вызова C из Rust и наоборот, `CString`/`CStr` для взаимодействия со строками и как писать безопасные обёртки вокруг небезопасного кода.

- ```unsafe``` открывает доступ к возможностям, которые обычно запрещены компилятором Rust
    - Разыменование сырых указателей
    - Доступ к *изменяемым* статическим переменным
    - https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
- С большой силой приходит большая ответственность
    - ```unsafe``` говорит компилятору: «Я, программист, беру на себя ответственность за соблюдение инвариантов, которые компилятор обычно гарантирует»
    - Нужно гарантировать отсутствие одновременных изменяемых и неизменяемых псевдонимов ссылок, висячих указателей, некорректных ссылок и т. д.
    - Использование ```unsafe``` следует ограничивать как можно меньшей областью
    - У всего кода с ```unsafe``` должен быть комментарий «safety», описывающий предположения

### Примеры unsafe в Rust
```rust
unsafe fn harmless() {}
fn main() {
    // Safety: вызываем безобидную небезопасную функцию
    unsafe {
        harmless();
    }
    let a = 42u32;
    let p = &a as *const u32;
    // Safety: p — корректный указатель на переменную, которая останется в области видимости
    unsafe {
        println!("{}", *p);
    }
    // Safety: небезопасно; только для иллюстрации
    let dangerous_buffer = 0xb8000 as *mut u32;
    unsafe {
        println!("About to go kaboom!!!");
        *dangerous_buffer = 0; // На большинстве современных машин это вызовет SEGV
    }
}
```

### Простой пример FFI (функция Rust, вызываемая из C)

## Строки FFI: CString и CStr

FFI расшифровывается как *Foreign Function Interface* (интерфейс внешних функций) — механизм, с помощью которого Rust вызывает функции, написанные на других языках (например, C), и наоборот.

При взаимодействии с кодом на C типы `String` и `&str` в Rust (которые являются UTF-8 без завершающего нуля) напрямую несовместимы с C-строками (которые представляют собой массивы байтов с завершающим нулём). Для этого в `std::ffi` Rust предоставляет `CString` (владеющий) и `CStr` (заимствованный):

| Тип | Аналог | Когда использовать |
|------|-------------|----------|
| `CString` | `String` (владеющий) | Создание C-строки из данных Rust |
| `&CStr` | `&str` (заимствованный) | Получение C-строки из внешнего кода |

```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

fn demo_ffi_strings() {
    // Создаём совместимую с C строку (добавляется завершающий нуль)
    let c_string = CString::new("Hello from Rust").expect("CString::new failed");
    let ptr: *const c_char = c_string.as_ptr();

    // Преобразуем C-строку обратно в Rust (unsafe, потому что мы доверяем указателю)
    // Safety: ptr валиден и заканчивается нулём (мы только что его создали)
    let back_to_rust: &CStr = unsafe { CStr::from_ptr(ptr) };
    let rust_str: &str = back_to_rust.to_str().expect("Invalid UTF-8");
    println!("{}", rust_str);
}
```

> **Предупреждение**: `CString::new()` вернёт ошибку, если входные данные содержат внутренние нулевые байты (`\0`). Всегда обрабатывайте `Result`. `CStr` активно используется в примерах FFI ниже.

- Методы ```FFI``` должны быть помечены ```#[no_mangle]```, чтобы компилятор не искажал их имена
- Скомпилируем крейт как статическую библиотеку
    ```
    #[no_mangle] 
    pub extern "C" fn add(left: u64, right: u64) -> u64 {
        left + right
    }
    ```
- Скомпилируем следующий C-код и скомпонуем его с нашей статической библиотекой
    ```
    #include <stdio.h>
    #include <stdint.h>
    extern uint64_t add(uint64_t, uint64_t);
    int main() {
        printf("Add returned %llu\n", add(21, 21));
    }
    ``` 

### Сложный пример FFI
- В следующих примерах мы создадим интерфейс логирования на Rust и предоставим его для [PYTHON] и ```C```
    - Мы увидим, как один и тот же интерфейс можно использовать нативно из Rust и из C
    - Мы рассмотрим инструменты вроде ```cbindgen`` для генерации заголовочных файлов для ```C```
    - Мы увидим, как обёртки ```unsafe``` служат мостом к безопасному коду на Rust

## Вспомогательные функции логгера
```rust
fn create_or_open_log_file(log_file: &str, overwrite: bool) -> Result<File, String> {
    if overwrite {
        File::create(log_file).map_err(|e| e.to_string())
    } else {
        OpenOptions::new()
            .write(true)
            .append(true)
            .open(log_file)
            .map_err(|e| e.to_string())
    }
}

fn log_to_file(file_handle: &mut File, message: &str) -> Result<(), String> {
    file_handle
        .write_all(message.as_bytes())
        .map_err(|e| e.to_string())
}
```

## Структура логгера
```rust
struct SimpleLogger {
    log_level: LogLevel,
    file_handle: File,
}

impl SimpleLogger {
    fn new(log_file: &str, overwrite: bool, log_level: LogLevel) -> Result<Self, String> {
        let file_handle = create_or_open_log_file(log_file, overwrite)?;
        Ok(Self {
            file_handle,
            log_level,
        })
    }

    fn log_message(&mut self, log_level: LogLevel, message: &str) -> Result<(), String> {
        if log_level as u32 <= self.log_level as u32 {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let message = format!("Simple: {timestamp} {log_level} {message}\n");
            log_to_file(&mut self.file_handle, &message)
        } else {
            Ok(())
        }
    }
}
```

## Тестирование
- Тестирование в Rust — это просто
    - Тестовые функции помечаются ```#[test]``` и не входят в скомпилированный бинарный файл
    - Легко создавать мок-методы для тестов
```rust
#[test]
fn testfunc() -> Result<(), String> {
    let mut logger = SimpleLogger::new("test.log", false, LogLevel::INFO)?;
    logger.log_message(LogLevel::TRACELEVEL1, "Hello world")?;
    logger.log_message(LogLevel::CRITICAL, "Critical message")?;
    Ok(()) // Компилятор автоматически уничтожает logger здесь
}
```
```bash
cargo test
```

## FFI: вызов из C в Rust
- cbindgen — отличный инструмент для генерации заголовочных файлов для экспортируемых функций Rust
    - Устанавливается через cargo
```bash
cargo install cbindgen
cbindgen 
```
- Функции и структуры экспортируются с помощью ```#[no_mangle]``` и ```#[repr(C)]```
    - Мы будем использовать распространённый шаблон интерфейса: передаём `**` в реализацию и возвращаем 0 при успехе и ненулевое значение при ошибке
    - **Непрозрачные и прозрачные структуры**: наш `SimpleLogger` передаётся как *непрозрачный указатель* (`*mut SimpleLogger`) — сторона C никогда не обращается к его полям, поэтому `#[repr(C)]` **не** нужен. Используйте `#[repr(C)]`, когда код на C должен напрямую читать и записывать поля структуры:

```rust
// Непрозрачная — C хранит только указатель и никогда не проверяет поля. #[repr(C)] не нужен.
struct SimpleLogger { /* Поля только для Rust */ }

// Прозрачная — C читает и записывает поля напрямую. ОБЯЗАТЕЛЬНО #[repr(C)].
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```
```c
typedef struct SimpleLogger SimpleLogger;
uint32_t create_simple_logger(const char *file_name, struct SimpleLogger **out_logger);
uint32_t log_entry(struct SimpleLogger *logger, const char *message);
uint32_t drop_logger(struct SimpleLogger *logger);
```

- Обратите внимание, что нам нужно много проверок на корректность
- Мы должны явно «утечь» память, чтобы Rust не освободил её автоматически
```rust
#[no_mangle] 
pub extern "C" fn create_simple_logger(file_name: *const std::os::raw::c_char, out_logger: *mut *mut SimpleLogger) -> u32 {
    use std::ffi::CStr;
    // Убеждаемся, что указатель не NULL
    if file_name.is_null() || out_logger.is_null() {
        return 1;
    }
    // Safety: переданный указатель либо NULL, либо по контракту заканчивается нулём
    let file_name = unsafe {
        CStr::from_ptr(file_name)
    };
    let file_name = file_name.to_str();
    // Убеждаемся, что в file_name нет мусорных символов
    if file_name.is_err() {
        return 1;
    }
    let file_name = file_name.unwrap();
    // Задаём значения по умолчанию; в реальности их следовало бы передавать
    let new_logger = SimpleLogger::new(file_name, true, LogLevel::CRITICAL);
    // Проверяем, что логгер удалось создать
    if new_logger.is_err() {
        return 1;
    }
    let new_logger = Box::new(new_logger.unwrap());
    // Это не даёт Box уничтожиться при выходе из области видимости
    let logger_ptr: *mut SimpleLogger = Box::leak(new_logger);
    // Safety: logger не NULL, а logger_ptr валиден
    unsafe {
        *out_logger = logger_ptr;
    }
    return 0;
}
```

- Аналогичные проверки ошибок есть и в ```log_entry()```
```rust
#[no_mangle]
pub extern "C" fn log_entry(logger: *mut SimpleLogger, message: *const std::os::raw::c_char) -> u32 {
    use std::ffi::CStr;
    if message.is_null() || logger.is_null() {
        return 1;
    }
    // Safety: message не NULL
    let message = unsafe {
        CStr::from_ptr(message)
    };
    let message = message.to_str();
    // Убеждаемся, что в сообщении нет мусорных символов
    if message.is_err() {
        return 1;
    }
    // Safety: logger — валидный указатель, ранее созданный create_simple_logger()
    unsafe {
        (*logger).log_message(LogLevel::CRITICAL, message.unwrap()).is_err() as u32
    }
}

#[no_mangle]
pub extern "C" fn drop_logger(logger: *mut SimpleLogger) -> u32 {
    if logger.is_null() {
        return 1;
    }
    // Safety: logger — валидный указатель, ранее созданный create_simple_logger()
    unsafe {
        // Это создаёт Box<SimpleLogger>, который уничтожается при выходе из области видимости
        let _ = Box::from_raw(logger);
    }
    0
}
```

- Тестировать наш FFI (C) можно средствами Rust или написав программу на C
```rust
#[test]
fn test_c_logger() {
    // Конструкция c".." создаёт строку с завершающим нулём
    let file_name = c"test.log".as_ptr() as *const std::os::raw::c_char;
    let mut c_logger: *mut SimpleLogger = std::ptr::null_mut();
    assert_eq!(create_simple_logger(file_name, &mut c_logger), 0);
    // Ручной способ создать строку c"..."
    let message = b"message from C\0".as_ptr() as *const std::os::raw::c_char;
    assert_eq!(log_entry(c_logger, message), 0);
    drop_logger(c_logger);
}
```
```c
#include "logger.h"
...
int main() {
    SimpleLogger *logger = NULL;
    if (create_simple_logger("test.log", &logger) == 0) {
        log_entry(logger, "Hello from C");
        drop_logger(logger); /* Необходимо для закрытия дескриптора и т. д. */
    } 
    ...
}
```

## Обеспечение корректности unsafe-кода
- Если коротко: использование ```unsafe``` требует осознанных размышлений
    - Всегда документируйте предположения о безопасности, которые делает код, и проверяйте его с экспертами
    - Используйте инструменты вроде cbindgen, Miri, Valgrind, которые помогают проверять корректность
    - **Никогда не позволяйте panic раскручиваться через границу FFI** — это неопределённое поведение. Используйте `std::panic::catch_unwind` в точках входа FFI или настройте `panic = "abort"` в профиле
    - Если структура используется через FFI, пометьте её `#[repr(C)]`, чтобы гарантировать совместимую с C раскладку в памяти
    - Обратитесь к https://doc.rust-lang.org/nomicon/intro.html («Rustonomicon» — тёмные искусства unsafe Rust)
    - Обращайтесь за помощью к внутренним экспертам

### Инструменты проверки: Miri и Valgrind

Программисты C++ знакомы с Valgrind и санитайзерами. У Rust есть всё это **плюс** Miri, который гораздо точнее для UB, специфичного для Rust:

| | **Miri** | **Valgrind** | **Санитайзеры C++ (ASan/MSan/UBSan)** |
|---|---------|-------------|--------------------------------------|
| **Что находит** | UB, специфичное для Rust: stacked borrows, некорректные дискриминанты `enum`, чтение неинициализированных данных, нарушения алиасинга | Утечки памяти, использование после освобождения, некорректные чтения/записи, неинициализированная память | Переполнение буфера, использование после освобождения, гонки данных, UB |
| **Как работает** | Интерпретирует MIR (промежуточное представление Rust) — без нативного выполнения | Инструментирует скомпилированный бинарный файл во время выполнения | Инструментирование на этапе компиляции |
| **Поддержка FFI** | ❌ Не может пересекать границу FFI (пропускает вызовы C) | ✅ Работает с любым скомпилированным бинарным файлом, включая FFI | ✅ Работает, если код C тоже скомпилирован с санитайзерами |
| **Скорость** | ~100x медленнее нативного | ~10-50x медленнее | ~2-5x медленнее |
| **Когда использовать** | Чистый Rust `unsafe`-код, инварианты структур данных | Код FFI, полные интеграционные тесты бинарного файла | Сторона C/C++ в FFI, тестирование, чувствительное к производительности |
| **Находит ошибки алиасинга** | ✅ Модель Stacked Borrows | ❌ | Частично (TSan для гонок данных) |

**Рекомендация**: используйте **оба** — Miri для чистого unsafe Rust, Valgrind для интеграции FFI:

- **Miri** — находит UB, специфичное для Rust, которое Valgrind не видит (нарушения алиасинга, некорректные значения enum, stacked borrows):
    ```
    rustup +nightly component add miri
    cargo +nightly miri test                    # Запустить все тесты под Miri
    cargo +nightly miri test -- test_name       # Запустить конкретный тест
    ```
    > ⚠️ Miri требует nightly и не может выполнять вызовы FFI. Выносите логику unsafe-Rust в тестируемые модули.

- **Valgrind** — инструмент, который вы уже знаете; работает со скомпилированным бинарным файлом, включая FFI:
    ```
    sudo apt install valgrind
    cargo install cargo-valgrind
    cargo valgrind test                         # Запустить все тесты под Valgrind
    ```
    > Находит утечки в шаблонах `Box::leak` / `Box::from_raw`, которые распространены в коде FFI.

- **cargo-careful** — запускает тесты с дополнительными проверками времени выполнения (между обычными тестами и Miri):
    ```
    cargo install cargo-careful
    cargo +nightly careful test
    ```

## Итоги по unsafe Rust
- ```cbindgen``` — отличный инструмент для FFI (C) в сторону Rust
    - Для FFI-интерфейсов в обратную сторону используйте ```bindgen``` (см. обширную документацию)
- **Не предполагайте, что ваш unsafe-код корректен или что его безопасно использовать из безопасного Rust. Ошибиться очень легко, и даже код, который внешне работает правильно, может быть неверным по тонким причинам**
    - Используйте инструменты для проверки корректности
    - Если сомневаетесь, обратитесь за экспертным советом
- Убедитесь, что ваш код с ```unsafe``` содержит комментарии с явным описанием предположений и причин, по которым он корректен
    - У вызывающего кода ```unsafe``` тоже должны быть соответствующие комментарии о безопасности, и он должен соблюдать ограничения

# Упражнение: написание безопасной обёртки FFI

🔴 **Сложный уровень** — требует понимания блоков unsafe, сырых указателей и проектирования безопасного API

- Напишите безопасную обёртку на Rust вокруг функции `unsafe` в стиле FFI. Упражнение имитирует вызов функции на C, которая записывает отформатированную строку в буфер, предоставленный вызывающей стороной
- **Шаг 1**: реализуйте небезопасную функцию `unsafe_greet`, которая записывает приветствие в сырой буфер `*mut u8`
- **Шаг 2**: напишите безопасную обёртку `safe_greet`, которая выделяет `Vec<u8>`, вызывает небезопасную функцию и возвращает `String`
- **Шаг 3**: добавьте корректные комментарии `// Safety:` к каждому блоку unsafe

**Стартовый код:**
```rust
use std::fmt::Write as _;

/// Имитирует функцию на C: записывает "Hello, <name>!" в буфер.
/// Возвращает количество записанных байтов (без нулевого терминатора).
/// # Safety
/// - `buf` должен указывать на не менее чем `buf_len` доступных для записи байтов
/// - `name` должен быть валидным указателем на C-строку с завершающим нулём
unsafe fn unsafe_greet(buf: *mut u8, buf_len: usize, name: *const u8) -> isize {
    // TODO: Сформируйте приветствие, скопируйте байты в buf, верните длину
    // Подсказка: используйте std::ffi::CStr::from_ptr или перебирайте байты вручную
    todo!()
}

/// Безопасная обёртка — в публичном API нет unsafe
fn safe_greet(name: &str) -> Result<String, String> {
    // TODO: Выделите буфер Vec<u8>, создайте C-строку с нулём в конце для имени,
    // вызовите unsafe_greet внутри блока unsafe с комментарием Safety,
    // преобразуйте результат обратно в String
    todo!()
}

fn main() {
    match safe_greet("Rustacean") {
        Ok(msg) => println!("{msg}"),
        Err(e) => eprintln!("Error: {e}"),
    }
    // Ожидаемый вывод: Hello, Rustacean!
}
```

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
use std::ffi::CStr;

/// Имитирует функцию на C: записывает "Hello, <name>!" в буфер.
/// Возвращает количество записанных байтов или -1, если буфер слишком мал.
/// # Safety
/// - `buf` должен указывать на не менее чем `buf_len` доступных для записи байтов
/// - `name` должен быть валидным указателем на C-строку с завершающим нулём
unsafe fn unsafe_greet(buf: *mut u8, buf_len: usize, name: *const u8) -> isize {
    // Safety: вызывающая сторона гарантирует, что name — валидная строка с завершающим нулём
    let name_cstr = unsafe { CStr::from_ptr(name as *const std::os::raw::c_char) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let greeting = format!("Hello, {}!", name_str);
    if greeting.len() > buf_len {
        return -1;
    }
    // Safety: buf указывает на не менее чем buf_len доступных для записи байтов (гарантия вызывающей стороны)
    unsafe {
        std::ptr::copy_nonoverlapping(greeting.as_ptr(), buf, greeting.len());
    }
    greeting.len() as isize
}

/// Безопасная обёртка — в публичном API нет unsafe
fn safe_greet(name: &str) -> Result<String, String> {
    let mut buffer = vec![0u8; 256];
    // Создаём версию name с завершающим нулём для C API
    let name_with_null: Vec<u8> = name.bytes().chain(std::iter::once(0)).collect();

    // Safety: в buffer 256 доступных для записи байтов, name_with_null заканчивается нулём
    let bytes_written = unsafe {
        unsafe_greet(buffer.as_mut_ptr(), buffer.len(), name_with_null.as_ptr())
    };

    if bytes_written < 0 {
        return Err("Buffer too small or invalid name".to_string());
    }

    String::from_utf8(buffer[..bytes_written as usize].to_vec())
        .map_err(|e| format!("Invalid UTF-8: {e}"))
}

fn main() {
    match safe_greet("Rustacean") {
        Ok(msg) => println!("{msg}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
// Вывод:
// Hello, Rustacean!
```

</details>

----

