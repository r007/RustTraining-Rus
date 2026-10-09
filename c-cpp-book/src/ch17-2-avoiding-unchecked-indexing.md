## Избегание неконтролируемой индексации

> **Что вы узнаете:** почему `vec[i]` опасен в Rust (вызывает panic при выходе за границы), и безопасные альтернативы — `.get()`, итераторы и API `entry()` для `HashMap`. Они заменяют неопределённое поведение C++ явной обработкой.

- В C++ `vec[i]` и `map[key]` ведут к неопределённому поведению или вставляют новый ключ, если его не было. `[]` в Rust вызывает panic при выходе за границы.
- **Правило**: используйте `.get()` вместо `[]`, если не можете *доказать*, что индекс корректен.

### Сравнение C++ → Rust
```cpp
// C++ — тихое неопределённое поведение или вставка
std::vector<int> v = {1, 2, 3};
int x = v[10];        // Неопределённое поведение! operator[] не проверяет границы

std::map<std::string, int> m;
int y = m["missing"]; // Тихо вставляет ключ со значением 0!
```

```rust
// Rust — безопасные альтернативы
let v = vec![1, 2, 3];

// Плохо: вызывает panic, если индекс вне границ
// let x = v[10];

// Хорошо: возвращает Option<&i32>
let x = v.get(10);              // None — без panic
let x = v.get(1).copied().unwrap_or(0);  // 2, или 0, если элемента нет
```

### Реальный пример: безопасный разбор байтов из продакшн-кода на Rust
```rust
// Пример: diagnostics.rs
// Разбор бинарной записи SEL — буфер может оказаться короче, чем ожидалось
let sensor_num = bytes.get(7).copied().unwrap_or(0);
let ppin = cpu_ppin.get(i).map(|s| s.as_str()).unwrap_or("");
```

### Реальный пример: цепочка безопасных поисков через `.and_then()`
```rust
// Пример: profile.rs — двойной поиск: HashMap → Vec
pub fn get_processor(&self, location: &str) -> Option<&Processor> {
    self.processor_by_location
        .get(location)                              // HashMap → Option<&usize>
        .and_then(|&idx| self.processors.get(idx))   // Vec → Option<&Processor>
}
// Оба поиска возвращают Option — ни panic, ни неопределённого поведения
```

### Реальный пример: безопасная навигация по JSON
```rust
// Пример: framework.rs — каждый ключ JSON возвращает Option
let manufacturer = product_fru
    .get("Manufacturer")            // Option<&Value>
    .and_then(|v| v.as_str())       // Option<&str>
    .unwrap_or(UNKNOWN_VALUE)       // &str (безопасное значение по умолчанию)
    .to_string();
```
Сравните с паттерном C++: `json["SystemInfo"]["ProductFru"]["Manufacturer"]` — любой отсутствующий ключ выбрасывает `nlohmann::json::out_of_range`.

### Когда `[]` допустим
- **После проверки границ**: `if i < v.len() { v[i] }`
- **В тестах**: где panic — желаемое поведение
- **С константами**: `let first = v[0];` сразу после `assert!(!v.is_empty());`

----

## Безопасное извлечение значения с unwrap_or

- `unwrap()` вызывает panic при `None` / `Err`. В продакшн-коде предпочитайте безопасные альтернативы.

### Семейство unwrap
| **Метод** | **Поведение при None/Err** | **Когда использовать** |
|-----------|------------------------|-------------|
| `.unwrap()` | **Panic** | Только в тестах или когда ошибка исключена |
| `.expect("msg")` | Panic с сообщением | Когда panic оправдан — объясните почему |
| `.unwrap_or(default)` | Возвращает `default` | Есть дешёвое константное значение по умолчанию |
| `.unwrap_or_else(\|\| expr)` | Вызывает замыкание | Значение по умолчанию дорого вычислять |
| `.unwrap_or_default()` | Возвращает `Default::default()` | Тип реализует `Default` |

### Реальный пример: разбор с безопасными значениями по умолчанию
```rust
// Пример: peripherals.rs
// Группы захвата регулярного выражения могут не совпасть — предусматриваем безопасные значения
let bus_hex = caps.get(1).map(|m| m.as_str()).unwrap_or("00");
let fw_status = caps.get(5).map(|m| m.as_str()).unwrap_or("0x0");
let bus = u8::from_str_radix(bus_hex, 16).unwrap_or(0);
```

### Реальный пример: `unwrap_or_else` с запасной структурой
```rust
// Пример: framework.rs
// Вся функция оборачивает логику в замыкание, возвращающее Option;
// если что-то не получилось, возвращаем структуру по умолчанию:
(|| -> Option<BaseboardFru> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    // ... извлекаем поля цепочками .get()?
    Some(baseboard_fru)
})()
.unwrap_or_else(|| BaseboardFru {
    manufacturer: String::new(),
    model: String::new(),
    product_part_number: String::new(),
    serial_number: String::new(),
    asset_tag: String::new(),
})
```

### Реальный пример: `unwrap_or_default` при десериализации конфигурации
```rust
// Пример: framework.rs
// Если разбор JSON-конфигурации не удался, используем Default — без падения
Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
```
Аналог в C++ — `try/catch` вокруг `nlohmann::json::parse()` с ручным созданием значения по умолчанию в блоке catch.

----

## Функциональные преобразования: map, map_err, find_map

- Эти методы у `Option` и `Result` позволяют преобразовывать содержащееся значение без распаковки, заменяя вложенные `if/else` линейными цепочками.

### Краткий справочник
| **Метод** | **Применяется к** | **Что делает** | **Аналог в C++** |
|-----------|-------|---------|-------------------|
| `.map(\|v\| ...)` | `Option` / `Result` | Преобразует значение `Some`/`Ok` | `if (opt) { *opt = transform(*opt); }` |
| `.map_err(\|e\| ...)` | `Result` | Преобразует значение `Err` | Добавление контекста в блок catch |
| `.and_then(\|v\| ...)` | `Option` / `Result` | Объединяет в цепочку операции, возвращающие `Option`/`Result` | Вложенные проверки if |
| `.find_map(\|v\| ...)` | Итератор | `find` + `map` за один проход | Цикл с `if + break` |
| `.filter(\|v\| ...)` | `Option` / Итератор | Оставляет только значения, подходящие под предикат | `if (!predicate) return nullopt;` |
| `.ok()?` | `Result` | Преобразует `Result → Option` и передаёт `None` дальше | `if (result.has_error()) return nullopt;` |

### Реальный пример: цепочка `.and_then()` для извлечения полей JSON
```rust
// Пример: framework.rs — поиск серийного номера с запасными вариантами
let sys_info = json.get("SystemInfo")?;

// Сначала пробуем BaseboardFru.BoardSerialNumber
if let Some(serial) = sys_info
    .get("BaseboardFru")
    .and_then(|b| b.get("BoardSerialNumber"))
    .and_then(|v| v.as_str())
    .filter(valid_serial)     // Принимаем только непустые корректные серийные номера
{
    return Some(serial.to_string());
}

// Запасной вариант: BoardFru.SerialNumber
sys_info
    .get("BoardFru")
    .and_then(|b| b.get("SerialNumber"))
    .and_then(|v| v.as_str())
    .filter(valid_serial)
    .map(|s| s.to_string())   // Преобразуем &str → String, только если Some
```
В C++ это была бы пирамида `if (json.contains("BaseboardFru")) { if (json["BaseboardFru"].contains("BoardSerialNumber")) { ... } }`.

### Реальный пример: `find_map` — поиск и преобразование за один проход
```rust
// Пример: context.rs — находим запись SDR, соответствующую датчику и владельцу
pub fn find_for_event(&self, sensor_number: u8, owner_id: u8) -> Option<&SdrRecord> {
    self.by_sensor.get(&sensor_number).and_then(|indices| {
        indices.iter().find_map(|&i| {
            let record = &self.records[i];
            if record.sensor_owner_id() == Some(owner_id) {
                Some(record)
            } else {
                None
            }
        })
    })
}
```
`find_map` — это `find` и `map` в одном: останавливается на первом совпадении и сразу преобразует его. Аналог в C++ — цикл `for` с `if` и `break`.

### Реальный пример: `map_err` для контекста ошибки
```rust
// Пример: main.rs — добавляем контекст к ошибкам перед передачей наверх
let json_str = serde_json::to_string_pretty(&config)
    .map_err(|e| format!("Failed to serialize config: {}", e))?;
```
Преобразует `serde_json::Error` в понятную ошибку `String`, которая содержит контекст о том, *что именно* не удалось.

----

## Обработка JSON: nlohmann::json → serde

- Команды C++ обычно используют `nlohmann::json` для разбора JSON. В Rust используются **serde** и **serde_json** — это мощнее, потому что схема JSON кодируется *в системе типов*.

### Сравнение C++ (nlohmann) и Rust (serde)

```cpp
// C++ с nlohmann::json — доступ к полям во время выполнения
#include <nlohmann/json.hpp>
using json = nlohmann::json;

struct Fan {
    std::string logical_id;
    std::vector<std::string> sensor_ids;
};

Fan parse_fan(const json& j) {
    Fan f;
    f.logical_id = j.at("LogicalID").get<std::string>();    // бросает исключение, если нет
    if (j.contains("SDRSensorIdHexes")) {                   // ручная обработка значения по умолчанию
        f.sensor_ids = j["SDRSensorIdHexes"].get<std::vector<std::string>>();
    }
    return f;
}
```

```rust
// Rust с serde — схема на этапе компиляции, автоматическое сопоставление полей
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fan {
    pub logical_id: String,
    #[serde(rename = "SDRSensorIdHexes", default)]  // Ключ JSON → поле Rust
    pub sensor_ids: Vec<String>,                     // Нет в JSON → пустой Vec
    #[serde(default)]
    pub sensor_names: Vec<String>,                   // Нет в JSON → пустой Vec
}

// Одна строка заменяет всю функцию разбора:
let fan: Fan = serde_json::from_str(json_str)?;
```

### Ключевые атрибуты serde (реальные примеры из продакшн-кода на Rust)

| **Атрибут** | **Назначение** | **Аналог в C++** |
|--------------|------------|--------------------|
| `#[serde(default)]` | Использовать `Default::default()` для отсутствующих полей | `if (j.contains(key)) { ... } else { default; }` |
| `#[serde(rename = "Key")]` | Сопоставить имя ключа JSON с именем поля Rust | Ручной доступ `j.at("Key")` |
| `#[serde(flatten)]` | Поглощать неизвестные ключи в `HashMap` | `for (auto& [k,v] : j.items()) { ... }` |
| `#[serde(skip)]` | Не сериализовать/десериализовать это поле | Не сохранять в JSON |
| `#[serde(tag = "type")]` | Перечисление с внутренним тегом (поле-дискриминатор) | `if (j["type"] == "gpu") { ... }` |

### Реальный пример: полная структура конфигурации
```rust
// Пример: diag.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagConfig {
    pub sku: SkuConfig,
    #[serde(default)]
    pub level: DiagLevel,            // Нет в JSON → DiagLevel::default()
    #[serde(default)]
    pub modules: ModuleConfig,       // Нет в JSON → ModuleConfig::default()
    #[serde(default)]
    pub output_dir: String,          // Нет в JSON → ""
    #[serde(default, flatten)]
    pub options: HashMap<String, serde_json::Value>,  // Поглощает неизвестные ключи
}

// Загрузка — 3 строки (против ~20+ в C++ с nlohmann):
let content = std::fs::read_to_string(path)?;
let config: DiagConfig = serde_json::from_str(&content)?;
Ok(config)
```

### Десериализация перечислений с `#[serde(tag = "type")]`
```rust
// Пример: components.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]                   // JSON: {"type": "Gpu", "product": ...}
pub enum PcieDeviceKind {
    Gpu { product: GpuProduct, manufacturer: GpuManufacturer },
    Nic { product: NicProduct, manufacturer: NicManufacturer },
    NvmeDrive { drive_type: StorageDriveType, capacity_gb: u32 },
    // ... ещё 9 вариантов
}
// serde автоматически выбирает вариант по полю "type" — без цепочки if/else вручную
```
Аналог в C++: `if (j["type"] == "Gpu") { parse_gpu(j); } else if (j["type"] == "Nic") { parse_nic(j); } ...`

# Упражнение: десериализация JSON с serde

- Определите структуру `ServerConfig`, которую можно десериализовать из следующего JSON:
```json
{
    "hostname": "diag-node-01",
    "port": 8080,
    "debug": true,
    "modules": ["accel_diag", "nic_diag", "cpu_diag"]
}
```
- Используйте `#[derive(Deserialize)]` и `serde_json::from_str()` для разбора
- Добавьте `#[serde(default)]` к полю `debug`, чтобы оно по умолчанию было `false`, если отсутствует
- **Бонус**: добавьте поле `enum DiagLevel { Quick, Full, Extended }` с `#[serde(default)]`, которое по умолчанию равно `Quick`

**Стартовый код** (нужны `cargo add serde --features derive` и `cargo add serde_json`):
```rust
use serde::Deserialize;

// TODO: Определите перечисление DiagLevel с реализацией Default

// TODO: Определите структуру ServerConfig с атрибутами serde

fn main() {
    let json_input = r#"{
        "hostname": "diag-node-01",
        "port": 8080,
        "debug": true,
        "modules": ["accel_diag", "nic_diag", "cpu_diag"]
    }"#;

    // TODO: Десериализуйте и выведите конфигурацию
    // TODO: Попробуйте разобрать JSON без поля "debug" — убедитесь, что оно по умолчанию false
}
```

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
enum DiagLevel {
    #[default]
    Quick,
    Full,
    Extended,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    hostname: String,
    port: u16,
    #[serde(default)]       // по умолчанию false, если отсутствует
    debug: bool,
    modules: Vec<String>,
    #[serde(default)]       // по умолчанию DiagLevel::Quick, если отсутствует
    level: DiagLevel,
}

fn main() {
    let json_input = r#"{
        "hostname": "diag-node-01",
        "port": 8080,
        "debug": true,
        "modules": ["accel_diag", "nic_diag", "cpu_diag"]
    }"#;

    let config: ServerConfig = serde_json::from_str(json_input)
        .expect("Failed to parse JSON");
    println!("{config:#?}");

    // Проверка с отсутствующими необязательными полями
    let minimal = r#"{
        "hostname": "node-02",
        "port": 9090,
        "modules": []
    }"#;
    let config2: ServerConfig = serde_json::from_str(minimal)
        .expect("Failed to parse minimal JSON");
    println!("debug (default): {}", config2.debug);    // false
    println!("level (default): {:?}", config2.level);  // Quick
}
// Вывод:
// ServerConfig {
//     hostname: "diag-node-01",
//     port: 8080,
//     debug: true,
//     modules: ["accel_diag", "nic_diag", "cpu_diag"],
//     level: Quick,
// }
// debug (default): false
// level (default): Quick
```

</details>

----

