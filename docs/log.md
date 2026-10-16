# Журнал сессий

## 2026-10-17
- Созданы: `CLAUDE.md`, `plan.md`, `docs/{structure,design,decisions}.md`.
- Приняты решения D-001…D-009 (см. decisions.md).
- Кода нет; в `src/main.rs` старый набросок (не собирается на новом API).
- Дальше: Этап 0 — ветка `feat/workspace-setup`: workspace, версии gpui-kit, dev-loop, прототип анимаций.
- (позже в тот же день) Ветка `feat/workspace-setup`: workspace, `wv-ui` (theme, ui_kit, window), `cargo dev`. Окно запускается. Иконки — Phosphor (D-010).
- Дальше: `feat/ui-animation-prototype` (анимации GPUI, раскрывающаяся панель), hot-reload токенов темы, быстрый линкер, CI.
- Ветка `feat/ui-foundation`: шрифт (SF Pro Text → Inter → Segoe UI, D-012), иконки Phosphor, ui_kit (icon, icon_button, tab, block_group), вкладки ribbon, анимации (вкладки, ribbon, боковая панель), hot-reload темы из `assets/theme/light.toml`.
- Грабля: GPUI `svg()` рисуется только если `text_color` задан на самом svg, наследование от родителя не работает.
- Ветка `feat/core-editor`: `wv-core` — модель (Document/Paragraph/Run), Selection, Editor (insert/delete/enter/backspace, стили B/I/U, выравнивание, движение каретки по графемам), undo/redo через Edit-splice абзацев, 16 тестов. Не сделано: склейка undo при наборе, движение по словам, up/down (нужна вёрстка).
- В plan.md добавлена задача пользователя: кастомный titlebar (свернуть/развернуть/закрыть, «WordVolt — Новый документ»).
- Ветка `feat/titlebar`: кастомный titlebar (окно без системной рамки), кнопки свернуть/развернуть/закрыть (`window_control_area` для Windows, `start_window_move`/double-click для Linux, macOS — родные traffic lights), заголовок «WordVolt — Новый документ» (wv-i18n, en/ru), точка при несохранённых изменениях. Не проверено вживую: Linux/macOS.
- Ветка `feat/editor-view`: живой редактор в `wv-ui/src/editor/` (view, paragraph, keys, mouse, vertical): ввод, Enter/Backspace/Delete, стрелки, Home/End, выделение мышью, double/triple click, Ctrl+A/Z/Y/B/I/U/C/X/V, мигающая каретка. Ribbon (B/I/U, align, undo/redo) подключён к `wv-core`. В core: `revision`, `current_style`, `select_word/paragraph` (18 тестов). Статус-бар считает слова.
- Известные ограничения: один лист без пагинации, размер шрифта не отображается, justify как left, нет IME и автоскролла к каретке, dirty «липкий» до реализации Save.
- `feat/editor-view`, раунд 2 (по замечаниям пользователя): починена каретка при выравнивании (GPUI `position_for_index`/`index_for_position` игнорируют align — добавлен `editor/align.rs`), плавный скролл (`editor/scroll.rs`, easing + полоса прокрутки + автоскролл к каретке), плавная каретка (`editor/caret.rs`: fade-in + мягкое мигание), весь интерфейс на русском через `wv-i18n` (en/ru, плюрализация), английский README.md.
- Исправлено: выделение не было видно при выравнивании не по левому краю (GPUI красит фон run без учёта align) — теперь выделение рисуем сами (`align::selection_rects`), run-фон не используется.
- fix/mouse-selection: перетаскивание выделения мышью обрабатывается один раз на весь viewport (`on_drag_move`), абзац под курсором определяется по координатам; раньше слушатели движения висели на каждом абзаце.
- `feat/file-io` (стоит на `fix/mouse-selection`): `wv-io` — .wvd (JSON+версия), .txt (UTF-8/UTF-16/BOM), .md (подмножество), атомарная запись, 11 тестов; UI — `window/file_ops.rs` (Новый/Открыть/Сохранить/Сохранить как, подтверждение при несохранённых изменениях и закрытии окна), `actions.rs` (Ctrl+N/O/S/Shift+S), заголовок с именем файла, `Editor::is_modified/mark_saved`. Плавные ховеры: `ui_kit/hover.rs` (слой с fade 140 мс на кнопках, вкладках, кнопках окна).
- Не проверено вживую: диалоги ОС, подтверждение при закрытии окна. Ограничения: язык переключить нельзя, имя по умолчанию в «Сохранить как» — «Новый документ.wvd».
- `feat/docx`: `.docx` чтение/запись (`wv-io/src/docx/`: package, document_xml, writer; zip 2 + quick-xml 0.37), 17 тестов wv-io; диалоги Открыть/Сохранить как на `rfd` с фильтрами форматов (в «Сохранить как» формат выбирается фильтром, текущий формат предвыбран). Урок: удаление базовой ветки при мерже стекового PR закрывает зависимый PR — сначала перенацелить его на main.
- Не проверено вживую: открытие .docx из настоящего Word/LibreOffice (только наши тесты и пример в стиле Word); диалоги rfd.
