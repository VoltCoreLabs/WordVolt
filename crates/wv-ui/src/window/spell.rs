use std::sync::Arc;

use gpui_kit::*;
use wv_spell::Speller;

use super::state::MainWindow;
use crate::spell::SpellGlobal;

impl MainWindow {
    /// Loads the dictionaries in the background when spell checking is on (and not loaded yet).
    pub fn start_spell(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let global = cx.global_mut::<SpellGlobal>();
        if !global.enabled || global.speller.is_some() || global.loading {
            return;
        }
        global.loading = true;
        cx.spawn_in(window, async move |this, cx| {
            let speller = cx
                .background_executor()
                .spawn(async { Speller::load() })
                .await;
            for word in crate::settings::load_user_words() {
                speller.add_word(&word);
            }
            this.update_in(cx, |_, window, cx| {
                let global = cx.global_mut::<SpellGlobal>();
                global.speller = Some(Arc::new(speller));
                global.loading = false;
                window.refresh();
            })
            .ok();
        })
        .detach();
    }

    /// Switches spell checking on or off (remembered).
    pub fn toggle_spell(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let global = cx.global_mut::<SpellGlobal>();
        global.enabled = !global.enabled;
        let on = global.enabled;
        crate::settings::save_spellcheck(on);
        if on {
            self.start_spell(window, cx);
        }
        window.refresh();
        cx.notify();
    }

    /// Adds `word` to the personal dictionary (`keep`: for good; otherwise until the app closes).
    pub fn add_to_dictionary(&mut self, word: &str, keep: bool, cx: &mut Context<Self>) {
        if let Some(speller) = crate::spell::active(cx) {
            speller.add_word(word);
        }
        if keep {
            crate::settings::append_user_word(word);
        }
        cx.notify();
    }
}
