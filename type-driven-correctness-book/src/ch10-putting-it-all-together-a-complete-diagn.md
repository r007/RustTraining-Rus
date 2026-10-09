# Собираем всё вместе: полная платформа диагностики 🟡

> **Что вы узнаете:** как все семь основных паттернов (гл. 02–09) объединяются в единый диагностический сценарий: аутентификация, сессии, типизированные команды, токены аудита, размерные результаты, проверенные данные и регистры с phantom-типами. Суммарные накладные расходы во время выполнения при этом равны нулю.
>
> **Перекрёстные ссылки:** каждая глава с основными паттернами (гл. 02–09), [гл. 14](ch14-testing-type-level-guarantees.md) (тестирование этих гарантий)

## Цель

Эта глава объединяет **семь паттернов** из глав 2–9 в единый реалистичный диагностический сценарий. Мы построим проверку состояния сервера, которая:

1. **Проходит аутентификацию** (capability-токен, гл. 04)
2. **Открывает сессию IPMI** (typestate, гл. 05)
3. **Отправляет типизированные команды** (типизированные команды, гл. 02)
4. **Использует одноразовые токены** для журнала аудита (одноразовые типы, гл. 03)
5. **Возвращает размерные результаты** (анализ размерностей, гл. 06)
6. **Проверяет данные FRU** (проверенные границы, гл. 07)
7. **Читает типизированные регистры** (phantom-типы, гл. 09)

```rust,ignore
use std::marker::PhantomData;
use std::io;
// ──── Паттерн 1: размерные типы (гл. 06) ────

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Celsius(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Rpm(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Volts(pub f64);

// ──── Паттерн 2: типизированные команды (гл. 02) ────

/// Та же форма трейта, что в гл. 02; здесь используются методы (а не ассоциированные
/// константы) для единообразия. Ассоциированные константы (`const NETFN: u8`) — равноценная
/// альтернатива, если значение действительно фиксировано для типа.
pub trait IpmiCmd {
    type Response;
    fn net_fn(&self) -> u8;
    fn cmd_byte(&self) -> u8;
    fn payload(&self) -> Vec<u8>;
    fn parse_response(&self, raw: &[u8]) -> io::Result<Self::Response>;
}

pub struct ReadTemp { pub sensor_id: u8 }
impl IpmiCmd for ReadTemp {
    type Response = Celsius;   // ← размерный тип!
    fn net_fn(&self) -> u8 { 0x04 }
    fn cmd_byte(&self) -> u8 { 0x2D }
    fn payload(&self) -> Vec<u8> { vec![self.sensor_id] }
    fn parse_response(&self, raw: &[u8]) -> io::Result<Celsius> {
        if raw.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "пустой ответ"));
        }
        Ok(Celsius(raw[0] as f64))
    }
}

pub struct ReadFanSpeed { pub fan_id: u8 }
impl IpmiCmd for ReadFanSpeed {
    type Response = Rpm;
    fn net_fn(&self) -> u8 { 0x04 }
    fn cmd_byte(&self) -> u8 { 0x2D }
    fn payload(&self) -> Vec<u8> { vec![self.fan_id] }
    fn parse_response(&self, raw: &[u8]) -> io::Result<Rpm> {
        if raw.len() < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "нужно 2 байта"));
        }
        Ok(Rpm(u16::from_le_bytes([raw[0], raw[1]]) as f64))
    }
}

// ──── Паттерн 3: capability-токен (гл. 04) ────

pub struct AdminToken { _private: () }

pub fn authenticate(user: &str, pass: &str) -> Result<AdminToken, &'static str> {
    if user == "admin" && pass == "secret" {
        Ok(AdminToken { _private: () })
    } else {
        Err("аутентификация не пройдена")
    }
}

// ──── Паттерн 4: сессия с typestate (гл. 05) ────

pub struct Idle;
pub struct Active;

pub struct Session<State> {
    host: String,
    _state: PhantomData<State>,
}

impl Session<Idle> {
    pub fn connect(host: &str) -> Self {
        Session { host: host.to_string(), _state: PhantomData }
    }

    pub fn activate(
        self,
        _admin: &AdminToken,  // ← требует capability-токен
    ) -> Result<Session<Active>, String> {
        println!("Сессия активирована на {}", self.host);
        Ok(Session { host: self.host, _state: PhantomData })
    }
}

impl Session<Active> {
    /// Выполняет типизированную команду — доступно только для активных сессий.
    /// Возвращает io::Result, чтобы пробрасывать ошибки транспорта (согласованно с гл. 02).
    pub fn execute<C: IpmiCmd>(&mut self, cmd: &C) -> io::Result<C::Response> {
        let raw_response = self.raw_send(cmd.net_fn(), cmd.cmd_byte(), &cmd.payload())?;
        cmd.parse_response(&raw_response)
    }

    fn raw_send(&self, _nf: u8, _cmd: u8, _data: &[u8]) -> io::Result<Vec<u8>> {
        Ok(vec![42, 0x1E]) // заглушка: сырой ответ IPMI
    }

    pub fn close(self) { println!("Сессия закрыта"); }
}

// ──── Паттерн 5: одноразовый токен аудита (гл. 03) ────

/// Каждый запуск диагностики получает уникальный токен аудита.
/// Не Clone, не Copy: гарантирует, что каждая запись аудита уникальна.
pub struct AuditToken {
    run_id: u64,
}

impl AuditToken {
    pub fn issue(run_id: u64) -> Self {
        AuditToken { run_id }
    }

    /// Потребляет токен для записи в журнал аудита.
    pub fn log(self, message: &str) {
        println!("[АУДИТ run_id={}] {}", self.run_id, message);
        // токен потреблён — нельзя записать один и тот же run_id дважды
    }
}

// ──── Паттерн 6: проверенная граница (гл. 07) ────
// Упрощено по сравнению с полным ValidFru из гл. 07: только поля, нужные для этого
// составного примера. Полную версию TryFrom<RawFruData> см. в гл. 07.

pub struct ValidFru {
    pub board_serial: String,
    pub product_name: String,
}

impl ValidFru {
    pub fn parse(raw: &[u8]) -> Result<Self, &'static str> {
        if raw.len() < 8 { return Err("FRU слишком короткий"); }
        if raw[0] != 0x01 { return Err("неверная версия FRU"); }
        Ok(ValidFru {
            board_serial: "SN12345".to_string(),  // заглушка
            product_name: "ServerX".to_string(),
        })
    }
}

// ──── Паттерн 7: регистры с phantom-типом (гл. 09) ────

pub struct Width16;
pub struct Reg<W> { offset: u16, _w: PhantomData<W> }

impl Reg<Width16> {
    pub fn read(&self) -> u16 { 0x8086 } // заглушка
}

pub struct PcieDev {
    pub vendor_id: Reg<Width16>,
    pub device_id: Reg<Width16>,
}

impl PcieDev {
    pub fn new() -> Self {
        PcieDev {
            vendor_id: Reg { offset: 0x00, _w: PhantomData },
            device_id: Reg { offset: 0x02, _w: PhantomData },
        }
    }
}

// ──── Составной сценарий ────

fn full_diagnostic() -> Result<(), String> {
    // 1. Аутентификация → получаем capability-токен
    let admin = authenticate("admin", "secret")
        .map_err(|e| e.to_string())?;

    // 2. Подключение и активация сессии (typestate: Idle → Active)
    let session = Session::connect("192.168.1.100");
    let mut session = session.activate(&admin)?;  // требует AdminToken

    // 3. Отправляем типизированные команды (тип ответа соответствует команде)
    let temp: Celsius = session.execute(&ReadTemp { sensor_id: 0 })
        .map_err(|e| e.to_string())?;
    let fan: Rpm = session.execute(&ReadFanSpeed { fan_id: 1 })
        .map_err(|e| e.to_string())?;

    // Несоответствие типов будет поймано:
    // let wrong: Volts = session.execute(&ReadTemp { sensor_id: 0 })?;
    //  ❌ ОШИБКА: expected Celsius, found Volts

    // 4. Читаем регистры PCIe с phantom-типом
    let pcie = PcieDev::new();
    let vid: u16 = pcie.vendor_id.read();  // гарантированно u16

    // 5. Проверяем данные FRU на границе
    let raw_fru = vec![0x01, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0xFD];
    let fru = ValidFru::parse(&raw_fru)
        .map_err(|e| e.to_string())?;

    // 6. Выдаём одноразовый токен аудита
    let audit = AuditToken::issue(1001);

    // 7. Формируем отчёт (все данные типизированы и проверены)
    let report = format!(
        "Сервер: {} (SN: {}), VID: 0x{:04X}, CPU: {:?}, Вентилятор: {:?}",
        fru.product_name, fru.board_serial, vid, temp, fan,
    );

    // 8. Потребляем токен аудита — записать дважды нельзя
    audit.log(&report);
    // audit.log("повтор");  // ❌ use of moved value

    // 9. Закрываем сессию (typestate: Active → уничтожена)
    session.close();

    Ok(())
}
```

### Что доказывает компилятор

| Класс ошибки | Как это предотвращается | Паттерн |
|--------------|-------------------------|---------|
| Доступ без аутентификации | `activate()` требует `&AdminToken` | Capability-токен |
| Команда в неверном состоянии сессии | `execute()` существует только у `Session<Active>` | Typestate |
| Неверный тип ответа | `ReadTemp::Response = Celsius`, фиксируется трейтом | Типизированные команды |
| Путаница единиц (°C и об/мин) | `Celsius` ≠ `Rpm` ≠ `Volts` | Размерные типы |
| Несоответствие ширины регистра | `Reg<Width16>` возвращает `u16` | Phantom-типы |
| Обработка непроверенных данных | Сначала нужно вызвать `ValidFru::parse()` | Проверенная граница |
| Дублирование записей аудита | `AuditToken` потребляется при записи в журнал | Одноразовый тип |
| Нарушение порядка подачи питания | Каждый шаг требует токен предыдущего | Capability-токены (гл. 04) |

**Суммарные накладные расходы во время выполнения от ВСЕХ этих гарантий: ноль.**

Каждая проверка происходит на этапе компиляции. Сгенерированный ассемблер идентичен рукописному коду на C без единой проверки, но **в C могут быть ошибки, а здесь — нет**.

## Ключевые выводы

1. **Семь паттернов сочетаются без швов**: capability-токены, typestate, типизированные команды, одноразовые типы, размерные типы, проверенные границы и phantom-типы работают вместе.
2. **Компилятор доказывает невозможность восьми классов ошибок**: см. таблицу «Что доказывает компилятор» выше.
3. **Нулевые суммарные накладные расходы во время выполнения**: сгенерированный ассемблер идентичен коду на C без проверок.
4. **Каждый паттерн полезен сам по себе**: не обязательно использовать все семь; внедряйте их постепенно.
5. **Глава с интеграцией — это шаблон проектирования**: используйте её как отправную точку для собственных типизированных диагностических сценариев.
6. **От IPMI к Redfish в масштабе**: гл. 17 и 18 применяют эти же семь паттернов (плюс capability-миксины из гл. 08) к полному клиенту и серверу Redfish. Сценарий IPMI здесь — основа; пошаговые разборы Redfish показывают, как композиция масштабируется до производственных систем с несколькими источниками данных и ограничениями версий схем.

---
