# Модуль 7.1 — Rust для программистов на Python

<a href="https://training.tweede.golf/slides/7_1-rust-from-python/" target="_blank">Слайды</a>

## Упражнение 7.1.1: Проверка окружения

Установите [uv](https://docs.astral.sh/uv/#installation) и [Rust](https://www.rust-lang.org/tools/install).

Проверьте, что всё работает:
```bash
cd rust-dna
uv run maturin develop --uv
cd ..
uv run pytest
```

Вывод должен выглядеть примерно так:
```
❯ uv run pytest
Installed 5 packages in 9ms
============================= test session starts ==============================
platform linux -- Python 3.12.3, pytest-8.3.5, pluggy-1.5.0
rootdir: /home/tamme/dev/rust-training/exercises/7-rust-for-data-science/1-rust-from-python
configfile: pyproject.toml
collected 10 items

test_decoding.py ....                                                    [ 40%]
test_kmers.py ....                                                       [ 80%]
test_setup.py .                                                          [ 90%]
test_validation.py .                                                     [100%]

============================== 10 passed in 0.06s ==============================
```

## Упражнение 7.1.2: Перевод Python-кода на Rust

Откройте `exercises/7-rust-for-data-science/1-rust-from-python/` в редакторе и следуйте шагам в `main.py`, чтобы перевести логику на Rust в `rust-dna/lib.rs`.
