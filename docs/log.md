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
