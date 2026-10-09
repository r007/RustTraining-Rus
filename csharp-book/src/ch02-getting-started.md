## Установка и настройка

> **Что вы узнаете:** как установить Rust и настроить IDE, чем система сборки Cargo отличается от MSBuild/NuGet,
> как выглядит первая программа на Rust по сравнению с C#, и как читать ввод из командной строки.
>
> **Сложность:** 🟢 Начальный

### Установка Rust
```bash
# Установка Rust (работает на Windows, macOS, Linux)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# В Windows можно также скачать с: https://rustup.rs/
```

### Инструменты Rust против инструментов C#
| Инструмент C# | Аналог в Rust | Назначение |
|---------|----------------|---------|
| `dotnet new` | `cargo new` | Создание нового проекта |
| `dotnet build` | `cargo build` | Компиляция проекта |
| `dotnet run` | `cargo run` | Запуск проекта |
| `dotnet test` | `cargo test` | Запуск тестов |
| NuGet | Crates.io | Репозиторий пакетов |
| MSBuild | Cargo | Система сборки |
| Visual Studio | VS Code + rust-analyzer | IDE |

### Настройка IDE
1. **VS Code** (рекомендуется для начинающих)
   - Установите расширение «rust-analyzer»
   - Установите «CodeLLDB» для отладки

2. **Visual Studio** (Windows)
   - Установите расширение с поддержкой Rust

3. **JetBrains RustRover** (полноценная IDE)
   - Похожа на Rider для C#

***

## Первая программа на Rust

### Hello World на C#
```csharp
// Program.cs
using System;

namespace HelloWorld
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("Hello, World!");
        }
    }
}
```

### Hello World на Rust
```rust
// main.rs
fn main() {
    println!("Hello, World!");
}
```

### Ключевые отличия для разработчиков C#
1. **Классы не требуются** — функции могут существовать на верхнем уровне
2. **Пространств имён нет** — вместо них используется система модулей
3. **`println!` — это макрос** — обратите внимание на `!`
4. **Точка с запятой важна** — отсутствие завершающей точки с запятой превращает инструкцию в возвращаемое выражение
5. **Явного типа возвращаемого значения нет** — `main` возвращает `()` (тип unit)

### Создание первого проекта
```bash
# Создание нового проекта (аналог 'dotnet new console')
cargo new hello_rust
cd hello_rust

# Созданная структура проекта:
# hello_rust/
# ├── Cargo.toml      (аналог файла .csproj)
# └── src/
#     └── main.rs     (аналог Program.cs)

# Запуск проекта (аналог 'dotnet run')
cargo run
```

***

## Cargo против NuGet/MSBuild

### Конфигурация проекта

**C# (.csproj)**
```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
  
  <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
  <PackageReference Include="Serilog" Version="3.0.1" />
</Project>
```

**Rust (Cargo.toml)**
```toml
[package]
name = "hello_rust"
version = "0.1.0"
edition = "2021"

[dependencies]
serde_json = "1.0"    # Аналог Newtonsoft.Json
log = "0.4"           # Аналог Serilog
```

### Основные команды Cargo
```bash
# Создание нового проекта
cargo new my_project
cargo new my_project --lib  # Создать библиотечный проект

# Сборка и запуск
cargo build          # Аналог 'dotnet build'
cargo run            # Аналог 'dotnet run'
cargo test           # Аналог 'dotnet test'

# Управление пакетами
cargo add serde      # Добавить зависимость (аналог 'dotnet add package')
cargo update         # Обновить зависимости

# Релизная сборка
cargo build --release  # Оптимизированная сборка
cargo run --release    # Запуск оптимизированной версии

# Документация
cargo doc --open     # Сгенерировать и открыть документацию
```

### Workspace против Solution

**Решение C# (.sln)**
```text
MySolution/
├── MySolution.sln
├── WebApi/
│   └── WebApi.csproj
├── Business/
│   └── Business.csproj
└── Tests/
    └── Tests.csproj
```

**Рабочее пространство Rust (Cargo.toml)**
```toml
[workspace]
members = [
    "web_api",
    "business", 
    "tests"
]
```

***

## Чтение ввода и аргументов командной строки

Каждый разработчик C# знает `Console.ReadLine()`. Вот как в Rust обрабатывать пользовательский ввод, переменные окружения и аргументы командной строки.

### Ввод с консоли
```csharp
// C# — чтение пользовательского ввода
Console.Write("Введите ваше имя: ");
string? name = Console.ReadLine();  // Возвращает string? начиная с .NET 6+
Console.WriteLine($"Привет, {name}!");

// Разбор ввода
Console.Write("Введите число: ");
if (int.TryParse(Console.ReadLine(), out int number))
{
    Console.WriteLine($"Вы ввели: {number}");
}
else
{
    Console.WriteLine("Это не число.");
}
```

```rust
use std::io::{self, Write};

fn main() {
    // Чтение строки ввода
    print!("Введите ваше имя: ");
    io::stdout().flush().unwrap(); // flush нужен, потому что print! не сбрасывает буфер автоматически

    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("Не удалось прочитать строку");
    let name = name.trim(); // убираем завершающий перенос строки
    println!("Привет, {name}!");

    // Разбор ввода
    print!("Введите число: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Не удалось прочитать");
    match input.trim().parse::<i32>() {
        Ok(number) => println!("Вы ввели: {number}"),
        Err(_)     => println!("Это не число."),
    }
}
```

### Аргументы командной строки
```csharp
// C# — чтение аргументов CLI
static void Main(string[] args)
{
    if (args.Length < 1)
    {
        Console.WriteLine("Использование: program <filename>");
        return;
    }
    string filename = args[0];
    Console.WriteLine($"Обработка {filename}");
}
```

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    //  args[0] = имя программы (аналог имени сборки в C#)
    //  args[1..] = собственно аргументы

    if args.len() < 2 {
        eprintln!("Использование: {} <filename>", args[0]); // eprintln! → stderr
        std::process::exit(1);
    }
    let filename = &args[1];
    println!("Обработка {filename}");
}
```

### Переменные окружения
```csharp
// C#
string dbUrl = Environment.GetEnvironmentVariable("DATABASE_URL") ?? "localhost";
```

```rust
use std::env;

let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "localhost".to_string());
// env::var возвращает Result<String, VarError> — никаких null!
```

### Промышленные CLI-приложения с `clap`

Для всего, что выходит за рамки тривиального разбора аргументов, используйте крейт **`clap`** — это аналог `System.CommandLine` или библиотек вроде `CommandLineParser` в Rust.

```toml
# Cargo.toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

```rust
use clap::Parser;

/// Простой обработчик файлов — этот doc-комментарий станет текстом справки
#[derive(Parser, Debug)]
#[command(name = "processor", version, about)]
struct Args {
    /// Входной файл для обработки
    #[arg(short, long)]
    input: String,

    /// Выходной файл (по умолчанию — stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Включить подробное логирование
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Количество рабочих потоков
    #[arg(short = 'j', long, default_value_t = 4)]
    threads: usize,
}

fn main() {
    let args = Args::parse(); // автоматически разбирает, проверяет и генерирует --help

    if args.verbose {
        println!("Вход:    {}", args.input);
        println!("Выход:   {:?}", args.output);
        println!("Потоки:  {}", args.threads);
    }

    // Используйте args.input, args.output и т.д.
}
```

```bash
# Автоматически сгенерированная справка:
$ processor --help
Простой обработчик файлов

Usage: processor [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>      Входной файл для обработки
  -o, --output <OUTPUT>    Выходной файл (по умолчанию — stdout)
  -v, --verbose            Включить подробное логирование
  -j, --threads <THREADS>  Количество рабочих потоков [default: 4]
  -h, --help               Print help
  -V, --version            Print version
```

```csharp
// Эквивалент на C# с System.CommandLine (больше шаблонного кода):
var inputOption = new Option<string>("--input", "Входной файл") { IsRequired = true };
var verboseOption = new Option<bool>("--verbose", "Включить подробное логирование");
var rootCommand = new RootCommand("Простой обработчик файлов");
rootCommand.AddOption(inputOption);
rootCommand.AddOption(verboseOption);
rootCommand.SetHandler((input, verbose) => { /* ... */ }, inputOption, verboseOption);
await rootCommand.InvokeAsync(args);
// Подход с derive-макросом в clap короче и безопаснее по типам
```

| C# | Rust | Примечания |
|----|------|-------|
| `Console.ReadLine()` | `io::stdin().read_line(&mut buf)` | Нужно передать буфер, возвращает `Result` |
| `int.TryParse(s, out n)` | `s.parse::<i32>()` | Возвращает `Result<i32, ParseIntError>` |
| `args[0]` | `env::args().nth(1)` | В Rust args[0] — имя программы |
| `Environment.GetEnvironmentVariable` | `env::var("KEY")` | Возвращает `Result`, а не nullable-значение |
| `System.CommandLine` | `clap` | Основан на derive, автоматически генерирует справку |

***

