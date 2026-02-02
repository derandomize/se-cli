# Overview: модули и расширяемость

## Основные сущности (объектная модель)

- **`PipelineAst`**: список стадий пайплайна, каждая стадия — `SimpleCommandAst`.
- **`SimpleCommandAst`**:
  - `assignments: Vec<AssignmentAst>` — ведущие `NAME=value` перед командой;
  - `argv: Vec<WordAst>` — имя команды и аргументы (ещё с quote‑структурой до расширений).
- **`WordAst`**: одно “слово” как конкатенация частей (для случаев вроде `$x$y`):
  - `parts: Vec<WordPartAst>`
- **`WordPartAst`**:
  - `Literal { text, quoting }`
  - `Var { name, quoting }`
  - где `quoting ∈ {Unquoted, SingleQuoted, DoubleQuoted}`.

> Принцип: **пайпы и пробелы — структурные только вне кавычек**, а внутри кавычек входят в литерал.

## Компоненты и интерфейсы (Rust модули)

Предлагаемая модульная структура (без реализации деталей на этом этапе):

- `repl/` — чтение строк, цикл, печать ошибок
- `lex/` — `Lexer` → `Vec<Token>`
- `parse/` — `Parser` → `PipelineAst`
- `expand/` — `Expander` применяет `$NAME` и “quote removal”, возвращает `ExpandedPipeline`
- `env/` — `EnvStore` (shell vars) + “temporary overlay” для `NAME=value cmd`
- `exec/` — `Executor` (пайпы, запуск стадий)
- `builtins/` — реализации встроенных команд + `BuiltinRegistry`
- `external/` — адаптер к `std::process::Command`

## Как “легко добавлять новые команды”

**Встроенная команда** добавляется без изменения парсера/лексера:
1. Создать тип `FooBuiltin`, реализующий интерфейс:
   - `name() -> &'static str`
   - `run(args, io, ctx) -> ExitStatus`
2. Зарегистрировать в `BuiltinRegistry` (например, `HashMap<String, Arc<dyn Builtin>>`).

Важное: `run` работает с **потоками**, а не с “входными аргументами как текстом”. Аргументы (`argv`) и входной поток (`stdin`) — разные вещи.

## Коды возврата

- Каждый builtin возвращает `ExitStatus(u8)` (0 — успех, ненулевые — ошибка).
- Для external код берём из `ExitStatus` процесса (или маппим сигналы/ошибки запуска в фиксированные коды).
- Итоговый статус строки — статус **последней стадии пайплайна** (как в большинстве shell’ов).
