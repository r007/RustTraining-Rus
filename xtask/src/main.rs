use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Уровни каталога: (идентификатор, название, описание). Совпадают с таблицей уровней в README.
const LEVELS: &[(&str, &str, &str)] = &[
    (
        "bridge",
        "Мост",
        "Изучение Rust для тех, кто пришёл из другого языка",
    ),
    (
        "deep-dive",
        "Погружение",
        "Углублённое изучение крупной подсистемы Rust",
    ),
    (
        "advanced",
        "Продвинутый",
        "Паттерны и приёмы для опытных Rust-разработчиков",
    ),
    (
        "expert",
        "Эксперт",
        "Передовые техники на уровне типов и корректности",
    ),
    (
        "practices",
        "Практики",
        "Инженерия, инструменты и готовность к продакшену",
    ),
];

/// Теги каталога: (идентификатор для фильтра, отображаемое название).
const TAGS: &[(&str, &str)] = &[
    ("from-cpp", "Из C/C++"),
    ("from-csharp", "Из C#"),
    ("from-python", "Из Python"),
    ("ownership", "Владение"),
    ("async", "Async"),
    ("concurrency", "Конкурентность"),
    ("tokio", "Tokio"),
    ("types", "Система типов"),
    ("unsafe", "Unsafe"),
    ("ffi", "FFI"),
    ("embedded", "Embedded"),
    ("no-std", "no_std"),
    ("testing", "Тестирование"),
    ("errors", "Обработка ошибок"),
    ("project", "Итоговый проект"),
    ("tooling", "Инструменты"),
    ("cross-compile", "Кросс-компиляция"),
    ("ci-cd", "CI/CD"),
];

/// Книга сборника.
struct Book {
    /// Каталог с book.toml относительно корня репозитория; он же имя в site/ и часть адреса на сайте.
    slug: &'static str,
    title: &'static str,
    description: &'static str,
    /// Идентификатор уровня из LEVELS.
    level: &'static str,
    /// Идентификаторы тегов из TAGS.
    tags: &'static [&'static str],
}

const BOOKS: &[Book] = &[
    Book {
        slug: "training-book",
        title: "Практический курс Rust",
        description: "Основы языка, многозадачность, веб, FFI, Python и встраиваемые системы",
        level: "bridge",
        tags: &["from-python", "ownership", "async", "ffi", "embedded"],
    },
    Book {
        slug: "c-cpp-book",
        title: "Курс Rust для программистов на C/C++",
        description: "Семантика перемещения, RAII, FFI, embedded, no_std",
        level: "bridge",
        tags: &["from-cpp", "ownership", "ffi", "embedded", "no-std"],
    },
    Book {
        slug: "csharp-book",
        title: "Rust для программистов C#",
        description: "Для разработчиков Swift / C# / Java",
        level: "bridge",
        tags: &["from-csharp", "ownership", "errors", "project"],
    },
    Book {
        slug: "python-book",
        title: "Rust для программистов на Python",
        description: "От динамической к статической типизации, конкурентность без GIL",
        level: "bridge",
        tags: &["from-python", "types", "concurrency", "project"],
    },
    Book {
        slug: "async-book",
        title: "Async Rust: от Future до продакшена",
        description: "Tokio, потоки (streams), отмена и безопасность при отмене",
        level: "deep-dive",
        tags: &["async", "tokio", "concurrency", "project"],
    },
    Book {
        slug: "rust-patterns-book",
        title: "Паттерны Rust",
        description: "Pin, аллокаторы, lock-free-структуры, unsafe",
        level: "advanced",
        tags: &["types", "unsafe", "concurrency", "project"],
    },
    Book {
        slug: "type-driven-correctness-book",
        title: "Корректность на уровне типов",
        description: "Typestate, phantom-типы, capability-токены",
        level: "expert",
        tags: &["types", "embedded", "concurrency", "testing"],
    },
    Book {
        slug: "engineering-book",
        title: "Инженерные практики Rust",
        description: "Build-скрипты, кросс-компиляция, покрытие кода, CI/CD",
        level: "practices",
        tags: &["tooling", "cross-compile", "ci-cd", "testing", "no-std"],
    },
];

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask должен находиться во вложенном каталоге воркспейса")
        .to_path_buf()
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("build") => cmd_build(),
        Some("serve") => {
            cmd_build();
            cmd_serve();
        }
        Some("deploy") => cmd_deploy(),
        Some("clean") => cmd_clean(),
        Some("--help" | "-h" | "help") | None => print_usage(0),
        Some(other) => {
            eprintln!("Неизвестная команда: {other}\n");
            print_usage(1);
        }
    }
}

fn print_usage(code: i32) {
    let stream: &mut dyn Write = if code == 0 {
        &mut std::io::stdout()
    } else {
        &mut std::io::stderr()
    };
    let _ = writeln!(
        stream,
        "\
Использование: cargo xtask <КОМАНДА>

Команды:
  build    Собрать все книги в site/ (для локального просмотра)
  serve    Собрать и запустить сервер на http://localhost:3000
  deploy   Собрать все книги в docs/ (для GitHub Pages)
  clean    Удалить каталоги site/ и docs/"
    );
    std::process::exit(code);
}

// ── сборка ───────────────────────────────────────────────────────────

fn cmd_build() {
    if !check_mdbook() {
        eprintln!("Ошибка: 'mdbook' не найден в PATH. Установите его: https://rust-lang.github.io/mdBook/guide/installation.html");
        std::process::exit(1);
    }
    build_to("site");
}

fn cmd_deploy() {
    if !check_mdbook() {
        eprintln!("Ошибка: 'mdbook' не найден в PATH.");
        std::process::exit(1);
    }
    build_to("docs");
    println!("\nЧтобы опубликовать результат, закоммитьте docs/ и включите GitHub Pages: Settings → Pages → Deploy from a branch → /docs.");
}

fn check_mdbook() -> bool {
    Command::new("mdbook")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn build_to(dir_name: &str) {
    let root = project_root();
    let out = root.join(dir_name);

    if out.exists() {
        fs::remove_dir_all(&out).expect("не удалось очистить каталог вывода");
    }
    fs::create_dir_all(&out).expect("не удалось создать каталог вывода");

    println!("Сборка общего сайта в {dir_name}/\n");

    let mut ok = 0u32;
    for book in BOOKS {
        let slug = book.slug;
        let book_dir = root.join(slug);
        if !book_dir.is_dir() {
            eprintln!("  ✗ {slug}/ не найден, пропускаю");
            continue;
        }
        let dest = out.join(slug);
        let status = Command::new("mdbook")
            .args(["build", "--dest-dir"])
            .arg(&dest)
            .current_dir(&book_dir)
            .status()
            .expect("не удалось запустить mdbook: установлен ли он?");

        if status.success() {
            println!("  ✓ {slug}");
            ok += 1;
        } else {
            eprintln!("  ✗ {slug}: ОШИБКА СБОРКИ");
        }
    }
    println!("\n  собрано книг: {ok}/{}", BOOKS.len());

    write_landing(&out);

    // Отключаем обработку Jekyll на GitHub Pages
    fs::write(out.join(".nojekyll"), "").expect("не удалось создать .nojekyll");
    println!("\nГотово! Результат в {dir_name}/");
}

/// Копирует обложки в результат сборки: страница каталога ссылается на covers/<книга>.svg.
fn copy_covers(site: &Path) {
    let from = project_root().join("covers");
    let to = site.join("covers");
    fs::create_dir_all(&to).expect("не удалось создать каталог covers/");
    for entry in fs::read_dir(&from).expect("не удалось прочитать каталог covers/")
    {
        let entry = entry.expect("не удалось прочитать файл в covers/");
        fs::copy(entry.path(), to.join(entry.file_name())).expect("не удалось скопировать обложку");
    }
}

fn write_landing(site: &Path) {
    copy_covers(site);

    let mut sections = String::new();
    for &(level_id, level_title, level_desc) in LEVELS {
        let cards: String = BOOKS
            .iter()
            .filter(|b| b.level == level_id)
            .map(card)
            .collect();
        sections.push_str(&format!(
            r#"    <section class="cat cat-{level_id}" aria-labelledby="cat-{level_id}">
      <header>
        <h2 id="cat-{level_id}">{level_title}</h2>
        <p>{level_desc}</p>
      </header>
      <div class="grid">
{cards}      </div>
    </section>
"#
        ));
    }

    let chips: String = std::iter::once(r#"<button type="button" class="chip" data-filter="" aria-pressed="true">Все</button>"#.to_string())
        .chain(TAGS.iter().map(|(id, label)| {
            format!(r#"<button type="button" class="chip" data-filter="{id}" aria-pressed="false">{label}</button>"#)
        }))
        .collect::<Vec<_>>()
        .join("\n      ");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="Восемь учебных курсов по Rust на русском языке: от основ языка и перехода с C/C++, C#, Python до async, паттернов и инженерной практики.">
  <title>Учебные книги по Rust</title>
  <link rel="icon" href="covers/training-book.svg">
  <style>{CSS}</style>
</head>
<body>
  <header class="hero">
    <p class="eyebrow">🦀 Rust · от основ до продакшена</p>
    <h1>Книги по Rust на русском языке</h1>
    <p class="lead">Восемь курсов: от основ языка и перехода с C/C++, C#, Python до асинхронного программирования, продвинутых паттернов, корректности на уровне типов и инженерной практики.</p>
    <a class="repo-link" href="https://github.com/r007/RustTraining-Rus">Исходный код и вклад в проект на GitHub →</a>
  </header>

  <main>
    <div class="toolbar">
      <label class="search">
        <span class="visually-hidden">Поиск по книгам</span>
        <input id="search" type="search" placeholder="Поиск по названию, описанию и тегам" autocomplete="off">
      </label>
      <div class="chips" role="group" aria-label="Фильтр по тегам">
      {chips}
      </div>
    </div>

{sections}    <p class="empty" hidden>Ничего не найдено. Попробуйте другой запрос или тег.</p>
  </main>

  <footer>
    <p>Тексты книг — по лицензии CC BY 4.0; «Практический курс Rust» — по CC BY-SA 4.0. Код, скрипты сборки и страница каталога — по лицензии MIT. Подробности — в README.</p>
    <p>Собрано с помощью <a href="https://rust-lang.github.io/mdBook/">mdBook</a>.</p>
  </footer>
  <script>{JS}</script>
</body>
</html>
"#
    );
    fs::write(site.join("index.html"), html).expect("не удалось записать index.html");
    println!("  ✓ index.html");
}

fn card(book: &Book) -> String {
    let labels: Vec<&str> = book
        .tags
        .iter()
        .map(|id| {
            TAGS.iter()
                .find(|(tag, _)| tag == id)
                .map_or(*id, |(_, label)| *label)
        })
        .collect();
    let tags: String = labels.iter().map(|l| format!("<li>{l}</li>")).collect();
    let data_tags = book.tags.join(" ");
    let search = format!("{} {} {}", book.title, book.description, labels.join(" ")).to_lowercase();
    format!(
        r#"        <article class="card" data-tags="{data_tags}" data-search="{search}">
          <a class="cover" href="{slug}/" tabindex="-1" aria-hidden="true"><img src="covers/{slug}.svg" alt="" loading="lazy"></a>
          <div class="body">
            <h3><a href="{slug}/">{title}</a></h3>
            <p>{desc}</p>
            <ul class="tags">{tags}</ul>
            <div class="actions">
              <a class="btn primary" href="{slug}/">Читать онлайн</a>
            </div>
          </div>
        </article>
"#,
        slug = book.slug,
        title = book.title,
        desc = book.description,
    )
}

const CSS: &str = r#"
:root {
  color-scheme: light dark;
  --bg: #faf6ef; --surface: #ffffff; --text: #1f1a17; --muted: #6b625b;
  --border: rgba(60, 40, 20, .12); --shadow: 0 10px 30px rgba(60, 30, 10, .10);
  --accent: #b7410e; --on-accent: #ffffff; --chip: #f1ebe0;
  --c-bridge: #15803d; --c-deep-dive: #0e7490; --c-advanced: #b45309;
  --c-expert: #7e22ce; --c-practices: #0f766e;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --bg: #161210; --surface: #201b18; --text: #f1ebe4; --muted: #a79d93;
    --border: rgba(255, 240, 220, .09); --shadow: 0 10px 30px rgba(0, 0, 0, .40);
    --accent: #f08a5d; --on-accent: #1a1210; --chip: #2b2420;
    --c-bridge: #4ade80; --c-deep-dive: #22d3ee; --c-advanced: #fbbf24;
    --c-expert: #c084fc; --c-practices: #2dd4bf;
  }
}
* { box-sizing: border-box; }
[hidden] { display: none !important; }
.visually-hidden { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }
html { -webkit-text-size-adjust: 100%; }
body {
  margin: 0; color: var(--text); min-height: 100vh;
  font: 16px/1.55 -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  background:
    radial-gradient(1100px 520px at 50% -180px, color-mix(in srgb, var(--accent) 12%, transparent), transparent 70%),
    var(--bg);
}
a { color: var(--accent); }
.hero { padding: 56px 16px 28px; text-align: center; }
.eyebrow { margin: 0 0 8px; font-size: 13px; letter-spacing: .12em; text-transform: uppercase; color: var(--muted); }
.hero h1 { margin: 0 auto 12px; font-size: clamp(28px, 5vw, 42px); line-height: 1.15; max-width: 22ch; }
.lead { margin: 0 auto 16px; max-width: 62ch; color: var(--muted); }
.repo-link { font-weight: 600; text-decoration: none; }
main { max-width: 1120px; margin: 0 auto; padding: 8px 16px 40px; }
.toolbar { position: sticky; top: 0; z-index: 2; padding: 12px 0; background: var(--bg); }
.search input {
  width: 100%; padding: 12px 14px; font: inherit; color: var(--text); background: var(--surface);
  border: 1px solid var(--border); border-radius: 12px; outline: none;
}
.search input:focus { border-color: var(--accent); box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 25%, transparent); }
.chips { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 12px; }
.chip {
  font: inherit; font-size: 14px; padding: 6px 12px; border-radius: 999px; cursor: pointer;
  color: var(--text); background: var(--chip); border: 1px solid var(--border);
}
.chip[aria-pressed="true"] { background: var(--accent); border-color: var(--accent); color: var(--on-accent); }
.cat { margin-top: 36px; }
.cat header { margin-bottom: 16px; border-left: 4px solid var(--stripe); padding-left: 14px; }
.cat h2 { margin: 0; font-size: 22px; }
.cat header p { margin: 4px 0 0; color: var(--muted); }
.cat-bridge { --stripe: var(--c-bridge); }
.cat-deep-dive { --stripe: var(--c-deep-dive); }
.cat-advanced { --stripe: var(--c-advanced); }
.cat-expert { --stripe: var(--c-expert); }
.cat-practices { --stripe: var(--c-practices); }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 20px; }
.card {
  display: flex; flex-direction: column; overflow: hidden; background: var(--surface);
  border: 1px solid var(--border); border-radius: 16px; box-shadow: var(--shadow);
  transition: transform .15s ease, box-shadow .15s ease;
}
.card:hover { transform: translateY(-3px); }
.cover { display: block; aspect-ratio: 16 / 9; background: var(--chip); }
.cover img { display: block; width: 100%; height: 100%; object-fit: cover; }
.body { display: flex; flex: 1; flex-direction: column; gap: 10px; padding: 16px 18px 18px; }
.body h3 { margin: 0; font-size: 18px; line-height: 1.3; }
.body h3 a { color: var(--text); text-decoration: none; }
.body h3 a:hover { text-decoration: underline; }
.body h3 a:focus-visible, .btn:focus-visible, .chip:focus-visible, .repo-link:focus-visible {
  outline: 2px solid var(--accent); outline-offset: 2px;
}
.body p { margin: 0; color: var(--muted); font-size: 15px; }
.tags { display: flex; flex-wrap: wrap; gap: 6px; margin: auto 0 0; padding: 0; list-style: none; }
.tags li { font-size: 12px; padding: 3px 8px; border-radius: 6px; background: var(--chip); color: var(--muted); }
.actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 4px; }
.btn {
  display: inline-block; padding: 8px 14px; border-radius: 10px; font-size: 14px; font-weight: 600;
  text-decoration: none; color: var(--text); border: 1px solid var(--border); background: var(--surface);
}
.btn.primary { background: var(--accent); border-color: var(--accent); color: var(--on-accent); }
.btn:hover { filter: brightness(1.05); }
.empty { margin-top: 36px; text-align: center; color: var(--muted); }
footer { max-width: 1120px; margin: 0 auto; padding: 24px 16px 48px; color: var(--muted); font-size: 14px; border-top: 1px solid var(--border); }
footer p { margin: 8px 0; }
@media (max-width: 480px) {
  .hero { padding-top: 36px; }
  .grid { grid-template-columns: 1fr; }
}
"#;

const JS: &str = r#"
(function () {
  var search = document.getElementById('search');
  var chips = document.querySelectorAll('.chip');
  var cards = document.querySelectorAll('.card');
  var sections = document.querySelectorAll('.cat');
  var empty = document.querySelector('.empty');
  var tag = '';
  function apply() {
    var q = search.value.trim().toLowerCase();
    cards.forEach(function (card) {
      var byTag = !tag || card.dataset.tags.split(' ').indexOf(tag) !== -1;
      var byText = !q || card.dataset.search.indexOf(q) !== -1;
      card.hidden = !(byTag && byText);
    });
    sections.forEach(function (sec) {
      sec.hidden = !sec.querySelector('.card:not([hidden])');
    });
    empty.hidden = !!document.querySelector('.card:not([hidden])');
  }
  chips.forEach(function (chip) {
    chip.addEventListener('click', function () {
      tag = chip.dataset.filter;
      chips.forEach(function (c) { c.setAttribute('aria-pressed', String(c === chip)); });
      apply();
    });
  });
  search.addEventListener('input', apply);
})();
"#;

enum ResolveResult {
    File(PathBuf),
    Redirect(String),
    NotFound,
}

/// Преобразует `request_target` (путь HTTP-запроса, например `/foo/bar?x=1`) в файл внутри `site_canon`.
/// Возвращает `ResolveResult::File`, если файл найден, `Redirect`, если для каталога нужен завершающий слеш,
/// и `NotFound` для попыток обхода каталогов и отсутствующих файлов.
///
/// ПРИМЕЧАНИЕ: функция сохраняет и усиливает многоуровневую защиту из PR#18:
/// 1. Декодирование percent-encoding через `percent_decode_path`.
/// 2. Отказ от нулевых байтов.
/// 3. Блокировка обхода каталогов (`..`).
/// 4. Защита от выхода за пределы каталога через символические ссылки: канонизация и проверка префикса.
fn resolve_site_file(site_canon: &Path, request_target: &str) -> ResolveResult {
    let path_only = match request_target
        .split('?')
        .next()
        .and_then(|s| s.split('#').next())
    {
        Some(p) => p,
        None => return ResolveResult::NotFound,
    };

    // [Безопасность] Декодируем percent-encoding и отклоняем нулевые байты (из PR#18)
    let decoded = percent_decode_path(path_only);
    if decoded.as_bytes().contains(&0) {
        return ResolveResult::NotFound;
    }

    let rel = decoded.trim_start_matches('/');
    let mut file_path = site_canon.to_path_buf();
    if !rel.is_empty() {
        for seg in rel.split('/').filter(|s| !s.is_empty()) {
            // [Безопасность] Блокируем обход каталогов (из PR#18)
            if seg == ".." {
                return ResolveResult::NotFound;
            }
            file_path.push(seg);
        }
    }

    if file_path.is_dir() {
        // Если путь указывает на каталог без завершающего слеша, перенаправляем, чтобы относительные ссылки работали.
        if !request_target.ends_with('/') && !request_target.is_empty() {
            return ResolveResult::Redirect(format!("{path_only}/"));
        }
        file_path.push("index.html");
    }

    // [Безопасность] Канонизируем путь и проверяем, что он остаётся внутри site_canon (из PR#18)
    let real = match fs::canonicalize(&file_path) {
        Ok(r) => r,
        Err(_) => return ResolveResult::NotFound,
    };

    if !real.starts_with(site_canon) || !real.is_file() {
        return ResolveResult::NotFound;
    }

    ResolveResult::File(real)
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn percent_decode_path(input: &str) -> String {
    let mut decoded = Vec::with_capacity(input.len());
    let b = input.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(hi), Some(lo)) = (hex_val(b[i + 1]), hex_val(b[i + 2])) {
                decoded.push(hi << 4 | lo);
                i += 3;
                continue;
            }
        }
        decoded.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

// ── сервер ───────────────────────────────────────────────────────────

fn cmd_serve() {
    let site = project_root().join("site");
    let site_canon = fs::canonicalize(&site).expect(
        "каталог site/ не найден: сначала выполните `cargo xtask build` (команда `cargo xtask serve` запускает сборку автоматически)",
    );
    let addr = "127.0.0.1:3000";
    let listener = TcpListener::bind(addr).expect("не удалось занять порт 3000");

    // Обрабатываем Ctrl+C, чтобы cargo не сообщал об ошибке
    ctrlc_exit();

    println!("\nСервер запущен: http://localhost:3000  (Ctrl+C — остановить)");

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        let request = String::from_utf8_lossy(&buf[..n]);

        let path = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/");

        match resolve_site_file(&site_canon, path) {
            ResolveResult::File(file_path) => {
                let body = fs::read(&file_path).unwrap_or_default();
                let mime = guess_mime(&file_path);
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(&body);
            }
            ResolveResult::Redirect(new_path) => {
                let header = format!(
                    "HTTP/1.1 301 Moved Permanently\r\nLocation: {new_path}\r\nContent-Length: 0\r\n\r\n"
                );
                let _ = stream.write_all(header.as_bytes());
            }
            ResolveResult::NotFound => {
                let body = b"404 Not Found";
                let header = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(body);
            }
        }
    }
}

/// Устанавливает обработчик Ctrl+C, который завершает программу с кодом 0.
/// Без него ОС завершает процесс с STATUS_CONTROL_C_EXIT.
fn ctrlc_exit() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    })
    .expect("не удалось установить обработчик Ctrl-C");
}

fn guess_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

// ── очистка ──────────────────────────────────────────────────────────

fn cmd_clean() {
    let root = project_root();
    for dir_name in ["site", "docs"] {
        let dir = root.join(dir_name);
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("не удалось удалить каталог");
            println!("Удалено: {dir_name}/");
        }
    }
}
