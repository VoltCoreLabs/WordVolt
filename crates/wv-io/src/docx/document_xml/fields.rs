use wv_core::{CharStyle, Document};

use super::State;

/// A field being read: what its result (the text between `separate` and `end`) stands for.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Field {
    /// `REF name`: the text of a bookmark; read as a link to it.
    Ref(String),
    /// `PAGEREF name`: the page number of a bookmark.
    PageRef(String),
}

/// `page_ref` values from this on are indexes into `State::ref_names`, until the bookmarks (that
/// may come later in the file) are known.
const PENDING: u32 = 0x8000_0000;

/// The field an instruction like ` PAGEREF _Ref123 \h ` is.
fn parse(instr: &str) -> Option<Field> {
    let mut words = instr.split_whitespace();
    let kind = words.next()?.to_uppercase();
    let name = words.next()?.trim_matches('"').to_string();
    match kind.as_str() {
        "REF" => Some(Field::Ref(name)),
        "PAGEREF" => Some(Field::PageRef(name)),
        _ => None,
    }
}

impl State {
    /// `w:fldSimple`: the instruction is an attribute; the result is its content.
    pub(super) fn simple_field(&mut self, instr: &str) {
        self.field = parse(instr);
    }

    /// `w:fldChar`: a complex field is `begin`, the instruction in `w:instrText`, `separate`, the
    /// result, `end`.
    pub(super) fn field_char(&mut self, kind: &str) {
        match kind {
            "begin" => {
                self.instr.clear();
                self.collecting = true;
                self.field = None;
            }
            "separate" => {
                self.collecting = false;
                self.field = parse(&self.instr);
            }
            "end" => {
                self.collecting = false;
                self.field = None;
            }
            _ => {}
        }
    }

    /// What the field being read gives to the style of its result text.
    pub(super) fn apply_field(&mut self, style: &mut CharStyle) {
        let Some(field) = self.field.clone() else {
            return;
        };
        match field {
            Field::Ref(name) => style.link = style.link.take().or(Some(format!("#{name}"))),
            Field::PageRef(name) => {
                style.link = style.link.take().or(Some(format!("#{name}")));
                let index = match self.ref_names.iter().position(|n| *n == name) {
                    Some(i) => i,
                    None => {
                        self.ref_names.push(name);
                        self.ref_names.len() - 1
                    }
                };
                style.page_ref = Some(PENDING + index as u32);
            }
        }
    }
}

/// Points the page-number fields at the ids of the bookmarks they name (`names` as collected by
/// `apply_field`); a field whose bookmark does not exist becomes plain text.
pub(super) fn resolve(names: &[String], doc: &mut Document) {
    let bookmarks = doc.bookmarks.clone();
    for p in &mut doc.paragraphs {
        for run in &mut p.runs {
            let Some(pending) = run.style.page_ref.filter(|id| *id >= PENDING) else {
                continue;
            };
            run.style.page_ref = names
                .get((pending - PENDING) as usize)
                .and_then(|name| bookmarks.iter().find(|b| b.name == *name))
                .map(|b| b.id);
        }
        p.normalize();
    }
}

#[cfg(test)]
mod tests {
    use super::{Field, parse};

    #[test]
    fn instructions_are_told_apart() {
        assert_eq!(
            parse(" PAGEREF _Ref12 \\h "),
            Some(Field::PageRef("_Ref12".into()))
        );
        assert_eq!(parse(r"REF mark \h"), Some(Field::Ref("mark".into())));
        assert_eq!(parse(" TOC \\o \"1-3\""), None);
        assert_eq!(parse(""), None);
    }
}
