# Четырнадцать проверенных на практике приёмов 🟡

> **Что вы узнаете:** четырнадцать более мелких техник корректности по построению, от устранения sentinel-значений и sealed-трейтов до сессионных типов, `Pin`, RAII и `#[must_use]`. Каждая из них устраняет конкретный класс ошибок почти без затрат.
>
> **Перекрёстные ссылки:** [гл. 02](ch02-typed-command-interfaces-request-determi.md) (sealed-трейты расширяют гл. 02), [гл. 05](ch05-protocol-state-machines-type-state-for-r.md) (билдер с typestate расширяет гл. 05), [гл. 07](ch07-validated-boundaries-parse-dont-validate.md) (FromStr расширяет гл. 07)

## Четырнадцать проверенных на практике приёмов

Восемь основных паттернов (гл. 02–09) покрывают основные техники корректности по построению. Эта глава собирает четырнадцать **более мелких, но ценных приёмов**, которые регулярно встречаются в продакшн-коде на Rust. Каждый из них устраняет конкретный класс ошибок почти без затрат.

### Приём 1 — Sentinel → `Option` на границе

Аппаратные протоколы полны sentinel-значений: IPMI использует `0xFF` для «датчик отсутствует», PCI использует `0xFFFF` для «нет устройства», а SMBIOS использует `0x00` для «неизвестно». Если протаскивать такие значения через код как обычные целые числа, каждый потребитель должен помнить о проверке магического значения. Если хотя бы одно сравнение забыто, получаем призрачное показание 255 °C или ложное совпадение ID производителя.

**Правило:** преобразуйте sentinel-значения в `Option` на самой первой границе разбора и преобразуйте их *обратно* в sentinel только на границе сериализации.

#### Антипаттерн (из `pcie_tree/src/lspci.rs`)

```rust,ignore
// Sentinel хранится внутри — каждое сравнение должно помнить о нём
let mut current_vendor_id: u16 = 0xFFFF;
let mut current_device_id: u16 = 0xFFFF;

// ... позже разбор молча завершается ошибкой ...
current_vendor_id = u16::from_str_radix(hex, 16)
    .unwrap_or(0xFFFF);  // sentinel скрывает ошибку
```

Каждая функция, которая получает `current_vendor_id`, должна знать, что `0xFFFF` — особое значение. Если кто-то напишет `if vendor_id == target_id` и не проверит сначала `0xFFFF`, отсутствующее устройство тихо совпадёт, если и цель случайно разобрана из некорректных входных данных как `0xFFFF`.

#### Правильный паттерн (из `nic_sel/src/events.rs`)

```rust,ignore
pub struct ThermalEvent {
    pub record_id: u16,
    pub temperature: Option<u8>,  // None, если датчик сообщает 0xFF
}

impl ThermalEvent {
    pub fn from_raw(record_id: u16, raw_temp: u8) -> Self {
        ThermalEvent {
            record_id,
            temperature: if raw_temp != 0xFF {
                Some(raw_temp)
            } else {
                None
            },
        }
    }
}
```

Теперь каждый потребитель *обязан* обработать случай `None`, и компилятор это обеспечивает:

```rust,ignore
// Безопасно: компилятор гарантирует обработку отсутствующих температур
fn is_overtemp(temp: Option<u8>, threshold: u8) -> bool {
    temp.map_or(false, |t| t > threshold)
}

// Забыть обработать None — ошибка компиляции:
// fn bad_check(temp: Option<u8>, threshold: u8) -> bool {
//     temp > threshold  // ОШИБКА: нельзя сравнить Option<u8> с u8
// }
```

#### Практический эффект

`inventory/src/events.rs` использует тот же паттерн для тревог о температуре GPU:
```rust,ignore
temperature: if data[1] != 0xFF {
    Some(data[1] as i8)
} else {
    None
},
```

Рефакторинг для `pcie_tree/src/lspci.rs` несложен: замените `current_vendor_id: u16` на `current_vendor_id: Option<u16>`, замените `0xFFFF` на `None` и позвольте компилятору найти каждое место, которое нужно обновить.

| До | После |
|----|-------|
| `let mut vendor_id: u16 = 0xFFFF` | `let mut vendor_id: Option<u16> = None` |
| `.unwrap_or(0xFFFF)` | `.ok()` (уже возвращает `Option`) |
| `if vendor_id != 0xFFFF { ... }` | `if let Some(vid) = vendor_id { ... }` |
| Сериализация: `vendor_id` | `vendor_id.unwrap_or(0xFFFF)` |

***

### Приём 2 — Sealed-трейты

Глава 2 ввела `IpmiCmd` с ассоциированным типом, который связывает каждую команду с её ответом. Но есть лазейка: если *любой* код может реализовать `IpmiCmd`, кто-то может написать `MaliciousCmd`, у которого `parse_response` возвращает неверный тип или паникует. Безопасность типов всей системы держится на том, что каждая реализация корректна.

**Sealed-трейт** закрывает эту лазейку. Идея проста: сделать так, чтобы трейт требовал *приватный* супертрейт, который может реализовать только ваш крейт.

```rust,ignore
// — Приватный модуль: не экспортируется из крейта —
mod private {
    pub trait Sealed {}
}

// — Публичный трейт: требует Sealed, который внешний код не может реализовать —
pub trait IpmiCmd: private::Sealed {
    type Response;
    fn net_fn(&self) -> u8;
    fn cmd_byte(&self) -> u8;
    fn payload(&self) -> Vec<u8>;
    fn parse_response(&self, raw: &[u8]) -> io::Result<Self::Response>;
}
```

Внутри крейта вы реализуете `Sealed` для каждого одобренного типа команды:

```rust,ignore
pub struct ReadTemp { pub sensor_id: u8 }
impl private::Sealed for ReadTemp {}

impl IpmiCmd for ReadTemp {
    type Response = Celsius;
    fn net_fn(&self) -> u8 { 0x04 }
    fn cmd_byte(&self) -> u8 { 0x2D }
    fn payload(&self) -> Vec<u8> { vec![self.sensor_id] }
    fn parse_response(&self, raw: &[u8]) -> io::Result<Celsius> {
        if raw.is_empty() { return Err(io::Error::new(io::ErrorKind::InvalidData, "пустой ответ")); }
        Ok(Celsius(raw[0] as f64))
    }
}
```

Внешний код видит `IpmiCmd` и может вызывать `execute()`, но не может его реализовать:

```rust,ignore
// В другом крейте:
struct EvilCmd;
// impl private::Sealed for EvilCmd {}  // ОШИБКА: модуль `private` приватный
// impl IpmiCmd for EvilCmd { ... }     // ОШИБКА: требование `Sealed` не выполнено
```

#### Когда применять sealing

| Применяйте sealing, когда… | Не применяйте, когда… |
|----------------------------|----------------------|
| Безопасность зависит от корректности реализации (IpmiCmd, DiagModule) | Пользователи должны расширять систему (свои форматировщики отчётов) |
| Ассоциированные типы должны соблюдать инварианты | Трейт — простой маркер возможности (HasIpmi) |
| У вас есть эталонный набор реализаций | Сторонние плагины — цель дизайна |

#### Кандидаты в реальном коде

- `IpmiCmd`: некорректный разбор может повредить типизированные ответы
- `DiagModule`: фреймворк предполагает, что `run()` возвращает корректные записи DER
- `SelEventFilter`: сломанный фильтр может поглотить критические события SEL

***

### Приём 3 — `#[non_exhaustive]` для развивающихся enum

`SkuVariant` в `inventory/src/types.rs` сейчас имеет пять вариантов:

```rust,ignore
pub enum SkuVariant {
    S1001, S2001, S2002, S2003, S3001,
}
```

Когда выйдет следующее поколение и вы добавите `S4001`, любой внешний код, который сопоставляет с `SkuVariant` без ветки-заглушки `_`, **тихо перестанет компилироваться**. В этом и смысл. Но что с внутренним кодом? Без `#[non_exhaustive]` ваш `match` в *том же* крейте компилируется без `_`, и добавление нового варианта ломает вашу собственную сборку.

Пометка enum как `#[non_exhaustive]` заставляет **внешние крейты**, которые сопоставляют с ним, включать ветку-заглушку. Внутри определяющего крейта `#[non_exhaustive]` не действует: исчерпывающие сопоставления по-прежнему возможны.

**Почему это полезно:** когда вы публикуете `SkuVariant` из библиотечного крейта (или общего подкрейта в рабочей области), нижележащий код вынужден обрабатывать неизвестные будущие варианты. Когда вы добавите `S4001` в следующем поколении, нижележащий код уже компилируется: у него есть ветка `_`.

```rust,ignore
// В крейте gpu_sel (определяющем крейте):
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkuVariant {
    S1001,
    S2001,
    S2002,
    S2003,
    S3001,
    // Когда выйдет следующий SKU, добавьте его сюда.
    // У внешних потребителей уже есть подстановка: для них ноль поломок.
}

// Внутри gpu_sel — исчерпывающее сопоставление разрешено (подстановка не нужна):
fn diag_path_internal(sku: SkuVariant) -> &'static str {
    match sku {
        SkuVariant::S1001 => "legacy_gen1",
        SkuVariant::S2001 => "gen2_accel_diag",
        SkuVariant::S2002 => "gen2_alt_diag",
        SkuVariant::S2003 => "gen2_alt_hf_diag",
        SkuVariant::S3001 => "gen3_accel_diag",
        // Подстановка внутри определяющего крейта не нужна.
        // Добавление S4001 сюда вызовет ошибку компиляции в этом match,
        // и это именно то, что нужно: это заставит обновить код.
    }
}
```

```rust,ignore
// В бинарном крейте (нижележащем крейте, который зависит от inventory):
fn diag_path_external(sku: inventory::SkuVariant) -> &'static str {
    match sku {
        inventory::SkuVariant::S1001 => "legacy_gen1",
        inventory::SkuVariant::S2001 => "gen2_accel_diag",
        inventory::SkuVariant::S2002 => "gen2_alt_diag",
        inventory::SkuVariant::S2003 => "gen2_alt_hf_diag",
        inventory::SkuVariant::S3001 => "gen3_accel_diag",
        _ => "generic_diag",  // ОБЯЗАТЕЛЬНО для внешних крейтов из-за #[non_exhaustive]
    }
}
```

> **Совет по рабочей области:** если весь код находится в одном крейте, `#[non_exhaustive]` не поможет: он действует только на границах между крейтами. Для большой рабочей области проекта размещайте развивающиеся enum в общем крейте (`core_lib` или `inventory`), чтобы атрибут защищал потребителей в других крейтах рабочей области.

#### Кандидаты

| Enum | Модуль | Почему |
|------|--------|--------|
| `SkuVariant` | `inventory`, `net_inventory` | Новые SKU в каждом поколении |
| `SensorType` | `protocol_lib` | Спецификация IPMI резервирует 0xC0–0xFF для OEM |
| `CompletionCode` | `protocol_lib` | Производители BMC добавляют собственные коды |
| `Component` | `event_handler` | Новые категории оборудования (NewSoC был добавлен недавно) |

***

### Приём 4 — Билдер с typestate

Глава 5 показала typestate для *протоколов* (жизненные циклы сессий, обучение линка). Та же идея применима к *билдерам*: структурам, у которых `build()` или `finish()` можно вызвать только тогда, когда установлены все обязательные поля.

#### Проблема текучих (fluent) билдеров

`DerBuilder` в `diag_framework/src/der.rs` сейчас выглядит так (упрощённо):

```rust,ignore
// Текущий текучий билдер: finish() доступен всегда
pub struct DerBuilder {
    der: Der,
}

impl DerBuilder {
    pub fn new(marker: &str, fault_code: u32) -> Self { ... }
    pub fn mnemonic(mut self, m: &str) -> Self { ... }
    pub fn fault_class(mut self, fc: &str) -> Self { ... }
    pub fn finish(self) -> Der { self.der }  // ← доступен всегда!
}
```

Это компилируется без ошибок, но создаёт неполную запись DER:

```rust,ignore
let bad = DerBuilder::new("CSI_ERR", 62691)
    .finish();  // упс: нет mnemonic и fault_class
```

#### Билдер с typestate: `finish()` требует оба поля

```rust,ignore
pub struct Missing;
pub struct Set<T>(T);

pub struct DerBuilder<Mnemonic, FaultClass> {
    marker: String,
    fault_code: u32,
    mnemonic: Mnemonic,
    fault_class: FaultClass,
    description: Option<String>,
}

// Конструктор: начинаем с обоими обязательными полями в состоянии Missing
impl DerBuilder<Missing, Missing> {
    pub fn new(marker: &str, fault_code: u32) -> Self {
        DerBuilder {
            marker: marker.to_string(),
            fault_code,
            mnemonic: Missing,
            fault_class: Missing,
            description: None,
        }
    }
}

// Устанавливаем mnemonic (работает независимо от состояния fault_class)
impl<FC> DerBuilder<Missing, FC> {
    pub fn mnemonic(self, m: &str) -> DerBuilder<Set<String>, FC> {
        DerBuilder {
            marker: self.marker, fault_code: self.fault_code,
            mnemonic: Set(m.to_string()),
            fault_class: self.fault_class,
            description: self.description,
        }
    }
}

// Устанавливаем fault_class (работает независимо от состояния mnemonic)
impl<MN> DerBuilder<MN, Missing> {
    pub fn fault_class(self, fc: &str) -> DerBuilder<MN, Set<String>> {
        DerBuilder {
            marker: self.marker, fault_code: self.fault_code,
            mnemonic: self.mnemonic,
            fault_class: Set(fc.to_string()),
            description: self.description,
        }
    }
}

// Необязательные поля доступны в ЛЮБОМ состоянии
impl<MN, FC> DerBuilder<MN, FC> {
    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }
}

/// Полностью собранная запись DER.
pub struct Der {
    pub marker: String,
    pub fault_code: u32,
    pub mnemonic: String,
    pub fault_class: String,
    pub description: Option<String>,
}

// finish() доступен ТОЛЬКО тогда, когда оба обязательных поля в состоянии Set
impl DerBuilder<Set<String>, Set<String>> {
    pub fn finish(self) -> Der {
        Der {
            marker: self.marker,
            fault_code: self.fault_code,
            mnemonic: self.mnemonic.0,
            fault_class: self.fault_class.0,
            description: self.description,
        }
    }
}
```

Теперь ошибочный вызов становится ошибкой компиляции:

```rust,ignore
// ✅ Компилируется: оба обязательных поля установлены (порядок не важен)
let der = DerBuilder::new("CSI_ERR", 62691)
    .fault_class("GPU Module")   // порядок не важен
    .mnemonic("ACCEL_CARD_ER691")
    .description("Thermal throttle")
    .finish();

// ❌ Ошибка компиляции: finish() не существует у DerBuilder<Set<String>, Missing>
let bad = DerBuilder::new("CSI_ERR", 62691)
    .mnemonic("ACCEL_CARD_ER691")
    .finish();  // ОШИБКА: метод `finish` не найден
```

#### Когда использовать билдеры с typestate

| Используйте, когда… | Не стоит, когда… |
|---------------------|------------------|
| Пропуск поля приводит к тихим ошибкам (в DER пропущен mnemonic) | Все поля имеют разумные значения по умолчанию |
| Билдер — часть публичного API | Билдер — только тестовая обвязка |
| Более 2–3 обязательных полей | Одно обязательное поле (просто передайте его в `new()`) |

***

### Приём 5 — `FromStr` как граница валидации

Глава 7 показала `TryFrom<&[u8]>` для бинарных данных (записи FRU, записи SEL). Для **строковых** входных данных (файлов конфигурации, аргументов CLI, полей JSON) аналогичной границей служит `FromStr`.

#### Проблема

```rust,ignore
// C++ / Rust без валидации: тихо падает в значение по умолчанию
fn route_diag(level: &str) -> DiagMode {
    if level == "quick" { ... }
    else if level == "standard" { ... }
    else { QuickMode }  // опечатка в конфиге?  ¯\_(ツ)_/¯
}
```

Файл конфигурации с `"diag_level": "extendedd"` (опечатка) тихо получит `QuickMode`.

#### Паттерн (из `config_loader/src/diag.rs`)

```rust,ignore
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
    Quick,
    Standard,
    Extended,
    Stress,
}

impl FromStr for DiagLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quick"    | "1" => Ok(DiagLevel::Quick),
            "standard" | "2" => Ok(DiagLevel::Standard),
            "extended" | "3" => Ok(DiagLevel::Extended),
            "stress"   | "4" => Ok(DiagLevel::Stress),
            other => Err(format!("неизвестный уровень диагностики: '{other}'")),
        }
    }
}
```

Теперь опечатка ловится сразу:

```rust,ignore
let level: DiagLevel = "extendedd".parse()?;
// Err("неизвестный уровень диагностики: 'extendedd'")
```

#### Три преимущества

1. **Быстрый отказ:** некорректные входные данные ловятся на границе разбора, а не на три уровня глубже в логике диагностики.
2. **Синонимы явные:** `"MEM"`, `"DIMM"` и `"MEMORY"` все отображаются в `Component::Memory`: ветки `match` документируют соответствие.
3. **`.parse()` удобен:** поскольку `FromStr` интегрируется с `str::parse()`, получаются лаконичные однострочники: `let level: DiagLevel = config["level"].parse()?;`

#### Использование в реальном коде

В проекте уже есть 8 реализаций `FromStr`:

| Тип | Модуль | Заметные синонимы |
|-----|--------|-------------------|
| `DiagLevel` | `config_loader` | `"1"` = быстрый, `"4"` = стресс |
| `Component` | `event_handler` | `"MEM"` / `"DIMM"` = память, `"SSD"` / `"NVME"` = диск |
| `SkuVariant` | `net_inventory` | `"Accel-X1"` = S2001, `"Accel-M1"` = S2002, `"Accel-Z1"` = S3001 |
| `SkuVariant` | `inventory` | Те же синонимы (отдельный модуль, тот же паттерн) |
| `FaultStatus` | `config_loader` | Состояния жизненного цикла неисправности |
| `DiagAction` | `config_loader` | Типы действий по устранению |
| `ActionType` | `config_loader` | Категории действий |
| `DiagMode` | `cluster_diag` | Режимы тестирования нескольких узлов |

Сравнение с `TryFrom`:

| | `TryFrom<&[u8]>` | `FromStr` |
|---|---|---|
| Вход | Сырые байты (бинарные протоколы) | Строки (конфиги, CLI, JSON) |
| Типичный источник | IPMI, пространство конфигурации PCIe, FRU | Поля JSON, переменные окружения, ввод пользователя |
| Глава | гл. 07 | гл. 11 |
| Оба используют | `Result`: заставляет вызывающего обрабатывать некорректный ввод |

***

### Приём 6 — const-обобщения для проверки размера на этапе компиляции

Когда аппаратные буферы, банки регистров или кадры протокола имеют фиксированный размер, const-обобщения позволяют компилятору его проверить:

```rust,ignore
/// Банк регистров фиксированного размера. Размер — часть типа.
/// `RegisterBank<256>` и `RegisterBank<4096>` — разные типы.
pub struct RegisterBank<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> RegisterBank<N> {
    /// Читает регистр по заданному смещению.
    /// Время компиляции: N известно, поэтому размер массива фиксирован.
    /// Время выполнения: проверяется только смещение.
    pub fn read(&self, offset: usize) -> Option<u8> {
        self.data.get(offset).copied()
    }
}

// Конфигурационное пространство PCIe (обычное): 256 байт
type PciConfigSpace = RegisterBank<256>;

// Расширенное конфигурационное пространство PCIe: 4096 байт
type PcieExtConfigSpace = RegisterBank<4096>;

// Это разные типы: нельзя случайно передать один вместо другого:
fn read_extended_cap(config: &PcieExtConfigSpace, offset: usize) -> Option<u8> {
    config.read(offset)
}
// read_extended_cap(&pci_config, 0x100);
//                   ^^^^^^^^^^^ expected RegisterBank<4096>, found RegisterBank<256> ❌
```

**Утверждения на этапе компиляции с const-обобщениями:**

```rust,ignore
/// Административные команды NVMe используют буферы по 4096 байт. Проверяем на этапе компиляции.
pub struct NvmeBuffer<const N: usize> {
    data: Box<[u8; N]>,
}

impl<const N: usize> NvmeBuffer<N> {
    pub fn new() -> Self {
        // Проверка во время выполнения: разрешены только 512 или 4096
        assert!(N == 4096 || N == 512, "Буферы NVMe должны быть 512 или 4096 байт");
        NvmeBuffer { data: Box::new([0u8; N]) }
    }
}
// NvmeBuffer::<1024>::new();  // в такой форме паникует во время выполнения
// Для настоящей проверки на этапе компиляции см. приём 9 (const-утверждения).
```

> **Когда использовать:** фиксированные протокольные буферы (NVMe, конфигурационное пространство PCIe), дескрипторы DMA, глубины аппаратных FIFO. Везде, где размер — аппаратная константа, которая никогда не должна меняться во время выполнения.

***

### Приём 7 — Безопасные обёртки вокруг `unsafe`

В проекте сейчас нет ни одного блока `unsafe`. Но когда вы добавите доступ к регистрам MMIO, DMA или FFI в accel-mgmt или accel-query, `unsafe` понадобится. Корректный по построению подход: **обернуть каждый блок `unsafe` в безопасную абстракцию**, чтобы небезопасность была локализована и поддавалась аудиту.

```rust,ignore
/// Регистр, отображённый в MMIO. Указатель действителен, пока существует отображение.
/// Весь unsafe заключён в этом модуле: вызывающий код использует безопасные методы.
pub struct MmioRegion {
    base: *mut u8,
    len: usize,
}

impl MmioRegion {
    /// # Safety
    /// - `base` должен быть корректным указателем на область, отображённую в MMIO
    /// - Область должна оставаться отображённой, пока существует эта структура
    /// - Никакой другой код не должен алиасить эту область
    pub unsafe fn new(base: *mut u8, len: usize) -> Self {
        MmioRegion { base, len }
    }

    /// Безопасное чтение: проверка границ предотвращает выход за пределы MMIO.
    pub fn read_u32(&self, offset: usize) -> Option<u32> {
        if offset + 4 > self.len { return None; }
        // SAFETY: offset проверен выше, base корректен согласно контракту new()
        Some(unsafe {
            core::ptr::read_volatile(self.base.add(offset) as *const u32)
        })
    }

    /// Безопасная запись: проверка границ предотвращает выход за пределы MMIO.
    pub fn write_u32(&self, offset: usize, value: u32) -> bool {
        if offset + 4 > self.len { return false; }
        // SAFETY: offset проверен выше, base корректен согласно контракту new()
        unsafe {
            core::ptr::write_volatile(self.base.add(offset) as *mut u32, value);
        }
        true
    }
}
```

**Комбинируем с phantom-типами (гл. 09) для типизированного MMIO:**

```rust,ignore
use std::marker::PhantomData;

pub struct ReadOnly;
pub struct ReadWrite;

pub struct TypedMmio<Perm> {
    region: MmioRegion,
    _perm: PhantomData<Perm>,
}

impl TypedMmio<ReadOnly> {
    pub fn read_u32(&self, offset: usize) -> Option<u32> {
        self.region.read_u32(offset)
    }
    // Нет метода записи: ошибка компиляции при попытке записи в область ReadOnly
}

impl TypedMmio<ReadWrite> {
    pub fn read_u32(&self, offset: usize) -> Option<u32> {
        self.region.read_u32(offset)
    }
    pub fn write_u32(&self, offset: usize, value: u32) -> bool {
        self.region.write_u32(offset, value)
    }
}
```

> **Правила для обёрток над `unsafe`:**
>
> | Правило | Зачем |
> |---------|-------|
> | Один `unsafe fn new()` с задокументированными инвариантами `# Safety` | Вызывающий берёт ответственность один раз |
> | Все остальные методы безопасны | Вызывающие не могут вызвать UB |
> | Комментарий `# SAFETY:` у каждого блока `unsafe` | Аудиторы могут проверить локально |
> | Обернуть в модуль с `#[deny(unsafe_op_in_unsafe_fn)]` | Даже внутри `unsafe fn` отдельные операции требуют `unsafe` |
> | Запустить `cargo +nightly miri test` на обёртке | Проверка соответствия модели памяти |

---

### ✅ Контрольная точка: приёмы 1–7

Теперь у вас есть семь повседневных приёмов. Вот краткая сводка:

| Приём | Устраняемый класс ошибок | Усилия на внедрение |
|:-----:|--------------------------|:-------------------:|
| 1 | Путаница sentinel-значений (0xFF) | Низкие: один `match` на границе |
| 2 | Несанкционированные реализации трейтов | Низкие: добавить супертрейт `Sealed` |
| 3 | Поломка потребителей при росте enum | Низкие: одна строка-атрибут |
| 4 | Пропущенные поля билдера | Средние: дополнительные параметры типа |
| 5 | Опечатки в строковых конфигурациях | Низкие: `impl FromStr` |
| 6 | Неверные размеры буферов | Низкие: параметр const-обобщения |
| 7 | Небезопасный код, разбросанный по кодовой базе | Средние: модуль-обёртка |

Приёмы 8–14 — **более продвинутые**: они затрагивают async, вычисления на этапе компиляции, сессионные типы, `Pin` и `Drop`. Сделайте паузу, если нужно: приёмы выше уже дают высокую отдачу при низких затратах, и их можно внедрить хоть завтра.

***

### Приём 8 — Автоматы состояний в async-коде

Когда драйверы аппаратуры используют `async` (например, асинхронная связь с BMC или асинхронный ввод-вывод NVMe), typestate по-прежнему работает, но владение через точки `.await` требует внимания:

```rust,ignore
use std::marker::PhantomData;

pub struct Idle;
pub struct Authenticating;
pub struct Active;

pub struct AsyncSession<S> {
    host: String,
    _state: PhantomData<S>,
}

impl AsyncSession<Idle> {
    pub fn new(host: &str) -> Self {
        AsyncSession { host: host.to_string(), _state: PhantomData }
    }

    /// Переход Idle → Authenticating → Active.
    /// Сессия потребляется (перемещается в future) через .await.
    pub async fn authenticate(self, user: &str, pass: &str)
        -> Result<AsyncSession<Active>, String>
    {
        // Фаза 1: отправляем учётные данные (потребляет сессию Idle)
        let pending: AsyncSession<Authenticating> = AsyncSession {
            host: self.host,
            _state: PhantomData,
        };

        // Имитация асинхронной аутентификации в BMC
        // tokio::time::sleep(Duration::from_secs(1)).await;

        // Фаза 2: возвращаем активную сессию
        Ok(AsyncSession {
            host: pending.host,
            _state: PhantomData,
        })
    }
}

impl AsyncSession<Active> {
    pub async fn send_command(&mut self, cmd: &[u8]) -> Vec<u8> {
        // асинхронный ввод-вывод здесь...
        vec![0x00]
    }
}

// Использование:
// let session = AsyncSession::new("192.168.1.100");
// let mut session = session.authenticate("admin", "pass").await?;
// let resp = session.send_command(&[0x04, 0x2D]).await;
```

**Ключевые правила для async typestate:**

| Правило | Зачем |
|---------|-------|
| Методы перехода принимают `self` (по значению), а не `&mut self` | Передача владения работает через `.await` |
| Возвращать `Result<NextState, (Error, PrevState)>` для восстанимых ошибок | Вызывающий может повторить попытку с предыдущего состояния |
| Не разделять состояние между несколькими future | Один future владеет одной сессией |
| Использовать ограничения `Send + 'static`, если применяется `tokio::spawn` | Сессия должна перемещаться между потоками |

> **Оговорка:** если при ошибке нужно вернуть *предыдущее* состояние (чтобы повторить попытку), верните `Result<AsyncSession<Active>, (Error, AsyncSession<Idle>)>`, чтобы вызывающий получил владение обратно. Без этого неудачный `.await` навсегда уничтожает сессию.

***

### Приём 9 — Уточняющие типы через const-утверждения

Когда числовое ограничение является инвариантом времени компиляции (а не данными времени выполнения), используйте вычисления на этапе компиляции (`const`), чтобы его обеспечить. Это отличается от приёма 6, который задаёт различия размеров на уровне типов: здесь мы *отвергаем недопустимые значения* на этапе компиляции:

```rust,ignore
/// ID датчика, который должен находиться в диапазоне SDR IPMI (0x01..=0xFE).
/// Ограничение проверяется на этапе компиляции, когда `N` — константа.
pub struct SdrSensorId<const N: u8>;

impl<const N: u8> SdrSensorId<N> {
    /// Проверка на этапе компиляции: паникует во время компиляции, если N вне диапазона.
    pub const fn validate() {
        assert!(N >= 0x01, "ID датчика должен быть >= 0x01");
        assert!(N <= 0xFE, "ID датчика должен быть <= 0xFE (0xFF зарезервирован)");
    }

    pub const VALIDATED: () = Self::validate();

    pub const fn value() -> u8 { N }
}

// Использование:
fn read_sensor_const<const N: u8>() -> f64 {
    let _ = SdrSensorId::<N>::VALIDATED;  // проверка на этапе компиляции
    // читаем датчик N...
    42.0
}

// read_sensor_const::<0x20>();   // ✅ компилируется: 0x20 допустим
// read_sensor_const::<0x00>();   // ❌ ошибка компиляции: "ID датчика должен быть >= 0x01"
// read_sensor_const::<0xFF>();   // ❌ ошибка компиляции: 0xFF зарезервирован
```

**Упрощённый вариант: ограниченные ID вентиляторов:**

```rust,ignore
pub struct BoundedFanId<const N: u8>;

impl<const N: u8> BoundedFanId<N> {
    pub const VALIDATED: () = assert!(N < 8, "Сервер поддерживает не более 8 вентиляторов (0..7)");

    pub const fn id() -> u8 {
        let _ = Self::VALIDATED;
        N
    }
}

// BoundedFanId::<3>::id();   // ✅
// BoundedFanId::<10>::id();  // ❌ ошибка компиляции
```

> **Когда использовать:** аппаратно заданные фиксированные ID (ID датчиков, слоты вентиляторов, номера слотов PCIe), известные на этапе компиляции. Если значение приходит из данных времени выполнения (файл конфигурации, ввод пользователя), используйте `TryFrom` / `FromStr` (гл. 07, приём 5).

***

### Приём 10 — Сессионные типы для обмена по каналам

Когда два компонента обмениваются сообщениями по каналу (например, оркестратор диагностики и рабочий поток), **сессионные типы** кодируют протокол в системе типов:

```rust,ignore
use std::marker::PhantomData;

// Протокол: клиент отправляет Request, сервер отправляет Response, затем завершение.
pub struct SendRequest;
pub struct RecvResponse;
pub struct Done;

/// Типизированный конец канала. `S` — текущее состояние протокола.
pub struct Chan<S> {
    // В реальном коде: оборачивает пару mpsc::Sender/Receiver
    _state: PhantomData<S>,
}

impl Chan<SendRequest> {
    /// Отправить запрос: переход в состояние RecvResponse.
    pub fn send(self, request: DiagRequest) -> Chan<RecvResponse> {
        // ... отправка в канал ...
        Chan { _state: PhantomData }
    }
}

impl Chan<RecvResponse> {
    /// Получить ответ: переход в состояние Done.
    pub fn recv(self) -> (DiagResponse, Chan<Done>) {
        // ... получение из канала ...
        (DiagResponse { passed: true }, Chan { _state: PhantomData })
    }
}

impl Chan<Done> {
    /// Закрытие канала: возможно только когда протокол завершён.
    pub fn close(self) { /* drop */ }
}

pub struct DiagRequest { pub test_name: String }
pub struct DiagResponse { pub passed: bool }

// Протокол ОБЯЗАН соблюдаться по порядку:
fn orchestrator(chan: Chan<SendRequest>) {
    let chan = chan.send(DiagRequest { test_name: "gpu_stress".into() });
    let (response, chan) = chan.recv();
    chan.close();
    println!("Результат: {}", if response.passed { "PASS" } else { "FAIL" });
}

// Нельзя получить ответ до отправки:
// fn wrong_order(chan: Chan<SendRequest>) {
//     chan.recv();  // ❌ нет метода `recv` у Chan<SendRequest>
// }
```

> **Когда использовать:** межпоточные диагностические протоколы, последовательности команд BMC, любой паттерн «запрос — ответ», где важен порядок. Для сложных многосообщенческих протоколов рассмотрите крейты [`session-types`](https://crates.io/crates/session-types) или [`rumpsteak`](https://crates.io/crates/rumpsteak).

***

### Приём 11 — `Pin` для самоссылочных автоматов состояний

Некоторым автоматам состояний нужно хранить ссылки на собственные данные (например, парсеру, который отслеживает позицию внутри собственного буфера). Rust обычно запрещает это, потому что перемещение структуры сделает внутренний указатель недействительным. `Pin<T>` решает эту проблему, гарантируя, что значение **не будет перемещено**:

```rust,ignore
use std::pin::Pin;
use std::marker::PhantomPinned;

/// Потоковый парсер, который хранит ссылку на собственный буфер.
/// После закрепления его нельзя переместить: внутренняя ссылка остаётся действительной.
pub struct StreamParser {
    buffer: Vec<u8>,
    /// Указывает внутрь `buffer`. Действительна только пока парсер закреплён.
    cursor: *const u8,
    _pin: PhantomPinned,  // отказываемся от Unpin: предотвращает случайное открепление
}

impl StreamParser {
    pub fn new(data: Vec<u8>) -> Pin<Box<Self>> {
        let parser = StreamParser {
            buffer: data,
            cursor: std::ptr::null(),
            _pin: PhantomPinned,
        };
        let mut boxed = Box::pin(parser);

        // Устанавливаем cursor на закреплённый буфер
        let cursor = boxed.buffer.as_ptr();
        // SAFETY: у нас эксклюзивный доступ, и парсер закреплён
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).cursor = cursor;
        }

        boxed
    }

    /// Читает следующий байт: вызывается только через Pin<&mut Self>.
    pub fn next_byte(self: Pin<&mut Self>) -> Option<u8> {
        // Парсер нельзя переместить, поэтому cursor остаётся действительным
        if self.cursor.is_null() { return None; }
        // ... продвигаем cursor по буферу ...
        Some(42) // заглушка
    }
}

// Использование:
// let mut parser = StreamParser::new(vec![0x01, 0x02, 0x03]);
// let byte = parser.as_mut().next_byte();
```

**Ключевая идея:** `Pin` — это корректное по построению решение проблемы самоссылочных структур. Без него понадобился бы `unsafe` и ручное отслеживание времён жизни. С ним компилятор запрещает перемещения, и инвариант внутреннего указателя сохраняется.

| Используйте `Pin`, когда… | Не используйте `Pin`, когда… |
|---------------------------|------------------------------|
| Автомат состояний хранит ссылки внутри структуры | Все поля независимо принадлежат структуре |
| Async-future заимствуют данные через `.await` | Самоссылки не нужны |
| Дескрипторы DMA, которые не должны перемещаться в памяти | Данные можно свободно перемещать |
| Аппаратные кольцевые буферы с внутренним курсором | Простой итерации по индексу достаточно |

***

### Приём 12 — RAII / `Drop` как гарантия корректности

Трейт `Drop` в Rust — это механизм корректности по построению: код очистки **нельзя забыть**, потому что компилятор вставляет его автоматически. Это особенно ценно для аппаратных ресурсов, которые нужно освободить ровно один раз.

```rust,ignore
use std::io;

/// Сессия IPMI, которую ОБЯЗАТЕЛЬНО нужно закрыть по завершении.
/// Реализация `Drop` гарантирует очистку даже при панике или раннем возврате через `?`.
pub struct IpmiSession {
    handle: u32,
}

impl IpmiSession {
    pub fn open(host: &str) -> io::Result<Self> {
        // ... согласовываем сессию IPMI ...
        Ok(IpmiSession { handle: 42 })
    }

    pub fn send_raw(&self, _data: &[u8]) -> io::Result<Vec<u8>> {
        Ok(vec![0x00])
    }
}

impl Drop for IpmiSession {
    fn drop(&mut self) {
        // Команда Close Session: выполняется всегда, даже при панике или раннем возврате.
        // В C забытый CloseSession() оставляет слот сессии BMC занятым.
        let _ = self.send_raw(&[0x06, 0x3C]);
        eprintln!("[RAII] сессия {} закрыта", self.handle);
    }
}
// Использование:
fn diagnose(host: &str) -> io::Result<()> {
    let session = IpmiSession::open(host)?;
    session.send_raw(&[0x04, 0x2D, 0x20])?;
    // Явное закрытие не нужно: Drop выполнится здесь автоматически
    Ok(())
    // Даже если send_raw вернёт Err(...), сессия всё равно закроется.
}
```

**Сбой C/C++, который устраняет RAII:**

```text
C:     session = ipmi_open(host);
       ipmi_send(session, data);
       if (error) return -1;        // 🐛 утечка сессии: забыли close()
       ipmi_close(session);

Rust:  let session = IpmiSession::open(host)?;
       session.send_raw(data)?;     // ✅ Drop выполняется при раннем возврате через ?
       // Drop всегда выполняется: утечка невозможна
```

**Комбинируем RAII с typestate (гл. 05) для упорядоченной очистки:**

Нельзя специализировать `Drop` по параметру обобщённого типа (ошибка Rust E0366). Вместо этого используйте **отдельные типы-обёртки для каждого состояния**:

```rust,ignore
use std::marker::PhantomData;

pub struct Open;
pub struct Locked;

pub struct GpuContext<S> {
    device_id: u32,
    _state: PhantomData<S>,
}

impl GpuContext<Open> {
    pub fn lock_clocks(self) -> LockedGpu {
        // ... блокируем частоты GPU для стабильных замеров ...
        LockedGpu { device_id: self.device_id }
    }
}

/// Отдельный тип для заблокированного состояния: имеет собственный Drop.
/// Нельзя написать `impl Drop for GpuContext<Locked>` (E0366),
/// поэтому используем отдельную обёртку, которая владеет заблокированным ресурсом.
pub struct LockedGpu {
    device_id: u32,
}

impl LockedGpu {
    pub fn run_benchmark(&self) -> f64 {
        // ... бенчмарк с заблокированными частотами ...
        42.0
    }
}

impl Drop for LockedGpu {
    fn drop(&mut self) {
        // Разблокируем частоты при Drop: срабатывает только для заблокированной обёртки.
        eprintln!("[RAII] частоты GPU {} разблокированы", self.device_id);
    }
}

// GpuContext<Open> не имеет собственного Drop: разблокировать нечего.
// LockedGpu всегда разблокирует частоты при Drop, даже при панике или раннем возврате.
```

> **Почему не `impl Drop for GpuContext<Locked>`?** Rust требует, чтобы реализации `Drop` применялись ко *всем* инстанциям обобщённого типа. Чтобы получить очистку, зависящую от состояния, используйте один из вариантов:
>
> | Подход | Плюсы | Минусы |
> |--------|-------|--------|
> | Отдельный тип-обёртка (выше) | Чисто, без накладных расходов | Дополнительное имя типа |
> | Обобщённый `Drop` и проверка `TypeId` во время выполнения | Один тип | Требует `'static`, накладные расходы во время выполнения |
> | Состояние в виде `enum` с исчерпывающим `match` в `Drop` | Один обобщённый тип | Диспетчеризация во время выполнения, меньше безопасности типов |

> **Когда использовать:** сессии BMC, блокировки частот GPU, отображения буферов DMA, файловые дескрипторы, охранники мьютексов (mutex guards), любой ресурс с обязательным шагом освобождения. Если вы пишете `fn close(&mut self)` или `fn cleanup()`, почти наверняка это должен быть `Drop`.

***

### Приём 13 — Иерархии типов ошибок как инструмент корректности

Хорошо спроектированные типы ошибок предотвращают тихое проглатывание ошибок и заставляют вызывающий код корректно обрабатывать каждый режим отказа. Использование `thiserror` для структурированных ошибок — паттерн корректности по построению: компилятор требует исчерпывающего сопоставления.

```toml
# Cargo.toml
[dependencies]
thiserror = "1"
# Для обработки ошибок на уровне приложения (необязательно):
# anyhow = "1"
```

```rust,ignore
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiagError {
    #[error("сбой связи по IPMI: {0}")]
    Ipmi(#[from] IpmiError),

    #[error("показание датчика {sensor_id:#04x} вне диапазона: {value}")]
    SensorRange { sensor_id: u8, value: f64 },

    #[error("GPU {gpu_id} не отвечает")]
    GpuTimeout { gpu_id: u32 },

    #[error("некорректная конфигурация: {0}")]
    Config(String),
}

#[derive(Debug, Error)]
pub enum IpmiError {
    #[error("аутентификация сессии не пройдена")]
    AuthFailed,

    #[error("команда {net_fn:#04x}/{cmd:#04x}: истекло время ожидания")]
    Timeout { net_fn: u8, cmd: u8 },

    #[error("код завершения {0:#04x}")]
    CompletionCode(u8),
}

// Вызывающий ОБЯЗАН обработать каждый вариант: никакого тихого проглатывания
fn run_thermal_check() -> Result<(), DiagError> {
    // Если здесь возвращается IpmiError, он автоматически преобразуется в DiagError::Ipmi
    // через атрибут #[from].
    let temp = read_cpu_temp()?;
    if temp > 105.0 {
        return Err(DiagError::SensorRange {
            sensor_id: 0x20,
            value: temp,
        });
    }
    Ok(())
}

# fn read_cpu_temp() -> Result<f64, DiagError> { Ok(42.0) }
```

**Почему это корректно по построению:**

| Без структурированных ошибок | С enum из `thiserror` |
|------------------------------|----------------------|
| `fn op() -> Result<T, String>` | `fn op() -> Result<T, DiagError>` |
| Вызывающий получает непрозрачную строку | Вызывающий сопоставляет конкретные варианты |
| Нельзя отличить сбой аутентификации от тайм-аута | `DiagError::Ipmi(IpmiError::AuthFailed)` и `Timeout` |
| Логирование проглатывает ошибку | `match` заставляет обработать каждый случай |
| Новый вариант ошибки: никто не замечает | Новый вариант: компилятор предупреждает о непокрытых ветках |

**Выбор между `anyhow` и `thiserror`:**

| Используйте `thiserror`, когда… | Используйте `anyhow`, когда… |
|---------------------------------|------------------------------|
| Пишете библиотеку или крейт | Пишете бинарник или CLI |
| Вызывающим нужно сопоставлять варианты ошибок | Вызывающие просто логируют и завершаются |
| Типы ошибок — часть публичного API | Внутренняя обвязка ошибок |
| `protocol_lib`, `accel_diag`, `thermal_diag` | Главный бинарник `diag_tool` |

> **Когда использовать:** каждый крейт в рабочей области должен определять собственный enum ошибок с `thiserror`. Верхнеуровневый бинарный крейт может использовать `anyhow`, чтобы объединить их. Это даёт вызывающим библиотеки гарантии обработки ошибок на этапе компиляции, сохраняя при этом удобство бинарника.

***

### Приём 14 — `#[must_use]` для обязательного использования

Атрибут `#[must_use]` превращает проигнорированные возвращаемые значения в предупреждения компилятора. Это лёгкий инструмент корректности по построению, который сочетается с каждым паттерном этого руководства:

```rust,ignore
/// Токен калибровки, который ОБЯЗАН быть использован: тихое отбрасывание — ошибка.
#[must_use = "токен калибровки нужно передать в calibrate(), а не отбрасывать"]
pub struct CalibrationToken {
    _private: (),
}

/// Результат диагностики, который ОБЯЗАН быть проверен: игнорирование сбоев — ошибка.
#[must_use = "результат диагностики нужно проверить на наличие сбоев"]
pub struct DiagResult {
    pub passed: bool,
    pub details: String,
}

/// Функции, возвращающие важные значения, тоже стоит пометить:
#[must_use = "аутентифицированную сессию нужно использовать или явно закрыть"]
pub fn authenticate(user: &str, pass: &str) -> Result<Session, AuthError> {
    // ...
#   unimplemented!()
}
#
# pub struct Session;
# pub struct AuthError;
```

**Что сообщает компилятор:**

```text
warning: unused `CalibrationToken` that must be used
  --> src/main.rs:5:5
   |
5  |     CalibrationToken { _private: () };
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: токен калибровки нужно передать в calibrate(), а не отбрасывать
```

**Применяйте `#[must_use]` к этим паттернам:**

| Паттерн | Что аннотировать | Почему |
|---------|------------------|--------|
| Одноразовые токены (гл. 03) | `CalibrationToken`, `FusePayload` | Отбрасывание без использования: логическая ошибка |
| Capability-токены (гл. 04) | `AdminToken` | Аутентификация, но токен игнорируется |
| Переходы typestate | Возвращаемый тип `authenticate()`, `activate()` | Сессия создана, но не используется |
| Результаты | `DiagResult`, `SensorReading` | Тихое проглатывание сбоев |
| Обработчики RAII (приём 12) | `IpmiSession`, `LockedGpu` | Ресурс открыт, но не используется |

> **Эмпирическое правило:** если отбрасывание значения без использования всегда является ошибкой, добавьте `#[must_use]`. Если это иногда намеренно (например, для `Vec`), не добавляйте. Префикс `_` (`let _ = foo()`) явно подтверждает отбрасывание и отключает предупреждение: это нормально, когда отбрасывание намеренное.

## Ключевые выводы

1. **Sentinel → Option на границе**: преобразуйте магические значения в `Option` при разборе; компилятор заставит вызывающих обрабатывать `None`.
2. **Sealed-трейты закрывают лазейку в реализациях**: приватный супертрейт означает, что реализовать трейт может только ваш крейт.
3. **`#[non_exhaustive]` и `#[must_use]` — однострочные аннотации с высокой отдачей**: добавляйте их к развивающимся enum и потребляемым токенам.
4. **Билдеры с typestate обеспечивают обязательные поля**: `finish()` существует только тогда, когда все обязательные параметры типа в состоянии `Set`.
5. **Каждый приём нацелен на конкретный класс ошибок**: внедряйте их постепенно; ни один приём не требует переписывать архитектуру.

---
