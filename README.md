# se-cli
[![CI](https://github.com/derandomize/se-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/derandomize/se-cli/actions/workflows/ci.yml)

Простой интерпретатор командной строки на Rust. Проект в рамках курса по разработке ПО.

> Статус: реализованы REPL и выполнение команд (`cat`, `echo`, `wc`, `pwd`, `exit`),
> а также запуск внешних программ и поддержка кавычек/окружения (присваивания и передача env во внешний процесс).  
> Также поддерживаются подстановки `$NAME` и пайплайны `|`.
> Добавлена builtin-команда `grep` (regex, `-w`, `-i`, `-A`).

## Документация

- Архитектура: `docs/ARCHITECTURE.md`

## Что планируется поддержать (по заданию)

- **Builtins**: `cat`, `echo`, `grep`, `wc`, `pwd`, `exit`
- **Кавычки**: `'...'` (full quoting) и `"..."` (weak quoting)
- **Окружение**: присваивания `NAME=value` и подстановка `$NAME`
- **External commands**: запуск через `PATH`, если команда не builtin
- **Пайплайны**: оператор `|`

## Пример использования

Примеры команд в REPL:

```text
echo "Hello, world!"
FILE=README.md
cat $FILE | wc
grep "Минимальный" README.md
grep -i "минимальный" README.md
grep -w "Минимал" README.md
grep -A 1 "II" README.md
grep "MIT" README.md LICENSE
echo hi | grep hi
cat README.md
wc README.md
pwd
FOO=bar cmd /C echo %FOO%        (Windows)
FOO=bar sh -c 'echo $FOO'        (Linux/macOS)
exit
```

## Выбор библиотеки для разбора аргументов (grep)

Для builtin-команды `grep` нужен нетривиальный разбор ключей (`-w`, `-i`, `-A N`).
Рассматривались варианты:

- `clap` — полнофункциональный парсер CLI (валидация, понятные ошибки, поддержка флагов/значений).
- `pico-args` / `argh` / `gumdrop` — более легкие варианты, но с меньшей гибкостью/удобством валидации.

Выбрали `clap`, потому что он дает предсказуемый разбор и удобные сообщения об ошибках,
а реализация остается компактной за счет derive-макроса.

## Сборка и запуск

- **Требования**: Rust stable (см. `Cargo.toml`, edition 2024).
- **Сборка**: `cargo build`
- **Запуск**: `cargo run`

Интерпретатор читает строки из stdin до `exit` или EOF.

## Разработка

- **Проверка форматирования**: `cargo fmt --all -- --check`
- **Линтинг**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Тесты**: `cargo test --all`

## Как помочь проекту (contributing)

Проект учебный, но вклад приветствуется:

- **Идеи/баги**: заведите issue в репозитории.
- **Pull request**: кратко опишите изменения и как их проверить.
- **Перед PR прогоните локально**:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all`

## Лицензия

MIT, см. файл `LICENSE`.

## Авторы

- Головачев Сергей
- Деружинский Дмитрий
- Токарев Алексей
