# Упражнения 🟡

> **Что вы узнаете:** практику применения паттернов корректности по построению к реалистичным сценариям аппаратного обеспечения: административные команды NVMe, автоматы состояний обновления прошивки, конвейеры обработки показаний датчиков, phantom-типы для PCIe, проверки здоровья по нескольким протоколам и диагностические протоколы с сессионными типами.
>
> **Перекрёстные ссылки:** [гл. 02](ch02-typed-command-interfaces-request-determi.md) (упражнение 1), [гл. 05](ch05-protocol-state-machines-type-state-for-r.md) (упражнение 2), [гл. 06](ch06-dimensional-analysis-making-the-compiler.md) (упражнение 3), [гл. 09](ch09-phantom-types-for-resource-tracking.md) (упражнение 4), [гл. 10](ch10-putting-it-all-together-a-complete-diagn.md) (упражнение 5)

## Практические задачи

### Упражнение 1: команда администрирования NVMe (типизированные команды)

Спроектируйте типизированный командный интерфейс для административных команд NVMe:

- `Identify` → `IdentifyResponse` (номер модели, серийный номер, версия прошивки)
- `GetLogPage` → `SmartLog` (температура, доступный резерв, количество прочитанных единиц данных)
- `GetFeature` → ответ, специфичный для функции

Требования:
1. Тип команды определяет тип ответа
2. Никакой диспетчеризации во время выполнения, только статическая
3. Добавьте newtype `NamespaceId`, который не даёт перепутать идентификаторы пространств имён с другими значениями `u32`

**Подсказка:** следуйте паттерну трейта `IpmiCmd` из гл. 02, но используйте константы, специфичные для NVMe.

<details>
<summary>Пример решения (упражнение 1)</summary>

```rust,ignore
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespaceId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub struct IdentifyResponse {
    pub model: String,
    pub serial: String,
    pub firmware_rev: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmartLog {
    pub temperature_kelvin: u16,
    pub available_spare_pct: u8,
    pub data_units_read: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArbitrationFeature {
    pub high_priority_weight: u8,
    pub medium_priority_weight: u8,
    pub low_priority_weight: u8,
}

/// Ключевой паттерн: ассоциированный тип фиксирует ответ каждой команды.
pub trait NvmeAdminCmd {
    type Response;
    fn opcode(&self) -> u8;
    fn nsid(&self) -> Option<NamespaceId>;
    fn parse_response(&self, raw: &[u8]) -> io::Result<Self::Response>;
}

pub struct Identify { pub nsid: NamespaceId }

impl NvmeAdminCmd for Identify {
    type Response = IdentifyResponse;
    fn opcode(&self) -> u8 { 0x06 }
    fn nsid(&self) -> Option<NamespaceId> { Some(self.nsid) }
    fn parse_response(&self, raw: &[u8]) -> io::Result<IdentifyResponse> {
        if raw.len() < 12 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "слишком коротко"));
        }
        Ok(IdentifyResponse {
            model: String::from_utf8_lossy(&raw[0..4]).trim().to_string(),
            serial: String::from_utf8_lossy(&raw[4..8]).trim().to_string(),
            firmware_rev: String::from_utf8_lossy(&raw[8..12]).trim().to_string(),
        })
    }
}

pub struct GetLogPage { pub log_id: u8 }

impl NvmeAdminCmd for GetLogPage {
    type Response = SmartLog;
    fn opcode(&self) -> u8 { 0x02 }
    fn nsid(&self) -> Option<NamespaceId> { None }
    fn parse_response(&self, raw: &[u8]) -> io::Result<SmartLog> {
        if raw.len() < 11 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "слишком коротко"));
        }
        Ok(SmartLog {
            temperature_kelvin: u16::from_le_bytes([raw[0], raw[1]]),
            available_spare_pct: raw[2],
            data_units_read: u64::from_le_bytes(raw[3..11].try_into().unwrap()),
        })
    }
}

pub struct GetFeature { pub feature_id: u8 }

impl NvmeAdminCmd for GetFeature {
    type Response = ArbitrationFeature;
    fn opcode(&self) -> u8 { 0x0A }
    fn nsid(&self) -> Option<NamespaceId> { None }
    fn parse_response(&self, raw: &[u8]) -> io::Result<ArbitrationFeature> {
        if raw.len() < 3 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "слишком коротко"));
        }
        Ok(ArbitrationFeature {
            high_priority_weight: raw[0],
            medium_priority_weight: raw[1],
            low_priority_weight: raw[2],
        })
    }
}

/// Статическая диспетчеризация: компилятор мономорфизирует код для каждого типа команды.
pub struct NvmeController;

impl NvmeController {
    pub fn execute<C: NvmeAdminCmd>(&self, cmd: &C) -> io::Result<C::Response> {
        // Собираем SQE из cmd.opcode()/cmd.nsid(),
        // отправляем в SQ, ждём CQ, затем:
        let raw = self.submit_and_read(cmd.opcode())?;
        cmd.parse_response(&raw)
    }

    fn submit_and_read(&self, _opcode: u8) -> io::Result<Vec<u8>> {
        // Реальная реализация обращается к /dev/nvme0
        Ok(vec![0; 512])
    }
}
```

**Ключевые моменты:**
- `NamespaceId(u32)` не даёт смешивать идентификаторы пространств имён с произвольными значениями `u32`.
- `NvmeAdminCmd::Response` — это «индекс типа»: `execute()` возвращает ровно `C::Response`.
- Полностью статическая диспетчеризация: никаких `Box<dyn …>` и никакого динамического приведения типов во время выполнения.

</details>

### Упражнение 2: автомат состояний обновления прошивки (typestate)

Смоделируйте жизненный цикл обновления прошивки BMC:

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Uploading : begin_upload()
    Uploading --> Uploading : send_chunk(data)
    Uploading --> Verifying : finish_upload()
    Uploading --> Idle : abort()
    Verifying --> Applying : verify() ✅ + токен VerifiedImage
    Verifying --> Idle : verify() ❌ или abort()
    Applying --> Rebooting : apply(token)
    Rebooting --> Complete : reboot_complete()
    Complete --> [*]

    note right of Applying : нет abort(): необратимо
    note right of Verifying : VerifiedImage — токен-доказательство
```

Требования:
1. Каждое состояние — отдельный тип
2. Загрузка может начинаться только из `Idle`
3. Проверка требует завершённой загрузки
4. Применение возможно только после успешной проверки: примите токен-доказательство `VerifiedImage`
5. Перезагрузка — единственный вариант после применения
6. Добавьте метод `abort()`, доступный в `Uploading` и `Verifying` (но не в `Applying`: уже поздно)

**Подсказка:** совместите typestate (гл. 05) с capability-токенами (гл. 04).

<details>
<summary>Пример решения (упражнение 2)</summary>

```rust,ignore
// --- Типы состояний ---
// Проектное решение: здесь состояние хранится непосредственно (`_state: S`), а не через
// `PhantomData<S>` (подход гл. 05). Это позволяет состояниям нести данные: например,
// `Uploading { bytes_sent: usize }` отслеживает прогресс. Используйте `PhantomData`,
// когда состояния — чистые маркеры (нулевого размера); храните состояние в структуре,
// когда состояния несут значимые данные времени выполнения.
pub struct Idle;
pub struct Uploading { bytes_sent: usize }  // не ZST: несёт данные о прогрессе
pub struct Verifying;
pub struct Applying;
pub struct Rebooting;
pub struct Complete;

/// Токен-доказательство: создаётся только внутри verify().
pub struct VerifiedImage { _private: () }

pub struct FwUpdate<S> {
    bmc_addr: String,
    _state: S,
}

impl FwUpdate<Idle> {
    pub fn new(bmc_addr: &str) -> Self {
        FwUpdate { bmc_addr: bmc_addr.to_string(), _state: Idle }
    }
    pub fn begin_upload(self) -> FwUpdate<Uploading> {
        FwUpdate { bmc_addr: self.bmc_addr, _state: Uploading { bytes_sent: 0 } }
    }
}

impl FwUpdate<Uploading> {
    pub fn send_chunk(mut self, chunk: &[u8]) -> Self {
        self._state.bytes_sent += chunk.len();
        self
    }
    pub fn finish_upload(self) -> FwUpdate<Verifying> {
        FwUpdate { bmc_addr: self.bmc_addr, _state: Verifying }
    }
    /// Abort доступен во время загрузки: возврат в Idle.
    pub fn abort(self) -> FwUpdate<Idle> {
        FwUpdate { bmc_addr: self.bmc_addr, _state: Idle }
    }
}

impl FwUpdate<Verifying> {
    /// При успехе возвращает следующее состояние И токен-доказательство VerifiedImage.
    pub fn verify(self) -> Result<(FwUpdate<Applying>, VerifiedImage), FwUpdate<Idle>> {
        // Реально: проверяем CRC, подпись, совместимость
        let token = VerifiedImage { _private: () };
        Ok((
            FwUpdate { bmc_addr: self.bmc_addr, _state: Applying },
            token,
        ))
    }
    /// Abort доступен во время проверки.
    pub fn abort(self) -> FwUpdate<Idle> {
        FwUpdate { bmc_addr: self.bmc_addr, _state: Idle }
    }
}

impl FwUpdate<Applying> {
    /// Потребляет токен-доказательство VerifiedImage: без проверки применить нельзя.
    /// Примечание: метода abort() здесь НЕТ, потому что после начала записи это слишком опасно.
    pub fn apply(self, _proof: VerifiedImage) -> FwUpdate<Rebooting> {
        FwUpdate { bmc_addr: self.bmc_addr, _state: Rebooting }
    }
}

impl FwUpdate<Rebooting> {
    pub fn wait_for_reboot(self) -> FwUpdate<Complete> {
        FwUpdate { bmc_addr: self.bmc_addr, _state: Complete }
    }
}

impl FwUpdate<Complete> {
    pub fn version(&self) -> &str { "2.1.0" }
}

// Использование:
// let fw = FwUpdate::new("192.168.1.100")
//     .begin_upload()
//     .send_chunk(b"image_data")
//     .finish_upload();
// let (fw, proof) = fw.verify().map_err(|_| "проверка не пройдена")?;
// let fw = fw.apply(proof).wait_for_reboot();
// println!("Новая версия: {}", fw.version());
```

**Ключевые моменты:**
- `abort()` существует только у `FwUpdate<Uploading>` и `FwUpdate<Verifying>`: вызов его у `FwUpdate<Applying>` — **ошибка компиляции**, а не проверка во время выполнения.
- `VerifiedImage` имеет приватное поле, поэтому создать его может только `verify()`.
- `apply()` потребляет токен-доказательство: пропустить проверку нельзя.

</details>

### Упражнение 3: конвейер показаний датчиков (анализ размерностей)

Постройте полный конвейер обработки показаний датчиков:

1. Определите newtype: `RawAdc`, `Celsius`, `Fahrenheit`, `Volts`, `Millivolts`, `Watts`
2. Реализуйте `From<Celsius> for Fahrenheit` и наоборот
3. Создайте `impl Mul<Volts, Output=Watts> for Amperes` (P = V × I)
4. Постройте обобщённый проверщик порогов `Threshold<T>`
5. Напишите конвейер: АЦП → калибровка → проверка порога → результат

Компилятор должен отвергать: сравнение `Celsius` с `Volts`, сложение `Watts` с `Rpm`, передачу `Millivolts` там, где ожидаются `Volts`.

<details>
<summary>Пример решения (упражнение 3)</summary>

```rust,ignore
use std::ops::{Add, Sub, Mul};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RawAdc(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Celsius(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Fahrenheit(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Volts(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Millivolts(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Amperes(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Watts(pub f64);

// --- Безопасные преобразования ---
impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self { Fahrenheit(c.0 * 9.0 / 5.0 + 32.0) }
}
impl From<Fahrenheit> for Celsius {
    fn from(f: Fahrenheit) -> Self { Celsius((f.0 - 32.0) * 5.0 / 9.0) }
}
impl From<Millivolts> for Volts {
    fn from(mv: Millivolts) -> Self { Volts(mv.0 / 1000.0) }
}
impl From<Volts> for Millivolts {
    fn from(v: Volts) -> Self { Millivolts(v.0 * 1000.0) }
}

// --- Арифметика для величин одной единицы ---
// ПРИМЕЧАНИЕ: сложение абсолютных температур (25°C + 30°C) физически сомнительно.
// См. в гл. 06 обсуждение newtype для ΔT: там подход строже. Здесь для упражнения оставим всё просто.
impl Add for Celsius {
    type Output = Celsius;
    fn add(self, rhs: Self) -> Celsius { Celsius(self.0 + rhs.0) }
}
impl Sub for Celsius {
    type Output = Celsius;
    fn sub(self, rhs: Self) -> Celsius { Celsius(self.0 - rhs.0) }
}

// P = V × I  (умножение разных единиц)
impl Mul<Amperes> for Volts {
    type Output = Watts;
    fn mul(self, rhs: Amperes) -> Watts { Watts(self.0 * rhs.0) }
}

// --- Обобщённый проверщик порогов ---
// Упражнение 3 расширяет Threshold из гл. 06 обобщённым ThresholdResult<T>,
// который несёт показание, вызвавшее срабатывание. Это развитие более простого
// enum ThresholdResult { Normal, Warning, Critical } из гл. 06.
pub enum ThresholdResult<T> {
    Normal(T),
    Warning(T),
    Critical(T),
}

pub struct Threshold<T> {
    pub warning: T,
    pub critical: T,
}

// Обобщённая реализация: работает для любого типа единиц, поддерживающего PartialOrd.
impl<T: PartialOrd + Copy> Threshold<T> {
    pub fn check(&self, reading: T) -> ThresholdResult<T> {
        if reading >= self.critical {
            ThresholdResult::Critical(reading)
        } else if reading >= self.warning {
            ThresholdResult::Warning(reading)
        } else {
            ThresholdResult::Normal(reading)
        }
    }
}
// Теперь `Threshold<Rpm>`, `Threshold<Volts>` и т. д. работают автоматически.

// --- Конвейер: АЦП → калибровка → порог → результат ---
pub struct CalibrationParams {
    pub scale: f64,  // отсчётов АЦП на °C
    pub offset: f64, // °C при АЦП 0
}

pub fn calibrate(raw: RawAdc, params: &CalibrationParams) -> Celsius {
    Celsius(raw.0 as f64 / params.scale + params.offset)
}

pub fn sensor_pipeline(
    raw: RawAdc,
    params: &CalibrationParams,
    threshold: &Threshold<Celsius>,
) -> ThresholdResult<Celsius> {
    let temp = calibrate(raw, params);
    threshold.check(temp)
}

// Проверка на этапе компиляции: эти строки НЕ скомпилируются:
// let _ = Celsius(25.0) + Volts(12.0);   // ОШИБКА: mismatched types
// let _: Millivolts = Volts(1.0);         // ОШИБКА: no implicit coercion
// let _ = Watts(100.0) + Rpm(3000);       // ОШИБКА: mismatched types
```

**Ключевые моменты:**
- Каждая физическая величина — отдельный тип: случайного смешивания не происходит.
- `Mul<Amperes> for Volts` даёт `Watts`, кодируя P = V × I в системе типов.
- Явные преобразования `From` для связанных единиц (мВ ↔ В, °C ↔ °F).
- `Threshold<Celsius>` принимает только `Celsius`: нельзя случайно проверить порог для об/мин.

</details>

### Упражнение 4: обход возможностей PCIe (phantom-типы + проверенная граница)

Смоделируйте связный список возможностей PCIe:

1. `RawCapability`: непроверенные байты из конфигурационного пространства
2. `ValidCapability`: разобранная и проверенная (через TryFrom)
3. Каждый тип возможности (MSI, MSI-X, PCI Express, Power Management) имеет собственную раскладку регистров с phantom-типом
4. Обход списка возвращает итератор значений `ValidCapability`

**Подсказка:** совместите проверенные границы (гл. 07) с phantom-типами (гл. 09).

<details>
<summary>Пример решения (упражнение 4)</summary>

```rust,ignore
use std::marker::PhantomData;

// --- Phantom-маркеры для типов возможностей ---
pub struct Msi;
pub struct MsiX;
pub struct PciExpress;
pub struct PowerMgmt;

// Идентификаторы возможностей PCI из спецификации
const CAP_ID_PM:   u8 = 0x01;
const CAP_ID_MSI:  u8 = 0x05;
const CAP_ID_PCIE: u8 = 0x10;
const CAP_ID_MSIX: u8 = 0x11;

/// Непроверенные байты: могут быть мусором.
#[derive(Debug)]
pub struct RawCapability {
    pub id: u8,
    pub next_ptr: u8,
    pub data: Vec<u8>,
}

/// Проверенная и помеченная типом возможность.
#[derive(Debug)]
pub struct ValidCapability<Kind> {
    id: u8,
    next_ptr: u8,
    data: Vec<u8>,
    _kind: PhantomData<Kind>,
}

// --- TryFrom: граница parse-don't-validate ---
impl TryFrom<RawCapability> for ValidCapability<PowerMgmt> {
    type Error = &'static str;
    fn try_from(raw: RawCapability) -> Result<Self, Self::Error> {
        if raw.id != CAP_ID_PM { return Err("не возможность PM"); }
        if raw.data.len() < 2 { return Err("данные PM слишком короткие"); }
        Ok(ValidCapability {
            id: raw.id, next_ptr: raw.next_ptr,
            data: raw.data, _kind: PhantomData,
        })
    }
}

impl TryFrom<RawCapability> for ValidCapability<Msi> {
    type Error = &'static str;
    fn try_from(raw: RawCapability) -> Result<Self, Self::Error> {
        if raw.id != CAP_ID_MSI { return Err("не возможность MSI"); }
        if raw.data.len() < 6 { return Err("данные MSI слишком короткие"); }
        Ok(ValidCapability {
            id: raw.id, next_ptr: raw.next_ptr,
            data: raw.data, _kind: PhantomData,
        })
    }
}

// (Аналогичные реализации TryFrom для MsiX и PciExpress опущены для краткости)

// --- Типобезопасные методы доступа: доступны только для правильной возможности ---
impl ValidCapability<PowerMgmt> {
    pub fn pm_control(&self) -> u16 {
        u16::from_le_bytes([self.data[0], self.data[1]])
    }
}

impl ValidCapability<Msi> {
    pub fn message_control(&self) -> u16 {
        u16::from_le_bytes([self.data[0], self.data[1]])
    }
    pub fn vectors_requested(&self) -> u32 {
        1 << ((self.message_control() >> 1) & 0x07)
    }
}

impl ValidCapability<MsiX> {
    pub fn table_size(&self) -> u16 {
        (u16::from_le_bytes([self.data[0], self.data[1]]) & 0x07FF) + 1
    }
}

// --- Обходчик возможностей: проходит по связному списку ---
pub struct CapabilityWalker<'a> {
    config_space: &'a [u8],
    next_ptr: u8,
}

impl<'a> CapabilityWalker<'a> {
    pub fn new(config_space: &'a [u8]) -> Self {
        // Указатель на первую возможность находится по смещению 0x34 в конфигурационном пространстве PCI
        let first_ptr = if config_space.len() > 0x34 {
            config_space[0x34]
        } else { 0 };
        CapabilityWalker { config_space, next_ptr: first_ptr }
    }
}

impl<'a> Iterator for CapabilityWalker<'a> {
    type Item = RawCapability;
    fn next(&mut self) -> Option<RawCapability> {
        if self.next_ptr == 0 { return None; }
        let off = self.next_ptr as usize;
        if off + 2 > self.config_space.len() { return None; }
        let id = self.config_space[off];
        let next = self.config_space[off + 1];
        let end = if next > 0 { next as usize } else {
            (off + 16).min(self.config_space.len())
        };
        let data = self.config_space[off + 2..end].to_vec();
        self.next_ptr = next;
        Some(RawCapability { id, next_ptr: next, data })
    }
}

// Использование:
// for raw_cap in CapabilityWalker::new(&config_space) {
//     if let Ok(pm) = ValidCapability::<PowerMgmt>::try_from(raw_cap) {
//         println!("Управление PM: 0x{:04X}", pm.pm_control());
//     }
// }
```

**Ключевые моменты:**
- `RawCapability` → `ValidCapability<Kind>` — граница parse-don't-validate.
- `pm_control()` существует только у `ValidCapability<PowerMgmt>`: вызов его у возможности MSI — **ошибка компиляции**.
- Итератор `CapabilityWalker` выдаёт сырые возможности; вызывающий код проверяет нужные ему через `TryFrom`.

</details>

### Упражнение 5: проверка здоровья по нескольким протоколам (capability-миксины)

Создайте фреймворк проверки здоровья:

1. Определите трейты-ингредиенты: `HasIpmi`, `HasRedfish`, `HasNvmeCli`, `HasGpio`
2. Создайте трейты-миксины:
   - `ThermalHealthMixin` (требует HasIpmi + HasGpio): читает температуры, проверяет сигналы тревоги
   - `StorageHealthMixin` (требует HasNvmeCli): проверки данных SMART
   - `BmcHealthMixin` (требует HasIpmi + HasRedfish): перекрёстная проверка данных BMC
3. Постройте `FullPlatformController`, который реализует все трейты-ингредиенты
4. Постройте `StorageOnlyController`, который реализует только `HasNvmeCli`
5. Убедитесь, что `StorageOnlyController` получает `StorageHealthMixin`, но НЕ остальные

<details>
<summary>Пример решения (упражнение 5)</summary>

```rust,ignore
// --- Трейты-ингредиенты ---
pub trait HasIpmi {
    fn ipmi_read_sensor(&self, id: u8) -> f64;
}
pub trait HasRedfish {
    fn redfish_get(&self, path: &str) -> String;
}
pub trait HasNvmeCli {
    fn nvme_smart_log(&self, dev: &str) -> SmartData;
}
pub trait HasGpio {
    fn gpio_read_alert(&self, pin: u8) -> bool;
}

pub struct SmartData {
    pub temperature_kelvin: u16,
    pub spare_pct: u8,
}

// --- Трейты-миксины с blanket impl ---
pub trait ThermalHealthMixin: HasIpmi + HasGpio {
    fn thermal_check(&self) -> ThermalStatus {
        let temp = self.ipmi_read_sensor(0x01);
        let alert = self.gpio_read_alert(12);
        ThermalStatus { temperature: temp, alert_active: alert }
    }
}
impl<T: HasIpmi + HasGpio> ThermalHealthMixin for T {}

pub trait StorageHealthMixin: HasNvmeCli {
    fn storage_check(&self) -> StorageStatus {
        let smart = self.nvme_smart_log("/dev/nvme0");
        StorageStatus {
            temperature_ok: smart.temperature_kelvin < 343, // 70 °C
            spare_ok: smart.spare_pct > 10,
        }
    }
}
impl<T: HasNvmeCli> StorageHealthMixin for T {}

pub trait BmcHealthMixin: HasIpmi + HasRedfish {
    fn bmc_health(&self) -> BmcStatus {
        let ipmi_temp = self.ipmi_read_sensor(0x01);
        let rf_temp = self.redfish_get("/Thermal/Temperatures/0");
        BmcStatus { ipmi_temp, redfish_temp: rf_temp, consistent: true }
    }
}
impl<T: HasIpmi + HasRedfish> BmcHealthMixin for T {}

pub struct ThermalStatus { pub temperature: f64, pub alert_active: bool }
pub struct StorageStatus { pub temperature_ok: bool, pub spare_ok: bool }
pub struct BmcStatus { pub ipmi_temp: f64, pub redfish_temp: String, pub consistent: bool }

// --- Полная платформа: все ингредиенты → все три миксина бесплатно ---
pub struct FullPlatformController;

impl HasIpmi for FullPlatformController {
    fn ipmi_read_sensor(&self, _id: u8) -> f64 { 42.0 }
}
impl HasRedfish for FullPlatformController {
    fn redfish_get(&self, _path: &str) -> String { "42.0".into() }
}
impl HasNvmeCli for FullPlatformController {
    fn nvme_smart_log(&self, _dev: &str) -> SmartData {
        SmartData { temperature_kelvin: 310, spare_pct: 95 }
    }
}
impl HasGpio for FullPlatformController {
    fn gpio_read_alert(&self, _pin: u8) -> bool { false }
}

// --- Только хранилище: только HasNvmeCli → только StorageHealthMixin ---
pub struct StorageOnlyController;

impl HasNvmeCli for StorageOnlyController {
    fn nvme_smart_log(&self, _dev: &str) -> SmartData {
        SmartData { temperature_kelvin: 315, spare_pct: 80 }
    }
}

// StorageOnlyController автоматически получает storage_check().
// Вызов thermal_check() или bmc_health() у него — ОШИБКА КОМПИЛЯЦИИ.
```

**Ключевые моменты:**
- Blanket `impl<T: HasIpmi + HasGpio> ThermalHealthMixin for T {}`: любой тип, реализующий оба ингредиента, автоматически получает миксин.
- `StorageOnlyController` реализует только `HasNvmeCli`, поэтому компилятор даёт ему `StorageHealthMixin`, но отвергает `thermal_check()` и `bmc_health()`: никаких проверок во время выполнения не нужно.
- Добавление нового миксина (например, `NetworkHealthMixin: HasRedfish + HasGpio`) — это один трейт и одна blanket impl: существующие контроллеры получат его автоматически, если подходят.

</details>

### Упражнение 6: диагностический протокол с сессионными типами (одноразовые типы + typestate)

Спроектируйте диагностическую сессию с одноразовыми токенами выполнения тестов:

1. `DiagSession` начинает в состоянии `Setup`
2. Переход в состояние `Running` выдаёт `N` токенов выполнения (по одному на каждый тестовый случай)
3. Каждый `TestToken` потребляется при запуске теста, что исключает повторный запуск одного и того же теста
4. Когда все токены потреблены, переход в состояние `Complete`
5. Формирование отчёта (только в состоянии `Complete`)

**Продвинутый вариант:** используйте const-обобщение `N`, чтобы отслеживать на уровне типов, сколько тестов осталось.

<details>
<summary>Пример решения (упражнение 6)</summary>

```rust,ignore
// --- Типы состояний ---
pub struct Setup;
pub struct Running;
pub struct Complete;

/// Одноразовый токен теста. НЕ Clone, НЕ Copy: потребляется при использовании.
pub struct TestToken {
    test_name: String,
}

#[derive(Debug)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
}

pub struct DiagSession<S> {
    name: String,
    results: Vec<TestResult>,
    _state: S,
}

impl DiagSession<Setup> {
    pub fn new(name: &str) -> Self {
        DiagSession {
            name: name.to_string(),
            results: Vec::new(),
            _state: Setup,
        }
    }

    /// Переход в Running: выдаёт по одному токену на каждый тестовый случай.
    pub fn start(self, test_names: &[&str]) -> (DiagSession<Running>, Vec<TestToken>) {
        let tokens = test_names.iter()
            .map(|n| TestToken { test_name: n.to_string() })
            .collect();
        (
            DiagSession {
                name: self.name,
                results: Vec::new(),
                _state: Running,
            },
            tokens,
        )
    }
}

impl DiagSession<Running> {
    /// Потребляет токен для запуска одного теста. Перемещение исключает двойной запуск.
    pub fn run_test(mut self, token: TestToken) -> Self {
        let passed = true; // реальный код здесь запускает настоящую диагностику
        self.results.push(TestResult {
            test_name: token.test_name,
            passed,
        });
        self
    }

    /// Переход в Complete.
    ///
    /// **Примечание:** это решение НЕ обеспечивает, что все токены потреблены:
    /// `finish()` можно вызвать, когда токены ещё остались. Такие токены просто
    /// уничтожатся (они не помечены `#[must_use]`). Для полной проверки на этапе компиляции
    /// используйте вариант с const-обобщением из примечания «Продвинутый» ниже, где
    /// `finish()` доступен только у `DiagSession<Running, 0>`.
    pub fn finish(self) -> DiagSession<Complete> {
        DiagSession {
            name: self.name,
            results: self.results,
            _state: Complete,
        }
    }
}

impl DiagSession<Complete> {
    /// Отчёт доступен ТОЛЬКО в состоянии Complete.
    pub fn report(&self) -> String {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        format!("{}: пройдено {}/{}", self.name, passed, total)
    }
}

// Использование:
// let session = DiagSession::new("стресс-тест GPU");
// let (mut session, tokens) = session.start(&["vram", "compute", "thermal"]);
// for token in tokens {
//     session = session.run_test(token);
// }
// let session = session.finish();
// println!("{}", session.report());  // "стресс-тест GPU: пройдено 3/3"
//
// // Это НЕ скомпилируется:
// // session.run_test(used_token);  →  ОШИБКА: use of moved value
// // running_session.report();      →  ОШИБКА: no method `report` on DiagSession<Running>
```

**Ключевые моменты:**
- `TestToken` не реализует `Clone` и `Copy`: потребление через `run_test(token)` перемещает его, поэтому повторный запуск того же теста — ошибка компиляции.
- `report()` существует только у `DiagSession<Complete>`: вызвать его в процессе выполнения невозможно.
- **Продвинутый** вариант использовал бы `DiagSession<Running, N>` с const-обобщениями, где `run_test` возвращает `DiagSession<Running, {N-1}>`, а `finish` доступен только у `DiagSession<Running, 0>`. Это гарантирует, что *все* токены потреблены до завершения.

</details>

## Ключевые выводы

1. **Практикуйтесь на реалистичных протоколах**: NVMe, обновление прошивки, конвейеры датчиков и PCIe — реальные цели для этих паттернов.
2. **Каждое упражнение соответствует основной главе**: используйте перекрёстные ссылки, чтобы повторить паттерн перед попыткой.
3. **Решения спрятаны в раскрывающихся блоках**: попробуйте каждое упражнение, прежде чем открывать решение.
4. **Сочетайте паттерны в упражнении 5**: проверки здоровья по нескольким протоколам объединяют типизированные команды, размерные типы и проверенные границы.
5. **Сессионные типы (упражнение 6) — передний край**: они обеспечивают порядок сообщений в каналах, распространяя typestate на распределённые системы.

---
