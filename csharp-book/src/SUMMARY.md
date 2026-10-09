# Содержание

[Введение](ch00-introduction.md)

---

# Часть I — Основы

- [1. Введение и мотивация](ch01-introduction-and-motivation.md)
- [2. Начало работы](ch02-getting-started.md)
    - [Справочник по ключевым словам *(необязательно)*](ch02-1-essential-keywords-reference.md)
- [3. Встроенные типы и переменные](ch03-built-in-types-and-variables.md)
    - [Истинная неизменяемость и иллюзии record](ch03-1-true-immutability-vs-record-illusions.md)
- [4. Управление потоком выполнения](ch04-control-flow.md)
- [5. Структуры данных и коллекции](ch05-data-structures-and-collections.md)
    - [Паттерны конструкторов](ch05-1-constructor-patterns.md)
    - [Коллекции — Vec, HashMap и итераторы](ch05-2-collections-vec-hashmap-and-iterators.md)
- [6. Перечисления и сопоставление с образцом](ch06-enums-and-pattern-matching.md)
    - [Исчерпывающее сопоставление и null-безопасность](ch06-1-exhaustive-matching-and-null-safety.md)
- [7. Владение и заимствование](ch07-ownership-and-borrowing.md)
    - [Глубокое погружение в безопасность памяти](ch07-1-memory-safety-deep-dive.md)
    - [Глубокое погружение во времена жизни](ch07-2-lifetimes-deep-dive.md)
    - [Умные указатели — не только единоличное владение](ch07-3-smart-pointers-beyond-single-ownership.md)
- [8. Крейты и модули](ch08-crates-and-modules.md)
    - [Управление пакетами — Cargo против NuGet](ch08-1-package-management-cargo-vs-nuget.md)
- [9. Обработка ошибок](ch09-error-handling.md)
    - [Типы ошибок уровня крейта и псевдонимы Result](ch09-1-crate-level-error-types-and-result-alias.md)
- [10. Трейты и обобщения](ch10-traits-and-generics.md)
    - [Ограничения обобщений](ch10-1-generic-constraints.md)
    - [Наследование против композиции](ch10-2-inheritance-vs-composition.md)
- [11. Трейты From и Into](ch11-from-and-into-traits.md)
- [12. Замыкания и итераторы](ch12-closures-and-iterators.md)
    - [Введение в макросы](ch12-1-macros-primer.md)

---

# Часть II — Конкурентность и системное программирование

- [13. Конкурентность](ch13-concurrency.md)
    - [Глубокое погружение в async/await](ch13-1-asyncawait-deep-dive.md)
- [14. Unsafe Rust и FFI](ch14-unsafe-rust-and-ffi.md)
    - [Тестирование](ch14-1-testing.md)

---

# Часть III — Миграция и лучшие практики

- [15. Паттерны миграции и разборы случаев](ch15-migration-patterns-and-case-studies.md)
    - [Основные крейты для разработчиков C#](ch15-1-essential-crates-for-c-developers.md)
    - [Стратегия постепенного внедрения](ch15-2-incremental-adoption-strategy.md)
- [16. Лучшие практики](ch16-best-practices.md)
    - [Сравнение производительности и миграция](ch16-1-performance-comparison-and-migration.md)
    - [Путь обучения и ресурсы](ch16-2-learning-path-and-resources.md)
    - [Экосистема инструментов Rust](ch16-3-rust-tooling-ecosystem.md)

---

# Итоговый проект

- [17. Итоговый проект: CLI-утилита для погоды на Rust](ch17-capstone-project.md)
