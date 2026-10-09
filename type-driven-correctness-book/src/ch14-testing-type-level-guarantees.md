# Тестирование гарантий на уровне типов 🟡

> **Что вы узнаете:** как проверить, что некорректный код *не компилируется* (trybuild), как подвергнуть фаззингу проверенные границы (proptest), как проверить инварианты RAII и как доказать бесплатность абстракций с помощью `cargo-show-asm`.
>
> **Перекрёстные ссылки:** [гл. 03](ch03-single-use-types-cryptographic-guarantee.md) (compile-fail для nonce), [гл. 07](ch07-validated-boundaries-parse-dont-validate.md) (proptest для границ), [гл. 05](ch05-protocol-state-machines-type-state-for-r.md) (RAII для сессий)

## Тестирование гарантий на уровне типов

Паттерны корректности по построению переносят ошибки из времени выполнения на этап компиляции. Но как **проверить**, что некорректный код действительно не компилируется? И как убедиться, что проверенные границы выдерживают фаззинг? В этой главе рассматриваются инструменты тестирования, которые дополняют корректность на уровне типов.

### Compile-fail тесты с `trybuild`

Крейт [`trybuild`](https://crates.io/crates/trybuild) позволяет утверждать, что определённый код **не должен компилироваться**. Это важно для сохранения инвариантов уровня типов при рефакторинге: если кто-то случайно добавит `Clone` к вашему одноразовому `Nonce`, compile-fail тест это поймает.

**Настройка:**

```toml
# Cargo.toml
[dev-dependencies]
trybuild = "1"
```

**Файл теста (`tests/compile_fail.rs`):**

```rust,ignore
#[test]
fn type_safety_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
```

**Тест: повторное использование nonce не должно компилироваться (`tests/ui/nonce_reuse.rs`):**

```rust,ignore
// tests/ui/nonce_reuse.rs
use my_crate::Nonce;

fn main() {
    let nonce = Nonce::new();
    encrypt(nonce);
    encrypt(nonce); // должно быть ошибкой: использование перемещённого значения
}

fn encrypt(_n: Nonce) {}
```

**Ожидаемая ошибка (`tests/ui/nonce_reuse.stderr`):**

```text
error[E0382]: use of moved value: `nonce`
 --> tests/ui/nonce_reuse.rs:6:13
  |
4 |     let nonce = Nonce::new();
  |         ----- move occurs because `nonce` has type `Nonce`, which does not implement the `Copy` trait
5 |     encrypt(nonce);
  |             ----- value moved here
6 |     encrypt(nonce); // должно быть ошибкой: использование перемещённого значения
  |             ^^^^^ value used here after move
```

**Другие compile-fail тесты по главам:**

| Паттерн (глава) | Проверяемое утверждение | Файл |
|-----------------|-------------------------|------|
| Одноразовый nonce (гл. 03) | Нельзя использовать nonce дважды | `nonce_reuse.rs` |
| Capability-токен (гл. 04) | Нельзя вызвать `admin_op()` без токена | `missing_token.rs` |
| Typestate (гл. 05) | Нельзя вызвать `send_command()` у `Session<Idle>` | `wrong_state.rs` |
| Размерные типы (гл. 06) | Нельзя сложить `Celsius + Rpm` | `unit_mismatch.rs` |
| Sealed-трейт (приём 2) | Внешний крейт не может реализовать sealed-трейт | `unseal_attempt.rs` |
| Non-exhaustive (приём 3) | Внешний match без подстановки не компилируется | `missing_wildcard.rs` |

**Интеграция с CI:**

```yaml
# .github/workflows/ci.yml
- name: Запуск compile-fail тестов
  run: cargo test --test compile_fail
```

### Property-based тестирование проверенных границ

Проверенные границы (гл. 07) разбирают данные один раз и отвергают некорректный ввод. Но откуда знать, что проверка ловит **все** некорректные входные данные? Property-based тестирование с [`proptest`](https://crates.io/crates/proptest) генерирует тысячи случайных входных данных, чтобы нагрузить границу:

```toml
# Cargo.toml
[dev-dependencies]
proptest = "1"
```

```rust,ignore
use proptest::prelude::*;

/// Из гл. 07: ValidFru оборачивает полезную нагрузку FRU, соответствующую спецификации.
/// В этих тестах используется полный ValidFru из гл. 07 с методами board_area(),
/// product_area() и format_version().
/// Примечание: в гл. 07 определён TryFrom<RawFruData>, поэтому сначала оборачиваем сырые байты.

proptest! {
    /// Любая последовательность байтов, прошедшая проверку, должна использоваться без паники.
    #[test]
    fn valid_fru_never_panics(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
        if let Ok(fru) = ValidFru::try_from(RawFruData(data)) {
            // Эти методы не должны паниковать для проверенного FRU
            // (методы из реализации ValidFru в гл. 07):
            let _ = fru.format_version();
            let _ = fru.board_area();
            let _ = fru.product_area();
        }
    }

    /// Круговой тест: format_version сохраняется при повторном разборе.
    #[test]
    fn fru_round_trip(data in valid_fru_strategy()) {
        let raw = RawFruData(data.clone());
        let fru = ValidFru::try_from(raw).unwrap();
        let version = fru.format_version();
        // Повторно разбираем те же байты: версия должна совпасть
        let reparsed = ValidFru::try_from(RawFruData(data)).unwrap();
        prop_assert_eq!(version, reparsed.format_version());
    }
}

/// Произвольная стратегия: генерирует векторы байтов, удовлетворяющие заголовку FRU.
/// Формат заголовка соответствует проверке `TryFrom<RawFruData>` из гл. 07:
///   - Байт 0: версия = 0x01
///   - Байты 1–6: смещения областей (×8 = фактическое смещение в байтах)
///   - Байт 7: контрольная сумма (сумма байтов 0–7 = 0 по модулю 256)
/// Тело случайное, но достаточно большое, чтобы смещения оставались в пределах.
fn valid_fru_strategy() -> impl Strategy<Value = Vec<u8>> {
    let header = vec![0x01, 0x00, 0x01, 0x02, 0x00, 0x00, 0x00];
    proptest::collection::vec(any::<u8>(), 64..256)
        .prop_map(move |body| {
            let mut fru = header.clone();
            let sum: u8 = fru.iter().fold(0u8, |a, &b| a.wrapping_add(b));
            fru.push(0u8.wrapping_sub(sum));
            fru.extend_from_slice(&body);
            fru
        })
}
```

**Пирамида тестирования для кода с корректностью по построению:**

```text
┌───────────────────────────────────┐
│    Compile-fail тесты (trybuild)  │ ← «Некорректный код не должен компилироваться»
├───────────────────────────────────┤
│  Property-тесты (proptest/quickcheck) │ ← «Корректные входные данные никогда не вызывают панику»
├───────────────────────────────────┤
│    Модульные тесты (#[test])       │ ← «Конкретные входные данные дают ожидаемый результат»
├───────────────────────────────────┤
│    Система типов (гл. 02–13)      │ ← «Целые классы ошибок не могут существовать»
└───────────────────────────────────┘
```

### Проверка RAII

RAII (приём 12) гарантирует очистку. Чтобы это проверить, убедимся, что реализация `Drop` действительно срабатывает:

```rust,ignore
use std::sync::atomic::{AtomicBool, Ordering};

// ПРИМЕЧАНИЕ: эти тесты используют глобальный AtomicBool, поэтому их нельзя запускать
// параллельно друг с другом. Используйте `#[serial_test::serial]` или запускайте с
// `cargo test -- --test-threads=1`. Либо используйте `Arc<AtomicBool>` для каждого теста,
// передаваемый через замыкание, чтобы полностью обойтись без глобального состояния.
static DROPPED: AtomicBool = AtomicBool::new(false);

struct TestSession;
impl Drop for TestSession {
    fn drop(&mut self) {
        DROPPED.store(true, Ordering::SeqCst);
    }
}

#[test]
fn session_drops_on_early_return() {
    DROPPED.store(false, Ordering::SeqCst);
    let result: Result<(), &str> = (|| {
        let _session = TestSession;
        Err("имитация сбоя")?;
        Ok(())
    })();
    assert!(result.is_err());
    assert!(DROPPED.load(Ordering::SeqCst), "Drop должен сработать при раннем возврате");
}

#[test]
fn session_drops_on_panic() {
    DROPPED.store(false, Ordering::SeqCst);
    let result = std::panic::catch_unwind(|| {
        let _session = TestSession;
        panic!("имитация паники");
    });
    assert!(result.is_err());
    assert!(DROPPED.load(Ordering::SeqCst), "Drop должен сработать при панике");
}
```

### Применение к вашей кодовой базе

Вот приоритизированный план добавления тестов на уровне типов в рабочую область:

| Крейт | Тип теста | Что тестировать |
|-------|-----------|-----------------|
| `protocol_lib` | Compile-fail | `Session<Idle>` не может вызвать `send_command()` |
| `protocol_lib` | Property | Любая последовательность байтов → `TryFrom` либо успешно завершается, либо возвращает Err (без паники) |
| `thermal_diag` | Compile-fail | Нельзя создать `FanReading` без миксина `HasSpi` |
| `accel_diag` | Property | Разбор показаний GPU: случайные байты → либо проверены, либо отвергнуты |
| `config_loader` | Property | Случайные строки → `FromStr` для `DiagLevel` никогда не паникует |
| `pci_topology` | Compile-fail | `Register<Width16>` нельзя передать туда, где ожидается `Width32` |
| `event_handler` | Compile-fail | Токен аудита нельзя клонировать |
| `diag_framework` | Compile-fail | `DerBuilder<Missing, _>` не может вызвать `finish()` |

### Бесплатные абстракции: доказательство через ассемблер

Распространённый вопрос: «Добавляют ли newtype и phantom-типы накладные расходы во время выполнения?» Ответ: **нет**. Они компилируются в такой же ассемблер, как и голые примитивы. Вот как это проверить.

**Настройка:**

```bash
cargo install cargo-show-asm
```

**Пример: newtype против голого u32:**

```rust,ignore
// src/lib.rs
#[derive(Clone, Copy)]
pub struct Rpm(pub u32);

#[derive(Clone, Copy)]
pub struct Celsius(pub f64);

// Арифметика через newtype
#[inline(never)]
pub fn add_rpm(a: Rpm, b: Rpm) -> Rpm {
    Rpm(a.0 + b.0)
}

// Голая арифметика (для сравнения)
#[inline(never)]
pub fn add_raw(a: u32, b: u32) -> u32 {
    a + b
}
```

**Запуск:**

```bash
cargo asm my_crate::add_rpm
cargo asm my_crate::add_raw
```

**Результат: одинаковый ассемблер:**

```asm
; add_rpm (newtype)           ; add_raw (голый u32)
my_crate::add_rpm:            my_crate::add_raw:
  lea eax, [rdi + rsi]         lea eax, [rdi + rsi]
  ret                          ret
```

Обёртка `Rpm` полностью стирается на этапе компиляции. То же верно для `PhantomData<S>` (ноль байт), токенов `ZST` (ноль байт) и всех остальных маркеров уровня типов, которые используются в этом руководстве.

**Проверка своих типов:**

```bash
# Показать ассемблер для конкретной функции
cargo asm --lib ipmi_lib::session::execute

# Показать, что PhantomData не добавляет байтов
cargo asm --lib --rust ipmi_lib::session::IpmiSession
```

> **Ключевой вывод:** каждый паттерн в этом руководстве имеет **нулевую стоимость во время выполнения**. Система типов делает всю работу и полностью стирается при компиляции. Вы получаете безопасность Haskell с производительностью C.

## Ключевые выводы

1. **trybuild проверяет, что некорректный код не компилируется**: это важно для сохранения инвариантов уровня типов при рефакторинге.
2. **proptest подвергает фаззингу границы валидации**: генерирует тысячи случайных входных данных, чтобы нагрузить реализации `TryFrom`.
3. **Проверка RAII подтверждает, что Drop выполняется**: счётчики Arc или флаги-заглушки доказывают, что очистка произошла.
4. **cargo-show-asm доказывает нулевую стоимость**: phantom-типы, ZST и newtype дают тот же ассемблер, что и голый C.
5. **Добавляйте compile-fail тесты для каждого «невозможного» состояния**: если кто-то случайно выведет `Clone` для одноразового типа, тест это поймает.

---

*Конец книги «Корректность на уровне типов в Rust»*
