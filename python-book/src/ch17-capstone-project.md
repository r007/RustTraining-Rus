## Итоговый проект: CLI-менеджер задач

> **Что вы узнаете:** как объединить всё изученное в курсе, написав полноценное CLI-приложение на Rust. Обычно такое приложение пишут на Python с помощью `argparse` + `json` + `pathlib`.
>
> **Сложность:** 🔴 Продвинутый

Этот итоговый проект использует концепции из каждой ключевой главы:
- **Гл. 3**: типы и переменные (структуры, перечисления)
- **Гл. 5**: коллекции (`Vec`, `HashMap`)
- **Гл. 6**: перечисления и сопоставление с образцом (статус задачи, команды)
- **Гл. 7**: владение и заимствование (передача ссылок)
- **Гл. 9**: обработка ошибок (`Result`, `?`, собственные ошибки)
- **Гл. 10**: трейты (`Display`, `FromStr`)
- **Гл. 11**: преобразования типов (`From`, `TryFrom`)
- **Гл. 12**: итераторы и замыкания (фильтрация, преобразование)
- **Гл. 8**: модули (организация структуры проекта)

***

## Проект: `rustdo`

Менеджер задач для командной строки (вроде инструментов `todo.txt` в Python), который хранит задачи в JSON-файле.

### Аналог на Python (что вы написали бы на Python)

```python
#!/usr/bin/env python3
"""Простой CLI-менеджер задач: версия на Python."""
import json
import sys
from pathlib import Path
from datetime import datetime
from enum import Enum

TASK_FILE = Path.home() / ".rustdo.json"

class Priority(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"

class Task:
    def __init__(self, id: int, title: str, priority: Priority, done: bool = False):
        self.id = id
        self.title = title
        self.priority = priority
        self.done = done
        self.created = datetime.now().isoformat()

def load_tasks() -> list[Task]:
    if not TASK_FILE.exists():
        return []
    data = json.loads(TASK_FILE.read_text())
    return [Task(**t) for t in data]

def save_tasks(tasks: list[Task]):
    TASK_FILE.write_text(json.dumps([t.__dict__ for t in tasks], indent=2))

# Команды: add, list, done, remove, stats
# ... (вы и так знаете, как это делается на Python)
```

### Ваша реализация на Rust

Собирайте проект по шагам. Каждый шаг соответствует концепциям из определённых глав.

***

## Шаг 1: определите модель данных (гл. 3, 6, 10, 11)

```rust
// src/task.rs
use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use chrono::Local;

/// Приоритет задачи: соответствует Priority(Enum) в Python
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

// Трейт Display (аналог __str__ в Python)
impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
        }
    }
}

// Трейт FromStr (разбор "high" → Priority::High)
impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" | "l" => Ok(Priority::Low),
            "medium" | "med" | "m" => Ok(Priority::Medium),
            "high" | "h" => Ok(Priority::High),
            other => Err(format!("неизвестный приоритет: '{other}' (используйте low/medium/high)")),
        }
    }
}

/// Одна задача: соответствует классу Task в Python
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub priority: Priority,
    pub done: bool,
    pub created: String,
}

impl Task {
    pub fn new(id: u32, title: String, priority: Priority) -> Self {
        Self {
            id,
            title,
            priority,
            done: false,
            created: Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.done { "✅" } else { "⬜" };
        let priority_icon = match self.priority {
            Priority::Low => "🟢",
            Priority::Medium => "🟡",
            Priority::High => "🔴",
        };
        write!(f, "{} {} [{}] {} ({})", status, self.id, priority_icon, self.title, self.created)
    }
}
```

> **Сравнение с Python**: в Python вы бы использовали `@dataclass` + `Enum`. В Rust `struct` + `enum` + макросы `derive` дают сериализацию, вывод и разбор бесплатно.

***

## Шаг 2: слой хранения (гл. 9, 7)

```rust
// src/storage.rs
use std::fs;
use std::path::PathBuf;
use crate::task::Task;

/// Путь к файлу задач (~/.rustdo.json)
fn task_file_path() -> PathBuf {
    let home = dirs::home_dir().expect("Не удалось определить домашний каталог");
    home.join(".rustdo.json")
}

/// Загрузить задачи с диска: возвращает пустой Vec, если файла нет
pub fn load_tasks() -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let path = task_file_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)?;  // ? передаёт io::Error дальше
    let tasks: Vec<Task> = serde_json::from_str(&content)?;  // ? передаёт ошибку serde дальше
    Ok(tasks)
}

/// Сохранить задачи на диск
pub fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn std::error::Error>> {
    let path = task_file_path();
    let json = serde_json::to_string_pretty(tasks)?;
    fs::write(&path, json)?;
    Ok(())
}
```

> **Сравнение с Python**: Python использует `Path.read_text()` + `json.loads()`. Rust использует `fs::read_to_string()` + `serde_json::from_str()`. Обратите внимание на `?`: каждая ошибка явная и передаётся дальше.

***

## Шаг 3: перечисление команд (гл. 6)

```rust
// src/command.rs
use crate::task::Priority;

/// Все возможные команды: один вариант перечисления на каждое действие
pub enum Command {
    Add { title: String, priority: Priority },
    List { show_done: bool },
    Done { id: u32 },
    Remove { id: u32 },
    Stats,
    Help,
}

impl Command {
    /// Разбирает аргументы командной строки в Command
    /// (в реальном проекте используйте `clap`, а здесь это учебный пример)
    pub fn parse(args: &[String]) -> Result<Self, String> {
        match args.first().map(|s| s.as_str()) {
            Some("add") => {
                let title = args.get(1)
                    .ok_or("использование: rustdo add <title> [priority]")?
                    .clone();
                let priority = args.get(2)
                    .map(|p| p.parse::<Priority>())
                    .transpose()
                    .map_err(|e| e.to_string())?
                    .unwrap_or(Priority::Medium);
                Ok(Command::Add { title, priority })
            }
            Some("list") => {
                let show_done = args.get(1).map(|s| s == "--all").unwrap_or(false);
                Ok(Command::List { show_done })
            }
            Some("done") => {
                let id: u32 = args.get(1)
                    .ok_or("использование: rustdo done <id>")?
                    .parse()
                    .map_err(|_| "id должен быть числом")?;
                Ok(Command::Done { id })
            }
            Some("remove") => {
                let id: u32 = args.get(1)
                    .ok_or("использование: rustdo remove <id>")?
                    .parse()
                    .map_err(|_| "id должен быть числом")?;
                Ok(Command::Remove { id })
            }
            Some("stats") => Ok(Command::Stats),
            _ => Ok(Command::Help),
        }
    }
}
```

> **Сравнение с Python**: Python использует `argparse` или `click`. Этот самописный разбор показывает, как `match` по шаблонам, похожим на перечисления, заменяет цепочки if/elif в Python. Для реальных проектов используйте крейт `clap`.

***

## Шаг 4: бизнес-логика (гл. 5, 12, 7)

```rust
// src/actions.rs
use crate::task::{Task, Priority};
use crate::storage;

pub fn add_task(title: String, priority: Priority) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage::load_tasks()?;
    let next_id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let task = Task::new(next_id, title.clone(), priority);
    println!("Добавлено: {task}");
    tasks.push(task);
    storage::save_tasks(&tasks)?;
    Ok(())
}

pub fn list_tasks(show_done: bool) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = storage::load_tasks()?;
    let filtered: Vec<&Task> = tasks.iter()
        .filter(|t| show_done || !t.done)   // Итератор + замыкание (гл. 12)
        .collect();

    if filtered.is_empty() {
        println!("Задач нет! 🎉");
        return Ok(());
    }

    for task in &filtered {
        println!("  {task}");   // Использует трейт Display (гл. 10)
    }
    println!("\nПоказано задач: {}", filtered.len());
    Ok(())
}

pub fn complete_task(id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage::load_tasks()?;
    let task = tasks.iter_mut()
        .find(|t| t.id == id)                // Iterator::find (гл. 12)
        .ok_or(format!("Задачи с id {id} нет"))?;
    task.done = true;
    println!("Выполнено: {task}");
    storage::save_tasks(&tasks)?;
    Ok(())
}

pub fn remove_task(id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage::load_tasks()?;
    let len_before = tasks.len();
    tasks.retain(|t| t.id != id);            // Vec::retain (гл. 5)
    if tasks.len() == len_before {
        return Err(format!("Задачи с id {id} нет").into());
    }
    println!("Удалена задача {id}");
    storage::save_tasks(&tasks)?;
    Ok(())
}

pub fn show_stats() -> Result<(), Box<dyn std::error::Error>> {
    let tasks = storage::load_tasks()?;
    let total = tasks.len();
    let done = tasks.iter().filter(|t| t.done).count();
    let pending = total - done;

    // Группировка по приоритету с помощью итераторов (гл. 12)
    let high = tasks.iter().filter(|t| !t.done && t.priority == Priority::High).count();
    let medium = tasks.iter().filter(|t| !t.done && t.priority == Priority::Medium).count();
    let low = tasks.iter().filter(|t| !t.done && t.priority == Priority::Low).count();

    println!("📊 Статистика задач");
    println!("   Всего:       {total}");
    println!("   Выполнено:   {done} ✅");
    println!("   В работе:    {pending}");
    println!("   🔴 Высокий:  {high}");
    println!("   🟡 Средний:  {medium}");
    println!("   🟢 Низкий:   {low}");
    Ok(())
}
```

> **Используемые паттерны Rust**: `iter().map().max()`, `iter().filter().collect()`, `iter_mut().find()`, `retain()`, `iter().filter().count()`. Они заменяют списочные включения Python, `next(x for x in ...)` и `Counter`.

***

## Шаг 5: собираем всё вместе (гл. 8)

```rust
// src/main.rs
mod task;
mod storage;
mod command;
mod actions;

use command::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match Command::parse(&args) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Ошибка: {e}");
            std::process::exit(1);
        }
    };

    let result = match command {
        Command::Add { title, priority } => actions::add_task(title, priority),
        Command::List { show_done } => actions::list_tasks(show_done),
        Command::Done { id } => actions::complete_task(id),
        Command::Remove { id } => actions::remove_task(id),
        Command::Stats => actions::show_stats(),
        Command::Help => {
            print_help();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Ошибка: {e}");
        std::process::exit(1);
    }
}

fn print_help() {
    println!("rustdo — менеджер задач для питонистов, которые изучают Rust\n");
    println!("ИСПОЛЬЗОВАНИЕ:");
    println!("  rustdo add <title> [low|medium|high]   Добавить задачу");
    println!("  rustdo list [--all]                    Показать невыполненные задачи");
    println!("  rustdo done <id>                       Отметить задачу выполненной");
    println!("  rustdo remove <id>                     Удалить задачу");
    println!("  rustdo stats                           Показать статистику");
}
```

```mermaid
graph TD
    CLI["main.rs<br/>(точка входа CLI)"] --> CMD["command.rs<br/>(разбор аргументов)"]
    CMD --> ACT["actions.rs<br/>(бизнес-логика)"]
    ACT --> STORE["storage.rs<br/>(хранение в JSON)"]
    ACT --> TASK["task.rs<br/>(модель данных)"]
    STORE --> TASK
    style CLI fill:#d4edda
    style CMD fill:#fff3cd
    style ACT fill:#fff3cd
    style STORE fill:#ffeeba
    style TASK fill:#ffeeba
```

***

## Шаг 6: зависимости в Cargo.toml

```toml
[package]
name = "rustdo"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
dirs = "5"
```

> **Аналог в Python**: это ваш `[project.dependencies]` из `pyproject.toml`. `cargo add serde serde_json chrono dirs` — аналог `pip install`.

***

## Шаг 7: тесты (гл. 14)

```rust
// src/task.rs — добавьте в конец файла
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_priority() {
        assert_eq!("high".parse::<Priority>().unwrap(), Priority::High);
        assert_eq!("H".parse::<Priority>().unwrap(), Priority::High);
        assert_eq!("med".parse::<Priority>().unwrap(), Priority::Medium);
        assert!("invalid".parse::<Priority>().is_err());
    }

    #[test]
    fn task_display() {
        let task = Task::new(1, "Write Rust".to_string(), Priority::High);
        let display = format!("{task}");
        assert!(display.contains("Write Rust"));
        assert!(display.contains("🔴"));
        assert!(display.contains("⬜")); // Ещё не выполнена
    }

    #[test]
    fn task_serialization_roundtrip() {
        let task = Task::new(1, "Test".to_string(), Priority::Low);
        let json = serde_json::to_string(&task).unwrap();
        let recovered: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.title, "Test");
        assert_eq!(recovered.priority, Priority::Low);
    }
}
```

> **Аналог в Python**: тесты `pytest`. Запускайте их через `cargo test` вместо `pytest`. Никакой магии поиска тестов не нужно: `#[test]` явно помечает тестовые функции.

***

## Дополнительные задания

Когда базовая версия заработает, попробуйте эти улучшения:

1. **Добавьте `clap` для разбора аргументов**: замените самописный разбор макросами derive из `clap`:
   ```rust
   #[derive(Parser)]
   enum Command {
       Add { title: String, #[arg(default_value = "medium")] priority: Priority },
       List { #[arg(long)] all: bool },
       Done { id: u32 },
       Remove { id: u32 },
       Stats,
   }
   ```

2. **Добавьте цветной вывод**: используйте крейт `colored` для цветов в терминале (аналог `colorama` в Python).

3. **Добавьте сроки выполнения**: добавьте поле `Option<NaiveDate>` и фильтруйте просроченные задачи.

4. **Добавьте теги и категории**: используйте `Vec<String>` для тегов и фильтруйте через `.iter().any()`.

5. **Сделайте библиотеку и исполняемый файл**: разделите код на `lib.rs` и `main.rs`, чтобы логику можно было переиспользовать (паттерн модулей, гл. 8).

***

## Что вы попрактиковали

| Глава | Концепция | Где использовалась |
|-------|-----------|--------------------|
| Гл. 3 | Типы и переменные | Поля структуры `Task`, `u32`, `String`, `bool` |
| Гл. 5 | Коллекции | `Vec<Task>`, `retain()`, `push()` |
| Гл. 6 | Перечисления и match | `Priority`, `Command`, исчерпывающее сопоставление |
| Гл. 7 | Владение и заимствование | `&[Task]` и `Vec<Task>`, `&mut` для отметки о выполнении |
| Гл. 8 | Модули | `mod task; mod storage; mod command; mod actions;` |
| Гл. 9 | Обработка ошибок | `Result<T, E>`, оператор `?`, `.ok_or()` |
| Гл. 10 | Трейты | `Display`, `FromStr`, `Serialize`, `Deserialize` |
| Гл. 11 | From/Into | `FromStr` для Priority, `.into()` для преобразования ошибок |
| Гл. 12 | Итераторы | `filter`, `map`, `find`, `count`, `collect` |
| Гл. 14 | Тестирование | `#[test]`, `#[cfg(test)]`, макросы проверок |

> 🎓 **Поздравляем!** Если вы создали этот проект, вы использовали каждую ключевую концепцию Rust, которая разобрана в этой книге. Вы больше не программист на Python, который учит Rust, а разработчик на Rust, который знает Python.

***

