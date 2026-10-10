# Модуль 1.2 — Введение

## Упражнение 1.2.1: Настройка окружения

В этом файле описано, как установить инструменты, которые понадобятся на курсе.

Все эти инструменты доступны для Linux, macOS и Windows.
Они нужны, чтобы писать и компилировать код на Rust, а также для удалённого наставничества.
*Важно: выполните эти инструкции до начала первого занятия.*
*Если у вас возникли проблемы с установкой, заранее свяжитесь с преподавателями. Мы бы не хотели тратить на проблемы с установкой время первого занятия.*

## Rust и Cargo

Сначала нам понадобится `rustc`, стандартный компилятор Rust.
`rustc` обычно не вызывают напрямую, а работают через `cargo`, менеджер пакетов Rust.
`rustup` устанавливает `rustc` и `cargo`.

Здесь всё просто: перейдите на <https://rustup.rs> и следуйте инструкциям.
Убедитесь, что вы устанавливаете последний стабильный тулчейн по умолчанию.
После установки выполните

```bash
rustc -V && cargo -V
```

Вывод должен выглядеть примерно так:

```bash
rustc 1.79.0 (129f3b996 2024-06-10)
cargo 1.79.0 (ffa9cf99a 2024-06-03)
```

С помощью Rustup можно устанавливать тулчейны и компоненты Rust. Подробнее:
- <https://rust-lang.github.io/rustup>
- <https://doc.rust-lang.org/cargo>

## Rustfmt и Clippy

Чтобы не спорить о стиле кода, Rust предоставляет собственный инструмент форматирования, Rustfmt.
Мы также будем использовать Clippy, набор линтов для анализа кода, который помогает ловить распространённые ошибки.
Clippy — очень полезный помощник при изучении Rust.
И Rustfmt, и Clippy по умолчанию устанавливаются вместе с Rustup.

Чтобы отформатировать проект с помощью Rustfmt, выполните:

```bash
cargo fmt
```

Чтобы запустить Clippy:

```bash
cargo clippy
```

Подробнее:
- Rustfmt: <https://github.com/rust-lang/rustfmt>
- Clippy: <https://github.com/rust-lang/rust-clippy>

## Visual Studio Code

На курсе мы будем писать код в Visual Studio Code (VS Code).
Конечно, вы можете использовать любой редактор, но если возникнут проблемы, мы не сможем гарантированно помочь с его настройкой.
Кроме того, VS Code позволяет совместно работать и проводить наставничество во время дистанционных занятий.

Инструкции по установке: <https://code.visualstudio.com/>.

Понадобится несколько расширений. Первое — Rust-Analyzer.
Инструкции по установке: <https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer>.
Rust-Analyzer много помогает при разработке и незаменим на старте.

Второе расширение — CodeLLDB.
Оно позволяет отлаживать код Rust прямо в VS Code.
Инструкции: <https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb>.

Если вы проходите курс дистанционно, установите также расширение Live Share.
С его помощью мы будем делиться кодом и помогать во время дистанционных занятий.
Инструкции по установке: <https://marketplace.visualstudio.com/items?itemName=MS-vsliveshare.vsliveshare>

Подробнее:
- <https://rust-analyzer.github.io/>
- [Live Share](http://aka.ms/vsls)

### Совет

В репозитории много проектов на Rust, и из-за сложной структуры Rust-Analyzer не может найти их автоматически.

Поэтому мы указали проекты вручную в файле `.vscode/settings.json`. Чтобы снизить нагрузку на компьютер, можно закомментировать проекты, которые не используются на курсе.

## Git

На курсе понадобится система контроля версий Git.
Если Git ещё не установлен, инструкции здесь: <https://git-scm.com/book/en/v2/Getting-Started-Installing-Git>.
Если вы новичок в Git, пригодится вводный материал GitHub <https://docs.github.com/en/get-started/using-git/about-git>, а также видео о работе с Git в VS Code: <https://www.youtube.com/watch?v=i_23KUAEtUM>.

Подробнее: <https://www.youtube.com/playlist?list=PLg7s6cbtAD15G8lNyoaYDuKZSKyJrgwB->

## Код курса

Когда всё установлено, склонируйте репозиторий с исходным кодом через Git.
Репозиторий находится здесь: <https://github.com/tweedegolf/rust-training>.

Инструкции по клонированию: <https://docs.github.com/en/get-started/getting-started-with-git/about-remote-repositories#cloning-with-https-urls>

### Пробный запуск

Когда код у вас на машине, перейдите в его каталог в терминале и выполните:

```
cd exercises/1-course-introduction/1-introduction/1-setup-your-installation
cargo run
```

В первый раз эта команда может выполняться долго: Cargo сначала загрузит индекс крейтов из реестра.
Затем он соберёт и запустит пакет `intro`, который находится в `exercises/1-course-introduction/1-introduction/1-setup-your-installation`.
Если всё прошло успешно, вы увидите такой вывод:

```
   Compiling intro v0.1.0 ([/path/to/rust-workshop]/exercises/1-course-introduction/1-introduction/1-setup-your-installation)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running `target/debug/intro`
🦀 Hello, world! 🦀
You've successfully compiled and run your first Rust project!
X: 2; Y: 2
```
Если Rust-Analyzer настроен правильно, над функцией `main` в `exercises/1-course-introduction/1-introduction/1-setup-your-installation/src/main.rs` появится кнопка «▶️ Run».
Если установлен CodeLLDB, рядом с ней появится кнопка «Debug», которая запускает отладочную сессию.
Попробуйте поставить точку останова, кликнув по номеру строки (появится красный кружок), и пошагово пройтись по коду: через функции, в них и из них, с помощью кнопок управления.
Значения переменных можно посмотреть, наведя на них курсор во время паузы, или раскрыв раздел «Local» в панели «Variables» слева во время отладки.

# Инструкции для модуля по FFI

*Этот раздел нужен, только если вы проходите один из модулей о FFI в Rust.*

Для работы с FFI нам понадобится скомпилировать C-код, поэтому нужен установленный компилятор C.
В упражнениях мы используем `clang`.

Необходимое условие: вызов `clang` в терминале должен работать. Если не работает, следуйте инструкциям для вашей платформы ниже.

## Linux

Для Debian и его производных (например, Ubuntu):
```bash
sudo apt update
sudo apt install clang
```

Для Arch:
```bash
sudo pacman -S clang
```

Для Fedora:
```bash
sudo dnf install clang
```

## Windows

Обязательно выберите опцию добавления установки в `PATH`.

С помощью winget:
```ps
winget install -i -e --id LLVM.LLVM
```

С помощью Chocolatey:
```ps
choco install llvm
```

Ручная установка:
- Перейдите на страницу релизов: https://github.com/llvm/llvm-project/releases
- Откройте один из свежих релизов
- Найдите файл LLVM-[ВЕРСИЯ]-win64.exe и скачайте его
- Запустите установщик

## macOS

С помощью Homebrew:
```bash
brew install llvm
```

Затем добавьте LLVM в `PATH`, например так:
```bash
echo 'export PATH="$(brew --prefix llvm)/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

# Инструкции для встраиваемых систем

*Этот раздел нужен, только если вы проходите один из модулей о встраиваемом Rust.*

## Аппаратура

Мы будем использовать [BBC micro:bit](https://microbit.org/buy/bbc-microbit-single) V2. Если он у вас уже есть, отлично; если нет, мы привезём его с собой.

Понадобится также кабель Micro-USB, но уверены, что он у вас найдётся.

Проверьте, что всё на месте. Если чего-то не хватает, свяжитесь с нами.

## Программное обеспечение

Теперь установим инструменты для прошивки микроконтроллера и просмотра кода.

Установите тулчейн `thumbv7em-none-eabihf` командой:
```bash
rustup target add thumbv7em-none-eabihf
```

Также установим пару инструментов для анализа бинарников:

```bash
rustup component add llvm-tools
cargo install cargo-binutils
```

Теперь установим [probe-rs](https://probe.rs). Следуйте [инструкции по установке](https://probe.rs/docs/getting-started/installation/). Probe-rs работает с отладочным интерфейсом micro:bit и позволяет прошивать приложение, выводить лог-сообщения, ставить точки останова и читать память устройства.

Для Linux команда такая:
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
```

Если вы работаете в Linux, нужно обновить правила udev.
В Ubuntu или Fedora выполните следующие команды в папке с тренингом, которую вы только что клонировали:

```bash
sudo cp 99-microbit-v2.rules /etc/udev/rules.d
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Может оказаться, что probe-rs находит два отладочных интерфейса. Такое бывает в Windows.
В этом случае откройте `.cargo/config.toml` и измените runner так, чтобы он указывал на конкретный отладчик, который нужно использовать. Значения должны совпадать с тем, что probe-rs показывает для одного из интерфейсов.
Не уверены? Выполните `probe-rs list`, чтобы получить список всех подключённых отладчиков.

## Пробный запуск

Перед началом проверим оборудование. Мы проверим микроконтроллер nRF52833 и датчик движения LSM303AGR, которые есть на micro:bit V2. Убедитесь, что у вас последняя версия исходного кода тренинга.

### Проверка оборудования

Подключите micro:bit V2 к компьютеру, включите его и выполните
```bash
cd ./exercises/1-course-introduction/1-introduction/2-embedded
cargo run --release
```

Если всё работает правильно, на дисплее должны появиться показания акселерометра. Если нет, не переживайте и свяжитесь с нами.

## Документация

Даташиты, руководства и схемы компонентов, которые мы используем в модулях по встраиваемым системам.
### BBC micro:bit V2
- [Документация по аппаратуре](https://tech.microbit.org/hardware/schematic/)
- [Схема](https://github.com/microbit-foundation/microbit-v2-hardware/blob/main/V2.21/MicroBit_V2.2.1_nRF52820%20schematic.PDF)
### nRF52833
- [Спецификация продукта nRF52833](https://docs.nordicsemi.com/r/bundle/ps_nrf52833/page/keyfeatures_html5.html)
### LSM303AGR
- [Даташит](https://www.st.com/resource/en/datasheet/lsm303agr.pdf)
