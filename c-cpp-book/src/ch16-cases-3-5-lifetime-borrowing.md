# Кейс 3: коммуникация с фреймворком → заимствование по времени жизни

> **Что вы узнаете:** как перевести паттерны взаимодействия с фреймворком через сырые указатели из C++ на систему заимствований Rust, основанную на временах жизни, устраняя риск висячих указателей и сохраняя абстракции без накладных расходов.

## Паттерн C++: сырой указатель на фреймворк
```cpp
// Исходный C++: каждый диагностический модуль хранит сырой указатель на фреймворк
class DiagBase {
protected:
    DiagFramework* m_pFramework;  // Сырой указатель — кто им владеет?
public:
    DiagBase(DiagFramework* fw) : m_pFramework(fw) {}
    
    void LogEvent(uint32_t code, const std::string& msg) {
        m_pFramework->GetEventLog()->Record(code, msg);  // Надеемся, что он ещё жив!
    }
};
// Проблема: m_pFramework — сырой указатель без гарантии времени жизни
// Если фреймворк уничтожен, пока модули ещё на него ссылаются → неопределённое поведение
```

## Решение на Rust: DiagContext с заимствованием по времени жизни
```rust
// Пример: module.rs — заимствуем, а не храним

/// Контекст, передаваемый диагностическим модулям во время выполнения.
/// Время жизни 'a гарантирует, что фреймворк переживёт контекст.
pub struct DiagContext<'a> {
    pub der_log: &'a mut EventLogManager,
    pub config: &'a ModuleConfig,
    pub framework_opts: &'a HashMap<String, String>,
}

/// Модули получают контекст как параметр — никогда не храним указатели на фреймворк
pub trait DiagModule {
    fn id(&self) -> &str;
    fn execute(&mut self, ctx: &mut DiagContext) -> DiagResult<()>;
    fn pre_execute(&mut self, _ctx: &mut DiagContext) -> DiagResult<()> {
        Ok(())
    }
    fn post_execute(&mut self, _ctx: &mut DiagContext) -> DiagResult<()> {
        Ok(())
    }
}
```

### Главная мысль
- Модули C++ **хранят** указатель на фреймворк (опасность: что если фреймворк уничтожат первым?)
- Модули Rust **получают** контекст как параметр функции — проверка заимствований гарантирует, что фреймворк жив во время вызова
- Никаких сырых указателей, никакой неоднозначности времён жизни, никаких «надеемся, что он ещё жив»

----

# Кейс 4: «божественный объект» → компонуемое состояние

## Паттерн C++: монолитный класс фреймворка
```cpp
// Исходный C++: фреймворк — божественный объект
class DiagFramework {
    // Обработка ловушек монитора здоровья
    std::vector<AlertTriggerInfo> m_alertTriggers;
    std::vector<WarnTriggerInfo> m_warnTriggers;
    bool m_healthMonHasBootTimeError;
    uint32_t m_healthMonActionCounter;
    
    // Диагностика GPU
    std::map<uint32_t, GpuPcieInfo> m_gpuPcieMap;
    bool m_isRecoveryContext;
    bool m_healthcheckDetectedDevices;
    // ... ещё 30+ полей, связанных с GPU
    
    // Дерево PCIe
    std::shared_ptr<CPcieTreeLinux> m_pPcieTree;
    
    // Логирование событий
    CEventLogMgr* m_pEventLogMgr;
    
    // ... несколько других методов
    void HandleGpuEvents();
    void HandleNicEvents();
    void RunGpuDiag();
    // Всё зависит от всего
};
```

## Решение на Rust: компонуемые структуры состояния
```rust
// Пример: main.rs — состояние разбито на сфокусированные структуры

#[derive(Default)]
struct HealthMonitorState {
    alert_triggers: Vec<AlertTriggerInfo>,
    warn_triggers: Vec<WarnTriggerInfo>,
    health_monitor_action_counter: u32,
    health_monitor_has_boot_time_error: bool,
    // Только поля, связанные с монитором здоровья
}

#[derive(Default)]
struct GpuDiagState {
    gpu_pcie_map: HashMap<u32, GpuPcieInfo>,
    is_recovery_context: bool,
    healthcheck_detected_devices: bool,
    // Только поля, связанные с GPU
}

/// Фреймворк компонует эти состояния, а не владеет всем плоско
struct DiagFramework {
    ctx: DiagContext,             // Контекст выполнения
    args: Args,                   // Аргументы командной строки
    pcie_tree: Option<DeviceTree>,  // shared_ptr не нужен
    event_log_mgr: EventLogManager,   // Владеющий, а не сырой указатель
    fc_manager: FcManager,        // Управление кодами неисправностей
    health: HealthMonitorState,   // Состояние монитора здоровья — отдельная структура
    gpu: GpuDiagState,           // Состояние GPU — отдельная структура
}
```

### Главная мысль
- **Тестируемость**: каждую структуру состояния можно тестировать независимо
- **Читаемость**: `self.health.alert_triggers` против `m_alertTriggers` — владение очевидно
- **Рефакторинг без страха**: изменение `GpuDiagState` не может случайно повлиять на обработку монитора здоровья
- **Никакой «каши из методов»**: функции, которым нужно только состояние монитора здоровья, принимают `&mut HealthMonitorState`, а не весь фреймворк

----

# Кейс 5: трейт-объекты — когда они действительно уместны

- Не всё должно быть перечислением! **Система плагинов диагностических модулей** — подлинный случай для трейт-объектов
- Почему? Потому что диагностические модули **открыты для расширения** — новые модули можно добавлять, не меняя фреймворк

```rust
// Пример: framework.rs — Vec<Box<dyn DiagModule>> здесь уместен
pub struct DiagFramework {
    modules: Vec<Box<dyn DiagModule>>,        // Полиморфизм времени выполнения
    pre_diag_modules: Vec<Box<dyn DiagModule>>,
    event_log_mgr: EventLogManager,
    // ...
}

impl DiagFramework {
    /// Регистрация диагностического модуля — любой тип, реализующий DiagModule
    pub fn register_module(&mut self, module: Box<dyn DiagModule>) {
        info!("Registering module: {}", module.id());
        self.modules.push(module);
    }
}
```

### Когда использовать каждый паттерн

| **Сценарий** | **Паттерн** | **Почему** |
|-------------|-----------|--------|
| Фиксированный набор вариантов, известный на этапе компиляции | `enum` + `match` | Исчерпывающая проверка, без vtable |
| Типы аппаратных событий (Degrade, Fatal, Boot, ...) | `enum GpuEventKind` | Все варианты известны, важна производительность |
| Типы устройств PCIe (GPU, NIC, Switch, ...) | `enum PcieDeviceKind` | Фиксированный набор, у каждого варианта свои данные |
| Система плагинов/модулей (открыта для расширения) | `Box<dyn Trait>` | Новые модули добавляются без изменения фреймворка |
| Мокирование в тестах | `Box<dyn Trait>` | Внедрение тестовых двойников |

### Упражнение: подумайте, прежде чем переводить
Дан следующий код на C++:
```cpp
class Shape { public: virtual double area() = 0; };
class Circle : public Shape { double r; double area() override { return 3.14*r*r; } };
class Rect : public Shape { double w, h; double area() override { return w*h; } };
std::vector<std::unique_ptr<Shape>> shapes;
```
**Вопрос**: должен ли перевод на Rust использовать `enum Shape` или `Vec<Box<dyn Shape>>`?

<details><summary>Решение (нажмите, чтобы развернуть)</summary>

**Ответ**: `enum Shape` — потому что набор фигур **закрыт** (известен на этапе компиляции). `Box<dyn Shape>` понадобился бы, только если бы пользователи могли добавлять новые типы фигур во время выполнения.

```rust
// Правильный перевод на Rust:
enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { r } => std::f64::consts::PI * r * r,
            Shape::Rect { w, h } => w * h,
        }
    }
}

fn main() {
    let shapes: Vec<Shape> = vec![
        Shape::Circle { r: 5.0 },
        Shape::Rect { w: 3.0, h: 4.0 },
    ];
    for shape in &shapes {
        println!("Area: {:.2}", shape.area());
    }
}
// Вывод:
// Area: 78.54
// Area: 12.00
```

</details>

----

# Метрики перевода и извлечённые уроки

## Что мы узнали
1. **По умолчанию используйте диспетчеризацию через перечисления** — в примерно 100 тысячах строк C++ действительно понадобилось лишь около 25 использований `Box<dyn Trait>` (системы плагинов, моки для тестов). Остальные примерно 900 виртуальных методов стали перечислениями с `match`
2. **Паттерн арены устраняет циклы ссылок** — `shared_ptr` и `enable_shared_from_this` — признаки неясного владения. Сначала подумайте, кто **владеет** данными
3. **Передавайте контекст, а не храните указатели** — `DiagContext<'a>` с ограниченным временем жизни безопаснее и понятнее, чем хранение `Framework*` в каждом модуле
4. **Разбивайте божественные объекты** — если в структуре 30+ полей, то, скорее всего, это 3–4 структуры в одном плаще
5. **Компилятор — ваш напарник по парному программированию** — около 400 вызовов `dynamic_cast` означали около 400 потенциальных сбоев во время выполнения. Отсутствие аналогов `dynamic_cast` в Rust означает отсутствие ошибок типов во время выполнения

## Самые трудные места
- **Аннотации времён жизни**: правильно расставить заимствования непросто, если вы привыкли к сырым указателям, — но когда код компилируется, он корректен
- **Борьба с проверкой заимствований**: желание иметь `&mut self` в двух местах одновременно. Решение: разбить состояние на отдельные структуры
- **Противостояние буквальному переводу**: соблазн писать `Vec<Box<dyn Base>>` повсюду. Спросите себя: «Закрыт ли этот набор вариантов?» → Если да, используйте enum

## Рекомендации для команд C++
1. Начните с небольшого, самодостаточного модуля (а не с божественного объекта)
2. Сначала переводите структуры данных, потом поведение
3. Пусть компилятор вас направляет — его сообщения об ошибках отличны
4. Выбирайте `enum` раньше, чем `dyn Trait`
5. Используйте [Rust playground](https://play.rust-lang.org/), чтобы опробовать паттерны перед интеграцией

----

