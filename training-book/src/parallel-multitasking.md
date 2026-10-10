# Модуль 4.2 — Параллельная многозадачность

<a href="https://training.tweede.golf/slides/4_2-parallel-multitasking/" target="_blank">Слайды</a>

## Упражнение 4.2.1: TF-IDF

Следуйте инструкциям в комментариях файла `exercises/4-multitasking/2-parallel-multitasking/1-tf-idf/src/main.rs`!

## Упражнение 4.2.2: Mutex

Простой mutex при ожидании захвата блокировки крутится в цикле (spin-loop). Это крайне неэффективно. К счастью, операционная система умеет ждать, пока блокировка освободится, и в это время просто усыпляет поток.

Эта возможность доступна в крейте [atomic_wait](https://docs.rs/atomic-wait/latest/atomic_wait/index.html). Раздел [о реализации mutex](https://marabos.nl/atomics/building-locks.html#mutex) из книги *Rust Atomics and Locks* объясняет, как им пользоваться.

- Замените `AtomicBool` на `AtomicU32`.
- Реализуйте `lock`. Будьте внимательны к ложным пробуждениям (spurious wakes): после того как `wait` вернул управление, условие всё равно нужно проверить.
- Реализуйте разблокировку (`Drop for MutexGuard<T>`) с помощью `wake_one`.

В упомянутой главе mutex оптимизируется дальше. Формально это выходит за рамки курса, но мы не будем вас останавливать, если вы попробуете (и всё равно постараемся помочь, если застрянете)!
