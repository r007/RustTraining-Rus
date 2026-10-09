# Проверенные на практике приёмы 🟡

> **Чему вы научитесь:**
> - Проверенные в бою паттерны, которые не умещаются в одну главу
> - Типичные ловушки и способы их обойти — от нестабильного CI до раздувания бинарника
> - Быстрые приёмы, которые можно применить к любому проекту на Rust уже сегодня
>
> **Перекрёстные ссылки:** каждая глава этой книги — приёмы затрагивают все темы

Эта глава собирает инженерные паттерны, которые регулярно встречаются в продакшн-кодовых базах
на Rust. Каждый приём самостоятельный — читайте их в любом порядке.

---

### 1. Ловушка `deny(warnings)`

**Проблема**: `#![deny(warnings)]` в исходном коде ломает сборку, когда Clippy добавляет новые
линты: код, который компилировался вчера, сегодня не собирается.

**Решение**: используйте `CARGO_ENCODED_RUSTFLAGS` в CI вместо атрибута на уровне исходного кода:

```yaml
# CI: считаем предупреждения ошибками, не трогая исходный код
env:
  CARGO_ENCODED_RUSTFLAGS: "-Dwarnings"
```

Или используйте `[workspace.lints]` для более точной настройки:

```toml
# Cargo.toml
[workspace.lints.rust]
unsafe_code = "deny"

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
```

> См. [Инструменты времени компиляции, линты воркспейса](ch08-compile-time-and-developer-tools.md) для полного паттерна.

---

### 2. Компилируем один раз, тестируем везде

**Проблема**: `cargo test` пересобирает код при переключении между `--lib`, `--doc` и `--test`,
потому что они используют разные профили.

**Решение**: используйте `cargo nextest` для модульных и интеграционных тестов, а doc-тесты
запускайте отдельно:

```bash
cargo nextest run --workspace        # Быстро: параллельно, с кешем
cargo test --workspace --doc         # Doc-тесты (nextest их запустить не может)
```

> См. [Инструменты времени компиляции](ch08-compile-time-and-developer-tools.md) для настройки `cargo-nextest`.

---

### 3. Гигиена фич

**Проблема**: у библиотечного крейта `default = ["std"]`, но никто не тестирует
`--no-default-features`. Однажды пользователь для встраиваемых систем сообщает, что крейт
не компилируется.

**Решение**: добавьте `cargo-hack` в CI:

```yaml
- name: Матрица фич
  run: |
    cargo hack check --each-feature --no-dev-deps
    cargo check --no-default-features
    cargo check --all-features
```

> См. [`no_std` и проверка фич](ch09-no-std-and-feature-verification.md) для полного паттерна.

---

### 4. Дебаты о lock-файле — коммитить или игнорировать?

**Простое правило:**

| Тип крейта | Коммитить `Cargo.lock`? | Почему |
|------------|-------------------------|--------|
| Бинарник / приложение | **Да** | Воспроизводимые сборки |
| Библиотека | **Нет** (`.gitignore`) | Пусть потребители выбирают версии |
| Воркспейс, содержащий оба типа | **Да** | Побеждает бинарник |

Добавьте проверку в CI, чтобы lock-файл оставался актуальным:

```yaml
- name: Проверка lock-файла
  run: cargo update --locked  # Падает, если Cargo.lock устарел
```

---

### 5. Отладочные сборки с оптимизированными зависимостями

**Проблема**: отладочные сборки мучительно медленные, потому что зависимости (особенно `serde`,
`regex`) не оптимизированы.

**Решение**: оптимизируйте зависимости в dev-профиле, оставив свой код неоптимизированным для
быстрой пересборки:

```toml
# Cargo.toml
[profile.dev.package."*"]
opt-level = 2  # Оптимизировать все зависимости в dev-режиме
```

Это немного замедляет первую сборку, зато заметно ускоряет выполнение во время разработки.
Особенно полезно для сервисов с базами данных и парсеров.

> См. [Профили релиза](ch07-release-profiles-and-binary-size.md) для переопределений профиля по отдельным крейтам.

---

### 6. Постоянная перезапись кеша CI

**Проблема**: `Swatinem/rust-cache@v2` сохраняет новый кеш при каждом PR, раздувая хранилище
и замедляя восстановление.

**Решение**: сохраняйте кеш только из `main`, а восстанавливать его можно откуда угодно:

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    save-if: ${{ github.ref == 'refs/heads/main' }}
```

Для воркспейсов с несколькими бинарниками добавьте `shared-key`:

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: "ci-${{ matrix.target }}"
    save-if: ${{ github.ref == 'refs/heads/main' }}
```

> См. [Собираем всё вместе](ch11-putting-it-all-together-a-production-cic.md) для полного workflow.

---

### 7. `RUSTFLAGS` против `CARGO_ENCODED_RUSTFLAGS`

**Проблема**: `RUSTFLAGS="-Dwarnings"` применяется ко *всему* — включая build-скрипты и
процедурные макросы. Предупреждение в build.rs крейта `serde_derive` роняет ваш CI.

**Решение**: используйте `CARGO_ENCODED_RUSTFLAGS`, который действует только на верхнеуровневый крейт:

```bash
# ПЛОХО — ломается из-за предупреждений build-скриптов сторонних крейтов
RUSTFLAGS="-Dwarnings" cargo build

# ХОРОШО — затрагивает только ваш крейт
CARGO_ENCODED_RUSTFLAGS="-Dwarnings" cargo build

# ТОЖЕ ХОРОШО — линты воркспейса (Cargo.toml)
[workspace.lints.rust]
warnings = "deny"
```

---

### 8. Воспроизводимые сборки с `SOURCE_DATE_EPOCH`

**Проблема**: встраивание `chrono::Utc::now()` в `build.rs` делает сборки невоспроизводимыми:
каждая сборка даёт другой хеш бинарника.

**Решение**: учитывайте `SOURCE_DATE_EPOCH`:

```rust
// build.rs
let timestamp = std::env::var("SOURCE_DATE_EPOCH")
    .ok()
    .and_then(|s| s.parse::<i64>().ok())
    .unwrap_or_else(|| chrono::Utc::now().timestamp());
println!("cargo:rustc-env=BUILD_TIMESTAMP={timestamp}");
```

> См. [Build-скрипты](ch01-build-scripts-buildrs-in-depth.md) для полных паттернов build.rs.

---

### 9. Workflow дедупликации `cargo tree`

**Проблема**: `cargo tree --duplicates` показывает 5 версий `syn` и 3 версии `tokio-util`.
Время компиляции мучительно большое.

**Решение**: систематическая дедупликация:

```bash
# Шаг 1: находим дубликаты
cargo tree --duplicates

# Шаг 2: находим, кто тянет старую версию
cargo tree --invert --package syn@1.0.109

# Шаг 3: обновляем виновника
cargo update -p serde_derive  # Может подтянуть syn 2.x

# Шаг 4: если обновления нет, фиксируем через [patch]
# [patch.crates-io]
# old-crate = { git = "...", branch = "syn2-migration" }

# Шаг 5: проверяем
cargo tree --duplicates  # Должно стать короче
```

> См. [Управление зависимостями](ch06-dependency-management-and-supply-chain-s.md) для `cargo-deny`
> и безопасности цепочки поставок.

---

### 10. Дымовой тест перед push

**Проблема**: вы делаете push, CI работает 10 минут и падает из-за проблемы с форматированием.

**Решение**: запускайте быстрые проверки локально перед push:

```toml
# Makefile.toml (cargo-make)
[tasks.pre-push]
description = "Локальный дымовой тест перед push"
script = '''
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib
'''
```

```bash
cargo make pre-push  # < 30 секунд
git push
```

Или используйте git-хук pre-push:

```bash
#!/bin/sh
# .git/hooks/pre-push
cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings
```

> См. [Собираем всё вместе](ch11-putting-it-all-together-a-production-cic.md) для паттернов `Makefile.toml`.

---

### 🏋️ Упражнения

#### 🟢 Упражнение 1: примените три приёма

Выберите три приёма из этой главы и примените их к существующему проекту на Rust. Какой
оказал наибольший эффект?

<details>
<summary>Решение</summary>

Типичная комбинация с наибольшим эффектом:

1. **`[profile.dev.package."*"] opt-level = 2`** — немедленное улучшение скорости выполнения
   в dev-режиме (в 2–10× быстрее для кода, который много парсит)

2. **`CARGO_ENCODED_RUSTFLAGS`** — устраняет ложные падения CI из-за предупреждений сторонних крейтов

3. **`cargo-hack --each-feature`** — обычно находит хотя бы одну нерабочую комбинацию фич
   в любом проекте с 3+ фичами

```bash
# Применяем приём 5:
echo '[profile.dev.package."*"]' >> Cargo.toml
echo 'opt-level = 2' >> Cargo.toml

# Применяем приём 7 в CI:
# Заменяем RUSTFLAGS на CARGO_ENCODED_RUSTFLAGS

# Применяем приём 3:
cargo install cargo-hack
cargo hack check --each-feature --no-dev-deps
```
</details>

#### 🟡 Упражнение 2: устраните дубликаты в дереве зависимостей

Запустите `cargo tree --duplicates` на реальном проекте. Устраните хотя бы один дубликат.
Замерьте время компиляции до и после.

<details>
<summary>Решение</summary>

```bash
# До
time cargo build --release 2>&1 | tail -1
cargo tree --duplicates | wc -l  # Считаем строки с дубликатами

# Находим и исправляем один дубликат
cargo tree --duplicates
cargo tree --invert --package <duplicate-crate>@<old-version>
cargo update -p <parent-crate>

# После
time cargo build --release 2>&1 | tail -1
cargo tree --duplicates | wc -l  # Должно стать меньше

# Типичный результат: снижение времени компиляции на 5–15% за каждый устранённый
# дубликат (особенно для тяжёлых крейтов вроде syn, tokio)
```
</details>

### Ключевые выводы

- Используйте `CARGO_ENCODED_RUSTFLAGS` вместо `RUSTFLAGS`, чтобы не ломать build-скрипты сторонних крейтов
- `[profile.dev.package."*"] opt-level = 2` — самый эффективный приём для улучшения опыта разработки
- Настройка кеша (`save-if` только для main) предотвращает раздувание кеша CI в активных репозиториях
- `cargo tree --duplicates` + `cargo update` — бесплатный выигрыш во времени компиляции: делайте это раз в месяц
- Запускайте быстрые проверки локально через `cargo make pre-push`, чтобы не тратить время на круги через CI

---
