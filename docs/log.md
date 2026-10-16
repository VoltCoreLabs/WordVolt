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
