# Модуль 5.3 — Rust в браузере

## Упражнение 5.3.1: Lettuce Crop на WebAssembly

В упражнении 5.1.1 мы создали веб-сервер, который предоставляет сервис обрезки изображений. Но нужно ли делать обрезку на сервере? Не было бы гораздо приватнее, если бы обрезка выполнялась в браузере пользователя, а не загружала изображения на наш внешний сервер?

В этом упражнении мы создадим новую версию сайта Lettuce Crop, который обрезает изображения с помощью [WebAssembly](https://webassembly.org/). WebAssembly позволяет запускать скомпилированный код в безопасной изолированной среде (песочнице) в браузере. Это значит, что выделенный сервер больше не понадобится: сайт будет состоять только из статических файлов, которые можно разместить на любом HTTP-сервере. Его даже можно бесплатно опубликовать на [GitHub Pages](https://pages.github.com/)!

### 5.3.1.A Сборка с помощью Wasm Pack
В `exercises/5-rust-for-web/3-rust-in-the-browser/1-lettuce-crop-wasm` уже настроен базовый проект WebAssembly. Как видно в `Cargo.toml`, проект сконфигурирован как динамическая библиотека (`"cdylib"`). Мы также добавили в зависимости крейт `wasm-bindgen`, который используется для генерации привязок WebAssembly.

Для сборки проекта используем `wasm-pack`. Сначала установите `wasm-pack`:
```
cargo install wasm-pack
```
Затем соберите проект с помощью `wasm-pack`. Поскольку мы хотим использовать его в браузере, задаём [target](https://rustwasm.github.io/docs/wasm-pack/commands/build.html#target) `web` и указываем, что сгенерированные файлы нужно поместить в папку `assets/pkg`:
```
wasm-pack build --target web --out-dir assets/pkg
```

Теперь в папке `assets/pkg` должно появиться несколько файлов:
- Файл `.wasm`, который содержит скомпилированный код WebAssembly;
- Несколько файлов `.d.ts`, которые описывают типы TypeScript для сгенерированных привязок;
- Файл `.js`, который содержит JavaScript-привязки к нашему бинарнику WebAssembly.

### 5.3.1.B Взаимодействие с JavaScript
Итак, какой функционал сейчас включён в скомпилированный WebAssembly? В `lib.rs` есть две функции: внешняя (extern) функция `alert()` и функция `hello()`. Обе помечены атрибутом `#[wasm_bindgen]`, чтобы показать, что их нужно связать с WebAssembly. Внешние функции связываются с существующими JavaScript-методами; в нашем случае это [функция `alert()` объекта window](https://developer.mozilla.org/ru/docs/Web/API/Window/alert), которая показывает всплывающее диалоговое окно.

Добавим WebAssembly на наш сайт. Вставьте следующий JavaScript в `<body>` файла `index.html`. Он загрузит бинарник WebAssembly и будет вызывать функцию `hello()` при нажатии кнопки отправки:
```html
<script type="module">
    import init, { hello } from "./pkg/lettuce_crop_wasm.js";
    init().then(() => {
        const submit_button = document.querySelector('input[type="submit"]');
        submit_button.onclick = () => {
            hello("WebAssembly");
        }
    });
</script>
```

Чтобы попробовать сайт, можно использовать любой HTTP-сервер, который умеет отдавать локальные файлы. Например, можно использовать `axum`, как в упражнении 5.1.1, а если у вас установлен `npm`, подойдёт, например, `npx http-server`.

### 5.3.1.C Обрезка изображений
Добавим в нашу библиотеку на Rust функцию `crop_image(bytes: Vec<u8>, max_size: u32) -> Vec<u8>`, которая будет обрезать изображения. Можно использовать ту же логику, что и в упражнении 5.1.1 (части D и E), чтобы создать `DynamicImage` из входных байтов, обрезать его и экспортировать в WebP. Пометьте функцию атрибутом `#[wasm_bindgen]` и пересоберите библиотеку, чтобы сгенерировать для неё привязки WebAssembly.

Если посмотреть на сгенерированные JavaScript-привязки, вы увидите, что `Vec<u8>` в функции `crop_image` превратился в `Uint8Array`. Нам понадобится написать JavaScript, который читает выбранное пользователем изображение и передаёт его в `crop_image` как `Uint8Array`.

Сначала получим два других входных элемента:
```js
const max_size = document.querySelector('input[name="max_size"]');
const image = document.querySelector('input[name="image"]');
```
Затем в `onclick` кнопки отправки можно получить выбранный файл через `image.files[0]`. Чтобы получить содержимое файла, используем [`FileReader`](https://developer.mozilla.org/ru/docs/Web/API/FileReader):
```js
const file = image.files[0];
const reader = new FileReader();
reader.onload = (evt) => {
    const bytes = new Uint8Array(evt.target.result);
    const cropped_bytes = crop_image(bytes, max_size.value); // вызываем нашу функцию
    // TODO: что-то сделать с cropped_bytes
};
reader.readAsArrayBuffer(file);
```
Наконец, чтобы показать пользователю результат, создадим [`Blob`](https://developer.mozilla.org/ru/docs/Web/API/Blob) из `Uint8Array` и превратим этот `Blob` в URL, на который перенаправим пользователя:
```js
window.location.href = URL.createObjectURL(new Blob([cropped_bytes]));
```
Если выбрать некорректный файл, в консоли браузера появится ошибка. Можно добавить более аккуратную обработку ошибок с помощью [try-catch](https://developer.mozilla.org/ru/docs/Web/JavaScript/Reference/Statements/try...catch), а также проверять, что `image.files[0]` существует, перед чтением файла. Было бы неплохо проверить и то, что значение `max_size` разумно.

### 5.3.1.D Использование крейта web-sys (бонус)
Вместо того чтобы использовать JavaScript для работы с HTML-документом или вручную связывать внешние JavaScript-функции через `#[wasm_bindgen]`, как мы делали с `alert()`, можно использовать крейт [`web-sys`](https://crates.io/crates/web-sys). Он предоставляет привязки к JavaScript Web API, доступным в браузере. Однако большинство этих API нужно включать вручную, по отдельным фичам.

Добавьте крейт `web-sys` в проект, включив все нужные фичи:
```
cargo add web-sys --features "Window,Document,HtmlElement,HtmlImageElement,Blob,Url"
```

Теперь пусть функция `crop_image` не возвращает массив байтов, а добавляет изображение в HTML-документ:
- Сначала получите элемент `body` HTML-документа:
    ```rust
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    ```
- Затем создайте HTML-элемент изображения:
    ```rust
    let img = document.create_element("img").unwrap();
    let img: web_sys::HtmlImageElement = img.dyn_into().unwrap();
    ```
- Чтобы задать источник изображения, снова понадобится `Blob`, с помощью которого получим временный data URL. Для этого сначала создаём JavaScript-массив:
    ```rust
    let bytes = web_sys::js_sys::Array::new();
    bytes.push(&web_sys::js_sys::Uint8Array::from(&buffer[..]));
    ```
- Далее создаём Blob и URL:
    ```rust
    let blob = web_sys::Blob::new_with_u8_array_sequence(&bytes).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
    ```
- И наконец задаём источник изображения и добавляем его в body документа:
    ```rust
    img.set_src(&url);
    body.append_child(&img).unwrap();
    ```
- Не забудьте соответствующим образом обновить и JavaScript-код в HTML-документе.
