# Краткая справочная карточка

### Шпаргалка: команды под рукой

```bash
# ─── Build-скрипты ───
cargo build                          # Сначала компилирует build.rs, затем крейт
cargo build -vv                      # Подробный режим — показывает вывод build.rs

# ─── Кросс-компиляция ───
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.17
cross build --release --target aarch64-unknown-linux-gnu

# ─── Бенчмаркинг ───
cargo bench                          # Запустить все бенчмарки
cargo bench -- parse                 # Запустить бенчмарки, в названии которых есть «parse»
cargo flamegraph -- --args           # Построить флеймграф по бинарнику
perf record -g ./target/release/bin  # Записать данные perf
perf report                          # Просмотреть данные perf в интерактивном режиме

# ─── Покрытие ───
cargo llvm-cov --html                # HTML-отчёт
cargo llvm-cov --lcov --output-path lcov.info
cargo llvm-cov --workspace --fail-under-lines 80
cargo tarpaulin --out Html           # Альтернативный инструмент

# ─── Проверка безопасности ───
cargo +nightly miri test             # Запустить тесты под Miri
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test
valgrind --leak-check=full ./target/debug/binary
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test -Zbuild-std --target x86_64-unknown-linux-gnu

# ─── Аудит и цепочка поставок ───
cargo audit                          # Поиск известных уязвимостей
cargo audit --deny warnings          # Падать в CI при любой рекомендации
cargo deny check                     # Проверки лицензий, рекомендаций, запретов и источников
cargo deny list                      # Список всех лицензий в дереве зависимостей
cargo vet                            # Проверка доверия к цепочке поставок
cargo outdated --workspace           # Поиск устаревших зависимостей
cargo semver-checks                  # Обнаружение несовместимых изменений API
cargo geiger                         # Подсчёт unsafe в дереве зависимостей

# ─── Оптимизация бинарника ───
cargo bloat --release --crates       # Вклад каждого крейта в размер
cargo bloat --release -n 20          # 20 самых больших функций
cargo +nightly udeps --workspace     # Поиск неиспользуемых зависимостей
cargo machete                        # Быстрый поиск неиспользуемых зависимостей
cargo expand --lib module::name      # Посмотреть раскрытия макросов
cargo msrv find                      # Определить минимальную версию Rust
cargo clippy --fix --workspace --allow-dirty  # Автоисправление предупреждений линтеров

# ─── Оптимизация времени компиляции ───
export RUSTC_WRAPPER=sccache         # Общий кеш компиляции
sccache --show-stats                 # Статистика попаданий в кеш
cargo nextest run                    # Более быстрый запускатель тестов
cargo nextest run --retries 2        # Повторять нестабильные тесты

# ─── Платформенная инженерия ───
cargo check --target thumbv7em-none-eabihf   # Проверить сборку no_std
cargo build --target x86_64-pc-windows-gnu   # Кросс-компиляция под Windows
cargo xwin build --target x86_64-pc-windows-msvc  # Кросс-компиляция с ABI MSVC
cfg!(target_os = "linux")                    # cfg времени компиляции (даёт bool)

# ─── Релиз ───
cargo release patch --dry-run        # Предварительный просмотр релиза
cargo release patch --execute        # Повысить версию, закоммитить, поставить тег, опубликовать
cargo dist plan                      # Предварительный просмотр артефактов дистрибуции
```

### Таблица решений: какой инструмент когда

| Задача | Инструмент | Когда использовать |
|--------|------------|--------------------|
| Встроить хеш git / информацию о сборке | `build.rs` | Бинарнику нужна прослеживаемость |
| Скомпилировать C-код из Rust | крейт `cc` в `build.rs` | FFI к небольшим C-библиотекам |
| Сгенерировать код из схем | `prost-build` / `tonic-build` | Protobuf, gRPC, FlatBuffers |
| Слинковать системную библиотеку | `pkg-config` в `build.rs` | OpenSSL, libpci, systemd |
| Статический бинарник для Linux | `--target x86_64-unknown-linux-musl` | Развёртывание в контейнерах и облаке |
| Собрать под старую glibc | `cargo-zigbuild` | Совместимость с RHEL 7, CentOS 7 |
| Бинарник для ARM-сервера | `cross` или `cargo-zigbuild` | Развёртывание на Graviton/Ampere |
| Статистические бенчмарки | Criterion.rs | Обнаружение регрессий производительности |
| Быстрая проверка производительности | Divan | Профилирование во время разработки |
| Поиск горячих точек | `cargo flamegraph` / `perf` | Когда бенчмарк уже показал медленный код |
| Покрытие строк и ветвлений | `cargo-llvm-cov` | Пороги покрытия в CI, анализ пробелов |
| Быстрая проверка покрытия | `cargo-tarpaulin` | Локальная разработка |
| Обнаружение UB в Rust | Miri | Чистый Rust `unsafe`-код |
| Безопасность памяти при FFI с C | Valgrind memcheck | Смешанные кодовые базы Rust/C |
| Обнаружение гонок данных | TSan или Miri | Конкурентный `unsafe`-код |
| Обнаружение переполнения буфера | ASan | `unsafe` арифметика указателей |
| Обнаружение утечек | Valgrind или LSan | Долго работающие сервисы |
| Локальный аналог CI | `cargo-make` | Автоматизация рабочих сценариев разработчика |
| Проверки перед коммитом | `cargo-husky` или git-хуки | Ловить проблемы до push |
| Автоматические релизы | `cargo-release` + `cargo-dist` | Управление версиями и дистрибуция |
| Аудит зависимостей | `cargo-audit` / `cargo-deny` | Безопасность цепочки поставок |
| Соответствие лицензиям | `cargo-deny` (лицензии) | Коммерческие и корпоративные проекты |
| Доверие к цепочке поставок | `cargo-vet` | Среды с высокими требованиями безопасности |
| Поиск устаревших зависимостей | `cargo-outdated` | Плановое обслуживание |
| Обнаружение несовместимых изменений | `cargo-semver-checks` | Публикация библиотечных крейтов |
| Анализ дерева зависимостей | `cargo tree --duplicates` | Дедупликация и сокращение графа зависимостей |
| Анализ размера бинарника | `cargo-bloat` | Развёртывания с ограничением по размеру |
| Поиск неиспользуемых зависимостей | `cargo-udeps` / `cargo-machete` | Сокращение времени компиляции и размера |
| Настройка LTO | `lto = true` или `"thin"` | Оптимизация релизного бинарника |
| Бинарник, оптимизированный по размеру | `opt-level = "z"` + `strip = true` | Встраиваемые системы / WASM / контейнеры |
| Аудит использования unsafe | `cargo-geiger` | Соблюдение политики безопасности |
| Отладка макросов | `cargo-expand` | Отладка derive и `macro_rules` |
| Более быстрая линковка | линкер `mold` | Внутренний цикл разработчика |
| Кеш компиляции | `sccache` | Скорость CI и локальных сборок |
| Более быстрые тесты | `cargo-nextest` | Скорость тестов в CI и локально |
| Соответствие MSRV | `cargo-msrv` | Публикация библиотек |
| Библиотека `no_std` | `#![no_std]` + `default-features = false` | Встраиваемые системы, UEFI, WASM |
| Кросс-компиляция под Windows | `cargo-xwin` / MinGW | Сборки Linux → Windows |
| Платформенная абстракция | `#[cfg]` + паттерн с трейтами | Кодовые базы для нескольких ОС |
| Вызовы API Windows | `windows-sys` / крейт `windows` | Нативная функциональность Windows |
| Сквозной замер времени | `hyperfine` | Бенчмарки всего бинарника, сравнение до и после |
| Property-based тестирование | `proptest` | Поиск крайних случаев, устойчивость парсеров |
| Snapshot-тестирование | `insta` | Проверка больших структурированных выводов |
| Фаззинг с обратной связью по покрытию | `cargo-fuzz` | Поиск падений в парсерах |
| Проверка моделей конкурентности | `loom` | Структуры данных без блокировок, порядок атомарных операций |
| Тестирование комбинаций фич | `cargo-hack` | Крейты с несколькими фичами `#[cfg]` |
| Быстрые проверки UB (почти нативная скорость) | `cargo-careful` | Ворота безопасности в CI, легче Miri |
| Автопересборка при сохранении | `cargo-watch` | Внутренний цикл разработчика, быстрая обратная связь |
| Документация воркспейса | `cargo doc` + rustdoc | Поиск API, онбординг, проверка ссылок в CI |
| Воспроизводимые сборки | `--locked` + `SOURCE_DATE_EPOCH` | Проверка целостности релиза |
| Настройка кеша CI | `Swatinem/rust-cache@v2` | Сокращение времени сборки (с холодного старта → из кеша) |
| Политика линтов воркспейса | `[workspace.lints]` в Cargo.toml | Единые линты Clippy и компилятора во всех крейтах |
| Автоисправление предупреждений линтеров | `cargo clippy --fix` | Автоматическая очистка тривиальных проблем |

### Дополнительная литература

| Тема | Ресурс |
|------|--------|
| Build-скрипты Cargo | [Книга Cargo — Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) |
| Кросс-компиляция | [Кросс-компиляция в Rust](https://rust-lang.github.io/rustup/cross-compilation.html) |
| Инструмент `cross` | [cross-rs/cross](https://github.com/cross-rs/cross) |
| `cargo-zigbuild` | [документация cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) |
| Criterion.rs | [Руководство по Criterion](https://bheisler.github.io/criterion.rs/book/) |
| Divan | [документация Divan](https://github.com/nvzqz/divan) |
| `cargo-llvm-cov` | [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) |
| `cargo-tarpaulin` | [документация tarpaulin](https://github.com/xd009642/tarpaulin) |
| Miri | [Miri на GitHub](https://github.com/rust-lang/miri) |
| Санитайзеры в Rust | [документация rustc по санитайзерам](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html) |
| `cargo-make` | [книга cargo-make](https://sagiegurari.github.io/cargo-make/) |
| `cargo-release` | [документация cargo-release](https://github.com/crate-ci/cargo-release) |
| `cargo-dist` | [книга cargo-dist](https://axodotdev.github.io/cargo-dist/book/) |
| Оптимизация по профилю (PGO) | [Руководство Rust по PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html) |
| Флеймграфы | [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) |
| `cargo-deny` | [документация cargo-deny](https://embarkstudios.github.io/cargo-deny/) |
| `cargo-vet` | [документация cargo-vet](https://mozilla.github.io/cargo-vet/) |
| `cargo-audit` | [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit) |
| `cargo-bloat` | [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat) |
| `cargo-udeps` | [cargo-udeps](https://github.com/est31/cargo-udeps) |
| `cargo-geiger` | [cargo-geiger](https://github.com/geiger-rs/cargo-geiger) |
| `cargo-semver-checks` | [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) |
| `cargo-nextest` | [документация nextest](https://nexte.st/) |
| `sccache` | [sccache](https://github.com/mozilla/sccache) |
| Линкер `mold` | [mold](https://github.com/rui314/mold) |
| `cargo-msrv` | [cargo-msrv](https://github.com/foresterre/cargo-msrv) |
| LTO | [Опции генерации кода rustc](https://doc.rust-lang.org/rustc/codegen-options/index.html) |
| Профили Cargo | [Книга Cargo — Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) |
| `no_std` | [Embedded Book для Rust](https://docs.rust-embedded.org/book/) |
| Крейт `windows-sys` | [windows-rs](https://github.com/microsoft/windows-rs) |
| `cargo-xwin` | [документация cargo-xwin](https://github.com/rust-cross/cargo-xwin) |
| `cargo-hack` | [cargo-hack](https://github.com/taiki-e/cargo-hack) |
| `cargo-careful` | [cargo-careful](https://github.com/RalfJung/cargo-careful) |
| `cargo-watch` | [cargo-watch](https://github.com/watchexec/cargo-watch) |
| Кеш Rust в CI | [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) |
| Книга по rustdoc | [Rustdoc Book](https://doc.rust-lang.org/rustdoc/) |
| Условная компиляция | [Справочник Rust — cfg](https://doc.rust-lang.org/reference/conditional-compilation.html) |
| Встраиваемый Rust | [Awesome Embedded Rust](https://github.com/rust-embedded/awesome-embedded-rust) |
| `hyperfine` | [hyperfine](https://github.com/sharkdp/hyperfine) |
| `proptest` | [proptest](https://github.com/proptest-rs/proptest) |
| `insta` | [insta — snapshot-тестирование](https://insta.rs/) |
| `cargo-fuzz` | [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) |
| `loom` | [loom — тестирование конкурентности](https://github.com/tokio-rs/loom) |

---

*Создано как сопроводительный справочник к книгам «Паттерны Rust» и «Корректность на уровне типов».*

*Версия 1.3 — добавлены cargo-hack, cargo-careful, cargo-watch, cargo doc, воспроизводимые сборки, стратегии кеширования CI, итоговое упражнение и диаграмма зависимостей между главами для полноты.*
