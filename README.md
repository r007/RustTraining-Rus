# Учебные книги по Rust

Восемь учебных курсов по Rust на русском языке: от основ языка и перехода с C/C++, C#, Python до асинхронного программирования, продвинутых паттернов, корректности на уровне типов и инженерной практики в продакшене.

Каждую книгу можно читать отдельно, но порядок в таблице ниже соответствует естественному пути обучения. Текст — на русском. Код, имена крейтов, команды и сообщения компилятора остаются на английском, как принято в экосистеме Rust.

> **Важно:** это учебные материалы, а не авторитетный справочник. Мы стремимся к точности, но ключевые детали сверяйте с [официальной документацией Rust](https://doc.rust-lang.org/) и [Rust Reference](https://doc.rust-lang.org/reference/).

**Читать онлайн:** [r007.github.io/RustTraining-Rus](https://r007.github.io/RustTraining-Rus/)
**Исходный код:** [github.com/r007/RustTraining-Rus](https://github.com/r007/RustTraining-Rus)

## 📖 С чего начать

Выберите книгу, которая соответствует вашему опыту. Книги сгруппированы по сложности:

| Уровень | Описание |
|---------|----------|
| 🟢 **Мост** | Изучение Rust для тех, кто пришёл из другого языка. Начните отсюда |
| 🔵 **Погружение** | Углублённое изучение крупной подсистемы Rust |
| 🟡 **Продвинутый** | Паттерны и приёмы для опытных Rust-разработчиков |
| 🟣 **Эксперт** | Передовые техники на уровне типов и корректности |
| 🟤 **Практики** | Инженерия, инструменты и готовность к продакшену |

| Книга | Уровень | Для кого |
|-------|---------|----------|
| [**Практический курс Rust**](https://r007.github.io/RustTraining-Rus/training-book/) | 🟢 Мост | Основы языка, многозадачность, веб, FFI, Python и встраиваемые системы |
| [**Rust для программистов на C/C++**](https://r007.github.io/RustTraining-Rus/c-cpp-book/) | 🟢 Мост | Семантика перемещения, RAII, FFI, embedded, no_std |
| [**Rust для программистов C#**](https://r007.github.io/RustTraining-Rus/csharp-book/) | 🟢 Мост | Swift / C# / Java → владение и система типов |
| [**Rust для программистов на Python**](https://r007.github.io/RustTraining-Rus/python-book/) | 🟢 Мост | От динамической типизации к статической, конкурентность без GIL |
| [**Async Rust**](https://r007.github.io/RustTraining-Rus/async-book/) | 🔵 Погружение | Tokio, потоки (streams), отмена и безопасность при отмене |
| [**Паттерны Rust**](https://r007.github.io/RustTraining-Rus/rust-patterns-book/) | 🟡 Продвинутый | Pin, аллокаторы, lock-free-структуры, unsafe |
| [**Корректность на уровне типов**](https://r007.github.io/RustTraining-Rus/type-driven-correctness-book/) | 🟣 Эксперт | Typestate, phantom-типы, capability-токены |
| [**Инженерные практики Rust**](https://r007.github.io/RustTraining-Rus/engineering-book/) | 🟤 Практики | Build-скрипты, кросс-компиляция, CI/CD, Miri |

Книги содержат диаграммы Mermaid, редактируемые песочницы Rust, упражнения и полнотекстовый поиск.

> **Совет:** читайте книги на [сайте](https://r007.github.io/RustTraining-Rus/): там есть навигация по боковой панели и поиск.
>
> **Локальный просмотр:** для работы без интернета или при участии в разработке (сначала [установите Rust](https://rustup.rs/)):
> ```
> git clone https://github.com/r007/RustTraining-Rus.git
> cd RustTraining-Rus
> cargo install mdbook mdbook-mermaid
> cargo xtask serve    # собирает все книги и запускает сервер: http://localhost:3000
> ```

---

## 💡 Источники вдохновения и благодарности

- [**The Rust Programming Language**](https://doc.rust-lang.org/book/): основа, на которой строится всё остальное
- [**Jon Gjengset**](https://www.youtube.com/c/JonGjengset): глубокие стримы о внутреннем устройстве Rust, серия `Crust of Rust`
- [**withoutboats**](https://without.boats/blog/): дизайн async, `Pin` и модель futures
- [**fasterthanlime (Amos)**](https://fasterthanli.me/): системное программирование с нуля и подробные разборы
- [**Mara Bos**](https://marabos.nl/): *Rust Atomics and Locks*, примитивы конкурентности
- [**Aleksey Kladov (matklad)**](https://matklad.github.io/): идеи о rust-analyzer, дизайне API и обработке ошибок
- [**Niko Matsakis**](https://smallcultfollowing.com/babysteps/): дизайн языка, внутреннее устройство borrow checker, Polonius
- [**Rust by Example**](https://doc.rust-lang.org/rust-by-example/) и [**Rustonomicon**](https://doc.rust-lang.org/nomicon/): практические паттерны и углублённое изучение unsafe
- [**This Week in Rust**](https://this-week-in-rust.org/): находки сообщества, которые повлияли на многие примеры
- [**Binary Musings — Tag(Rust)**](https://binarymusings.org/posts/category/rust/): глубокое погружение во внутреннее устройство Rust
- [**Tweede golf**](https://github.com/tweedegolf/rust-training): открытый курс Rust для профессионального обучения, на основе которого подготовлена книга [*Практический курс Rust*](training-book/)
- …и многие другие участники **сообщества Rust**, чьи статьи, доклады, RFC и обсуждения на форумах легли в основу этих материалов. Их слишком много, чтобы перечислить поимённо, но мы искренне благодарны каждому

---

## 🤝 Участие в проекте

Исправления опечаток, неточностей, перевода и примеров приветствуются. Порядок работы и стиль перевода описаны в [CONTRIBUTING.md](CONTRIBUTING.md). Сообщить об уязвимости можно по правилам из [SECURITY.md](SECURITY.md).

---

## 🔧 Для мейнтейнеров

<details>
<summary>Сборка, просмотр и редактирование книг локально</summary>

### Требования

Установите [Rust через **rustup**](https://rustup.rs/), если у вас его ещё нет, затем выполните:

```bash
cargo install mdbook@0.4.52 mdbook-mermaid@0.14.0
```

### Клонирование репозитория

```bash
git clone https://github.com/r007/RustTraining-Rus.git
cd RustTraining-Rus
```

### Сборка и запуск

```bash
cargo xtask build               # собрать все книги в site/ (для локального просмотра)
cargo xtask serve               # собрать и запустить на http://localhost:3000
cargo xtask deploy              # собрать все книги в docs/ (для GitHub Pages)
cargo xtask clean               # удалить site/ и docs/
```

Чтобы собрать или запустить одну книгу:

```bash
cd c-cpp-book && mdbook serve --open    # http://localhost:3000
```

### Публикация

Сайт публикуется на GitHub Pages при push в `main` через `.github/workflows/pages.yml`. Вручную ничего делать не нужно, но в настройках репозитория должен быть выбран источник **GitHub Actions** (Settings → Pages → Build and deployment).

### Docker

Контейнер для самостоятельного хостинга описан в [docker/README.md](docker/README.md).

</details>

---

## 📜 Лицензия

- Код (включая `xtask`, примеры и конфигурацию сборки) распространяется по лицензии [MIT](LICENSE).
- Текст книг и документация распространяются по лицензии [Creative Commons Attribution 4.0 International (CC BY 4.0)](LICENSE-DOCS), за исключением книги [*Практический курс Rust*](training-book/). Она основана на материалах Tweede golf, которые распространяются по лицензии [CC BY-SA 4.0](training-book/LICENSE), поэтому её перевод распространяется по той же лицензии.

Материалы основаны на оригинальном курсе Rust Training, распространяемом по тем же лицензиям. Авторские уведомления оригинала сохранены в файлах [LICENSE](LICENSE) и [LICENSE-DOCS](LICENSE-DOCS). Русский перевод и адаптацию выполнили участники проекта RustTraining-Rus.

Названия языков, инструментов и библиотек (Rust, Cargo, Tokio, .NET, mdBook и другие) принадлежат их правообладателям. Для юридических формулировок приоритет имеет английский текст файлов лицензий.
