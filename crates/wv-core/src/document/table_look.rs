/// No column gets narrower than this share of the text width.
pub const MIN_COLUMN: f32 = 0.04;

/// How a table is sized: column widths and the left indent (fractions of the text width; together
/// they may be less than 1, then the table is narrower than the text) and the least height of
/// every row (px; 0 = as tall as the text needs).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableLook {
    pub widths: Vec<f32>,
    pub indent: f32,
    pub rows: Vec<f32>,
}

impl TableLook {
    /// The look of a table of `rows` x `cols` cells: what is stored for it when that fits,
    /// otherwise equal columns filling the text width and rows as tall as their text.
    pub fn resolved(stored: Option<&TableLook>, rows: usize, cols: usize) -> Self {
        let widths = stored.filter(|s| s.widths.len() == cols).map_or_else(
            || vec![1.0 / cols.max(1) as f32; cols],
            |s| s.widths.clone(),
        );
        let rows_min = stored
            .filter(|s| s.rows.len() == rows)
            .map_or_else(|| vec![0.0; rows], |s| s.rows.clone());
        Self {
            widths,
            indent: stored.map_or(0.0, |s| s.indent),
            rows: rows_min,
        }
        .normalized()
    }

    /// Widths of at least [`MIN_COLUMN`], an indent of 0 or more, and everything inside the text
    /// width (the columns shrink together when it would not fit).
    pub fn normalized(mut self) -> Self {
        let clean = |w: f32| {
            if w.is_finite() {
                w.max(MIN_COLUMN)
            } else {
                MIN_COLUMN
            }
        };
        self.widths.iter_mut().for_each(|w| *w = clean(*w));
        self.indent = if self.indent.is_finite() {
            self.indent.clamp(0.0, 0.9)
        } else {
            0.0
        };
        self.rows
            .iter_mut()
            .for_each(|h| *h = if h.is_finite() { h.max(0.0) } else { 0.0 });
        let sum: f32 = self.widths.iter().sum();
        let room = (1.0 - self.indent).max(MIN_COLUMN);
        if sum > room {
            self.widths.iter_mut().for_each(|w| *w *= room / sum);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_fill_the_text_width_and_a_stored_look_must_fit() {
        let d = TableLook::resolved(None, 2, 3);
        assert_eq!(d.widths, vec![1.0 / 3.0; 3]);
        assert_eq!((d.indent, d.rows.len()), (0.0, 2));
        let stored = TableLook {
            widths: vec![0.2, 0.3],
            indent: 0.1,
            rows: vec![40.0, 0.0],
        };
        // Wrong number of columns: equal ones; the right number of rows is kept.
        let r = TableLook::resolved(Some(&stored), 2, 3);
        assert_eq!(r.widths.len(), 3);
        assert_eq!(r.rows, vec![40.0, 0.0]);
        assert_eq!(
            TableLook::resolved(Some(&stored), 2, 2).widths,
            vec![0.2, 0.3]
        );
    }

    #[test]
    fn a_table_never_leaves_the_text_width() {
        let look = TableLook {
            widths: vec![0.6, 0.6],
            indent: 0.2,
            rows: vec![],
        }
        .normalized();
        let total: f32 = look.widths.iter().sum::<f32>() + look.indent;
        assert!((total - 1.0).abs() < 1e-5);
        let tiny = TableLook {
            widths: vec![0.0, f32::NAN],
            ..Default::default()
        }
        .normalized();
        assert!(tiny.widths.iter().all(|w| *w >= MIN_COLUMN));
    }
}
