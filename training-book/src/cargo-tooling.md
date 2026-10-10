# Модуль 3.3 — Инструментарий Cargo

<a href="https://training.tweede.golf/slides/3_3-cargo/" target="_blank">Слайды</a>

## Упражнение 3.3.1: FizzBuzz

В этом упражнении вы попрактикуетесь в написании модульного теста и используете средства бенчмаркинга Rust, чтобы оптимизировать приложение [FizzBuzz](https://ru.wikipedia.org/wiki/FizzBuzz). Понадобится [`cargo-criterion`](https://bheisler.github.io/criterion.rs/book/cargo_criterion/cargo_criterion.html) — инструмент, который запускает бенчмарки и создаёт наглядные отчёты. Установить его можно командой

```bash
cargo install cargo-criterion --version=1.1.0
```

### 3.3.1.A Тестируем FizzBuzz ⭐
Откройте `exercises/3-crate-engineering/3-cargo-tooling/1-fizzbuzz/src/lib.rs`. Создайте модульный тест, который проверяет корректность функции `fizz_buzz`. Для этого можно использовать макрос [`include_str`](https://doc.rust-lang.org/std/macro.include_str.html), чтобы включить файл `exercises/3-crate-engineering/3-cargo-tooling/1-fizzbuzz/fizzbuzz.out` в бинарник как `&str`. Каждая строка `fizzbuzz.out` содержит ожидаемый результат функции `fizz_buzz` для номера строки, переданного в качестве входных данных. Запустить тест можно командой

```bash
cargo test
```

По умолчанию тестовый харнесс (test harness) перехватывает весь вывод и отбрасывает его. Если вы хотите отлаживать тестовый код с помощью печати, можно запустить

```bash
cargo test -- --nocapture
```

чтобы харнесс не перехватывал вывод.


### 3.3.1.B Бенчмаркинг FizzBuzz ⭐⭐
Вероятно, вы заметили, что реализация `fizz_buzz` не очень оптимизирована. Мы используем `criterion`, чтобы измерить производительность `fizz_buzz`. Чтобы запустить бенчмарк, выполните следующую команду из каталога `exercises/3-crate-engineering/3-cargo-tooling/1-fizzbuzz/`:

```bash
cargo criterion
```

Эта команда запустит бенчмарки и выведет статистику в терминал. Она также создаёт HTML-отчёты с графиками, которые находятся в `target/criterion/reports`. Например, `target/criterion/reports/index.html` — это сводка по всем бенчмаркам. Откройте её в браузере и посмотрите.

Ваша задача — оптимизировать функцию `fizz_buzz` и с помощью `cargo-criterion` измерить эффект каждого изменения. Не бойтесь менять сигнатуру `fizz_buzz`, если, например, хотите уменьшить количество аллокаций. Однако убедитесь, что функция по-прежнему выдаёт правильный результат. Насколько быстро вы сможете сделать FizzBuzz?
