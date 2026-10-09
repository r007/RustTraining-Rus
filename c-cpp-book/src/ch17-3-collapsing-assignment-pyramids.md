## Схлопывание пирамид присваиваний с помощью замыканий

> **Что вы узнаете:** как выразительный синтаксис Rust на основе выражений и замыкания превращают глубоко вложенные цепочки проверок `if/else` из C++ в чистый линейный код.

- В C++ для присваивания переменных часто нужны многоблочные цепочки `if/else`, особенно когда есть проверки или логика запасных вариантов. Синтаксис Rust на основе выражений и замыкания сворачивают их в плоский линейный код.

### Паттерн 1: присваивание кортежа через выражение `if`
```cpp
// C++ — три переменные задаются в многоблочной цепочке if/else
uint32_t fault_code;
const char* der_marker;
const char* action;
if (is_c44ad) {
    fault_code = 32709; der_marker = "CSI_WARN"; action = "No action";
} else if (error.is_hardware_error()) {
    fault_code = 67956; der_marker = "CSI_ERR"; action = "Replace GPU";
} else {
    fault_code = 32709; der_marker = "CSI_WARN"; action = "No action";
}
```

```rust
// Эквивалент на Rust:accel_fieldiag.rs
// Одно выражение присваивает все три сразу:
let (fault_code, der_marker, recommended_action) = if is_c44ad {
    (32709u32, "CSI_WARN", "No action")
} else if error.is_hardware_error() {
    (67956u32, "CSI_ERR", "Replace GPU")
} else {
    (32709u32, "CSI_WARN", "No action")
};
```

### Паттерн 2: IIFE (немедленно вызываемое функциональное выражение) для цепочек с возможными ошибками
```cpp
// C++ — «пирамида гибели» для навигации по JSON
std::string get_part_number(const nlohmann::json& root) {
    if (root.contains("SystemInfo")) {
        auto& sys = root["SystemInfo"];
        if (sys.contains("BaseboardFru")) {
            auto& bb = sys["BaseboardFru"];
            if (bb.contains("ProductPartNumber")) {
                return bb["ProductPartNumber"].get<std::string>();
            }
        }
    }
    return "UNKNOWN";
}
```

```rust
// Эквивалент на Rust:framework.rs
// Замыкание + оператор ? превращают пирамиду в линейный код:
let part_number = (|| -> Option<String> {
    let path = self.args.sysinfo.as_ref()?;
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let ppn = json
        .get("SystemInfo")?
        .get("BaseboardFru")?
        .get("ProductPartNumber")?
        .as_str()?;
    Some(ppn.to_string())
})()
.unwrap_or_else(|| "UNKNOWN".to_string());
```
Замыкание создаёт область `Option<String>`, в которой `?` досрочно прерывает выполнение на любом шаге. `.unwrap_or_else()` задаёт запасной вариант один раз, в конце.

### Паттерн 3: цепочка итераторов вместо ручного цикла + push_back
```cpp
// C++ — ручной цикл с промежуточными переменными
std::vector<std::tuple<std::vector<std::string>, std::string, std::string>> gpu_info;
for (const auto& [key, info] : gpu_pcie_map) {
    std::vector<std::string> bdfs;
    // ... разбираем bdf_path в bdfs
    std::string serial = info.serial_number.value_or("UNKNOWN");
    std::string model = info.model_number.value_or(model_name);
    gpu_info.push_back({bdfs, serial, model});
}
```

```rust
// Эквивалент на Rust:peripherals.rs
// Одна цепочка: values() → map → collect
let gpu_info: Vec<(Vec<String>, String, String, String)> = self
    .gpu_pcie_map
    .values()
    .map(|info| {
        let bdfs: Vec<String> = info.bdf_path
            .split(')')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim_start_matches('(').to_string())
            .collect();
        let serial = info.serial_number.clone()
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let model = info.model_number.clone()
            .unwrap_or_else(|| model_name.to_string());
        let gpu_bdf = format!("{}:{}:{}.{}",
            info.bdf.segment, info.bdf.bus, info.bdf.device, info.bdf.function);
        (bdfs, serial, model, gpu_bdf)
    })
    .collect();
```

### Паттерн 4: `.filter().collect()` вместо цикла + `if (condition) continue`
```cpp
// C++
std::vector<TestResult*> failures;
for (auto& t : test_results) {
    if (!t.is_pass()) {
        failures.push_back(&t);
    }
}
```

```rust
// Rust — из accel_diag/src/healthcheck.rs
pub fn failed_tests(&self) -> Vec<&TestResult> {
    self.test_results.iter().filter(|t| !t.is_pass()).collect()
}
```

### Итоги: когда применять каждый паттерн
| **Паттерн C++** | **Замена в Rust** | **Ключевое преимущество** |
|----------------|---------------------|-----------------|
| Присваивание переменных в многоблочном `if` | `let (a, b) = if ... { } else { };` | Все переменные связываются атомарно |
| Вложенная пирамида `if (contains)` | IIFE-замыкание с оператором `?` | Линейно, плоско, ранний выход |
| Цикл `for` + `push_back` | `.iter().map(\|\|).collect()` | Не нужен промежуточный изменяемый Vec |
| `for` + `if (cond) continue` | `.iter().filter(\|\|).collect()` | Декларативное намерение |
| `for` + `if + break` (поиск первого) | `.iter().find_map(\|\|)` | Поиск и преобразование за один проход |

----

# Итоговое упражнение: конвейер диагностических событий

🔴 **Сложный уровень** — комплексное упражнение, объединяющее перечисления, трейты, итераторы, обработку ошибок и обобщения

Это комплексное упражнение объединяет перечисления, трейты, итераторы, обработку ошибок и обобщения. Вы создадите упрощённый конвейер обработки диагностических событий, похожий на паттерны, используемые в продакшн-коде на Rust.

**Требования:**
1. Определите `enum Severity { Info, Warning, Critical }` с реализацией `Display`, а также `struct DiagEvent`, содержащую `source: String`, `severity: Severity`, `message: String` и `fault_code: u32`
2. Определите `trait EventFilter` с методом `fn should_include(&self, event: &DiagEvent) -> bool`
3. Реализуйте два фильтра: `SeverityFilter` (только события с серьёзностью >= заданной) и `SourceFilter` (только события от определённого источника)
4. Напишите функцию `fn process_events(events: &[DiagEvent], filters: &[&dyn EventFilter]) -> Vec<String>`, которая возвращает отформатированные строки отчёта для событий, прошедших **все** фильтры
5. Напишите `fn parse_event(line: &str) -> Result<DiagEvent, String>`, которая разбирает строки вида `"source:severity:fault_code:message"` (возвращает `Err` для некорректного ввода)

**Стартовый код:**
```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Info,
    Warning,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct DiagEvent {
    source: String,
    severity: Severity,
    message: String,
    fault_code: u32,
}

trait EventFilter {
    fn should_include(&self, event: &DiagEvent) -> bool;
}

struct SeverityFilter {
    min_severity: Severity,
}
// TODO: impl EventFilter for SeverityFilter

struct SourceFilter {
    source: String,
}
// TODO: impl EventFilter for SourceFilter

fn process_events(events: &[DiagEvent], filters: &[&dyn EventFilter]) -> Vec<String> {
    // TODO: Отфильтровать события, прошедшие ВСЕ фильтры, и отформатировать как
    // "[SEVERITY] source (FC:fault_code): message"
    todo!()
}

fn parse_event(line: &str) -> Result<DiagEvent, String> {
    // Разобрать "source:severity:fault_code:message"
    // Вернуть Err для некорректного ввода
    todo!()
}

fn main() {
    let raw_lines = vec![
        "accel_diag:Critical:67956:ECC uncorrectable error detected",
        "nic_diag:Warning:32709:Link speed degraded",
        "accel_diag:Info:10001:Self-test passed",
        "cpu_diag:Critical:55012:Thermal throttling active",
        "accel_diag:Warning:32710:PCIe link width reduced",
    ];

    // Разобрать все строки, собрать успешные и вывести ошибки разбора
    let events: Vec<DiagEvent> = raw_lines.iter()
        .filter_map(|line| match parse_event(line) {
            Ok(e) => Some(e),
            Err(e) => { eprintln!("Parse error: {e}"); None }
        })
        .collect();

    // Применяем фильтры: только Critical и Warning от accel_diag
    let sev_filter = SeverityFilter { min_severity: Severity::Warning };
    let src_filter = SourceFilter { source: "accel_diag".to_string() };
    let filters: Vec<&dyn EventFilter> = vec![&sev_filter, &src_filter];

    let report = process_events(&events, &filters);
    for line in &report {
        println!("{line}");
    }
    println!("--- {} event(s) matched ---", report.len());
}
```

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Info,
    Warning,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl Severity {
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Info" => Ok(Severity::Info),
            "Warning" => Ok(Severity::Warning),
            "Critical" => Ok(Severity::Critical),
            other => Err(format!("Unknown severity: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
struct DiagEvent {
    source: String,
    severity: Severity,
    message: String,
    fault_code: u32,
}

trait EventFilter {
    fn should_include(&self, event: &DiagEvent) -> bool;
}

struct SeverityFilter {
    min_severity: Severity,
}

impl EventFilter for SeverityFilter {
    fn should_include(&self, event: &DiagEvent) -> bool {
        event.severity >= self.min_severity
    }
}

struct SourceFilter {
    source: String,
}

impl EventFilter for SourceFilter {
    fn should_include(&self, event: &DiagEvent) -> bool {
        event.source == self.source
    }
}

fn process_events(events: &[DiagEvent], filters: &[&dyn EventFilter]) -> Vec<String> {
    events.iter()
        .filter(|e| filters.iter().all(|f| f.should_include(e)))
        .map(|e| format!("[{}] {} (FC:{}): {}", e.severity, e.source, e.fault_code, e.message))
        .collect()
}

fn parse_event(line: &str) -> Result<DiagEvent, String> {
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    if parts.len() != 4 {
        return Err(format!("Expected 4 colon-separated fields, got {}", parts.len()));
    }
    let fault_code = parts[2].parse::<u32>()
        .map_err(|e| format!("Invalid fault code '{}': {e}", parts[2]))?;
    Ok(DiagEvent {
        source: parts[0].to_string(),
        severity: Severity::from_str(parts[1])?,
        fault_code,
        message: parts[3].to_string(),
    })
}

fn main() {
    let raw_lines = vec![
        "accel_diag:Critical:67956:ECC uncorrectable error detected",
        "nic_diag:Warning:32709:Link speed degraded",
        "accel_diag:Info:10001:Self-test passed",
        "cpu_diag:Critical:55012:Thermal throttling active",
        "accel_diag:Warning:32710:PCIe link width reduced",
    ];

    let events: Vec<DiagEvent> = raw_lines.iter()
        .filter_map(|line| match parse_event(line) {
            Ok(e) => Some(e),
            Err(e) => { eprintln!("Parse error: {e}"); None }
        })
        .collect();

    let sev_filter = SeverityFilter { min_severity: Severity::Warning };
    let src_filter = SourceFilter { source: "accel_diag".to_string() };
    let filters: Vec<&dyn EventFilter> = vec![&sev_filter, &src_filter];

    let report = process_events(&events, &filters);
    for line in &report {
        println!("{line}");
    }
    println!("--- {} event(s) matched ---", report.len());
}
// Вывод:
// [CRITICAL] accel_diag (FC:67956): ECC uncorrectable error detected
// [WARNING] accel_diag (FC:32710): PCIe link width reduced
// --- 2 event(s) matched ---
```

</details>

----

