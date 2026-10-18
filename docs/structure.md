# Структура файлов WordVolt

Цель: любой файл читается целиком за один заход. Новая фича = новый файл/модуль, а не рост старого.

## Лимиты (жёсткие)
- Файл `.rs` ≤ **300 строк** (без тестов). Больше 400 — обязательно делить.
- Функция ≤ 50 строк. `mod.rs` — только `pub mod` / `pub use`, без логики.
- Тесты: небольшие — `#[cfg(test)] mod tests` внизу файла; большие — `tests/` или `foo/tests.rs`.
- Один файл = одна ответственность (один тип + его impl, или один набор родственных функций).
- Каждый крейт имеет `README.md` на 5–10 строк: что делает, публичный API, что НЕ делает. ИИ читает его вместо кода.

## Дерево

```
WordVolt/
├─ CLAUDE.md                 правила для ИИ
├─ plan.md                   задачи и этапы
├─ Cargo.toml                [workspace], общие версии зависимостей
├─ rust-toolchain.toml
├─ .github/workflows/        ci.yml (fmt, clippy, test, матрица 3 ОС)
├─ docs/
│  ├─ structure.md           этот файл
│  ├─ decisions.md           ADR
│  └─ log.md                 журнал сессий
├─ .github/workflows/        CI (ci.yml)
├─ assets/
│  ├─ fonts/                 встроенные шрифты (fallback)
│  ├─ icons/                 SVG иконки ribbon
│  ├─ dictionaries/{ru,en}/  словари Hunspell (index.aff/.dic + LICENSE) для орфографии
│  └─ i18n/{en,ru}/*.ftl     строки интерфейса (fluent)
├─ tests/fixtures/           примеры .docx/.md/.rtf для тестов io
├─ crates/
│  ├─ wv-core/               модель документа (без GPUI)
│  ├─ wv-layout/             вёрстка
│  ├─ wv-io/                 форматы файлов
│  ├─ wv-spell/              орфография (spellbook, Hunspell)
│  ├─ wv-i18n/               локализация
│  └─ wv-ui/                 GPUI-интерфейс
└─ app/                      бинарник `wordvolt` (main.rs ≤ 50 строк); icon.ico + app.rc + build.rs — иконка exe (ресурс 1, из него GPUI берёт иконку окна/панели задач); логотип — assets/brand/logo.png
```

### wv-core — модель и операции
```
crates/wv-core/src/
├─ lib.rs
├─ document/
│  ├─ mod.rs
│  ├─ document.rs       Document, корень, метаданные
│  ├─ block.rs          enum Block { Paragraph, Table, Image, ... }
│  ├─ paragraph.rs      Paragraph, список Run
│  ├─ run.rs            Run (текст + CharStyle)
│  ├─ table.rs          (v0.2)
│  ├─ image.rs          (v0.2)
│  └─ section.rs        поля, ориентация, колонтитулы
├─ style/
│  ├─ mod.rs
│  ├─ char_style.rs     шрифт, размер, B/I/U, цвет
│  ├─ para_style.rs     выравнивание, отступы, интервалы
│  ├─ named_styles.rs   «Заголовок 1» и т.п.
│  └─ list.rs           маркированные/нумерованные списки
├─ position/
│  ├─ mod.rs
│  ├─ position.rs       (block_id, offset)
│  ├─ selection.rs      anchor + head
│  └─ navigation.rs     по графемам/словам/строкам
├─ command/
│  ├─ mod.rs            trait Command + Executor
│  ├─ insert_text.rs
│  ├─ delete_range.rs
│  ├─ split_paragraph.rs
│  ├─ merge_paragraphs.rs
│  ├─ set_char_style.rs
│  └─ set_para_style.rs    ← один файл на команду
├─ history/
│  ├─ mod.rs
│  ├─ undo_stack.rs
│  └─ coalesce.rs       склейка набора текста
└─ error.rs
```

### wv-layout — вёрстка
```
crates/wv-layout/src/
├─ lib.rs
├─ fonts/       font_db.rs, fallback.rs
├─ shaping/     shaper.rs, cache.rs
├─ line/        line_breaker.rs, line_box.rs
├─ page/        page_setup.rs, paginator.rs, page_box.rs
├─ hit_test/    point_to_pos.rs, pos_to_rect.rs
└─ incremental.rs   инвалидация и пересчёт
```

### wv-io — форматы (каждый формат — своя папка, одинаковый вид)
```
crates/wv-io/src/
├─ lib.rs            trait Importer / Exporter, реестр по расширению
├─ wvd/       mod.rs, reader.rs, writer.rs, schema.rs
├─ docx/
│  ├─ mod.rs
│  ├─ reader/  package.rs (zip), document_xml.rs, styles_xml.rs, numbering_xml.rs, relationships.rs
│  ├─ writer/  package.rs, document_xml.rs, styles_xml.rs, numbering_xml.rs
│  └─ mapping.rs     docx ↔ wv-core
├─ markdown/  reader.rs, writer.rs
├─ txt/       reader.rs, writer.rs
├─ rtf/       reader.rs, writer.rs
├─ html/      reader.rs, writer.rs
├─ odt/       reader.rs, writer.rs
└─ pdf/       writer.rs, font_embed.rs
```

### wv-ui — интерфейс (GPUI изолирован здесь)
```
crates/wv-ui/src/
├─ lib.rs
├─ app.rs                 инициализация, глобальное состояние
├─ editor/clip.rs        копирование/вставка с форматированием (ClipFragment)
├─ spell.rs               глобальное состояние орфографии (SpellGlobal); window/spell.rs — загрузка, переключатель, «в словарь»
├─ window/                main_window.rs, titlebar.rs, status_bar.rs
├─ editor/
│  ├─ mod.rs
│  ├─ editor_view.rs      корневой вид редактора
│  ├─ page_canvas.rs      отрисовка страниц (виртуализация)
│  ├─ caret.rs
│  ├─ selection_paint.rs
│  ├─ input_handler.rs    клавиатура + IME
│  ├─ mouse.rs
│  └─ scroll_zoom.rs
├─ ribbon/
│  ├─ mod.rs
│  ├─ ribbon_view.rs
│  ├─ tab_home.rs   tab_insert.rs   tab_layout.rs   tab_view.rs   ← по вкладке на файл
│  └─ widgets/  font_picker.rs, size_picker.rs, color_picker.rs, toggle_button.rs
├─ panels/        styles_panel.rs, find_replace.rs, outline.rs
├─ dialogs/       open_save.rs, page_setup.rs, about.rs, settings.rs
├─ actions.rs     список GPUI actions (имена)
├─ keymap.rs      горячие клавиши
├─ clipboard.rs
├─ theme/         colors.rs, spacing.rs, radius.rs, typography.rs, light.rs (dark.rs — позже)
└─ ui_kit/        block.rs, button.rs, icon_button.rs, toggle_button.rs, dropdown.rs, color_swatch.rs, separator.rs, tooltip.rs, block_group.rs
```

## Правила масштабирования
1. **Новая команда редактирования** → новый файл в `wv-core/src/command/`, строка в `mod.rs`. Больше ничего не трогать.
2. **Новый формат** → новая папка в `wv-io/src/` по шаблону `reader.rs` + `writer.rs`, регистрация в `lib.rs`.
3. **Новая вкладка ribbon** → новый `tab_*.rs`.
4. **Новый тип блока** → `document/<name>.rs` + вариант в `Block` + отдельный файл раскладки в `wv-layout` + отрисовка в `wv-ui`.
5. Зависимости строго вниз: `app → ui → layout → core`; `io → core`. Нарушение = ошибка ревью.
6. Общие версии зависимостей — только в `[workspace.dependencies]` корневого `Cargo.toml`.
7. Публичный API крейта — через `lib.rs` (`pub use`), внутренности `pub(crate)`.
8. Строки интерфейса — только через `wv-i18n` ключи, не в коде.

## Как ИИ должен читать код
1. `CLAUDE.md` → `plan.md` (текущий этап) → `crates/<name>/README.md`.
2. Нужный файл находить через Grep по имени типа/функции, читать только его.
3. Файл > 300 строк — сначала разбить (отдельная задача/коммит), потом менять.
