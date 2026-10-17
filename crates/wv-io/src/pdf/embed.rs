use miniz_oxide::deflate::compress_to_vec_zlib;
use pdf_writer::types::{CidFontType, FontFlags, SystemInfo, UnicodeCmap};
use pdf_writer::{Filter, Finish, Name, Pdf, Rect, Ref, Str};

use super::fonts::Face;

/// Hands out object ids.
pub struct Ids(i32);

impl Ids {
    pub fn new() -> Self {
        Self(1)
    }

    pub fn next(&mut self) -> Ref {
        let id = Ref::new(self.0);
        self.0 += 1;
        id
    }
}

/// Writes the face as an embedded Type0 (Identity-H) font and returns its object id. Text is shown
/// as 2-byte glyph ids; a ToUnicode map keeps it searchable and copyable.
pub fn write_font(pdf: &mut Pdf, ids: &mut Ids, face: &Face, number: usize) -> Ref {
    let (type0, cid, descriptor, file, to_unicode) =
        (ids.next(), ids.next(), ids.next(), ids.next(), ids.next());
    let base = format!("WordVoltFont{number}");
    let base = Name(base.as_bytes());
    let parsed = ttf_parser::Face::parse(&face.data, 0).ok();
    let upem = parsed
        .as_ref()
        .map_or(1000.0, |f| f64::from(f.units_per_em()));
    let scale = |v: i16| (f64::from(v) * 1000.0 / upem) as f32;
    let (ascent, descent, bbox, cap, angle) = parsed.as_ref().map_or(
        (
            800.0,
            -200.0,
            Rect::new(0.0, -200.0, 1000.0, 800.0),
            700.0,
            0.0,
        ),
        |f| {
            let b = f.global_bounding_box();
            (
                scale(f.ascender()),
                scale(f.descender()),
                Rect::new(
                    scale(b.x_min),
                    scale(b.y_min),
                    scale(b.x_max),
                    scale(b.y_max),
                ),
                f.capital_height().map_or(scale(f.ascender()), scale),
                f.italic_angle(),
            )
        },
    );

    pdf.type0_font(type0)
        .base_font(base)
        .encoding_predefined(Name(b"Identity-H"))
        .descendant_font(cid)
        .to_unicode(to_unicode);

    let mut cid_font = pdf.cid_font(cid);
    cid_font
        .subtype(CidFontType::Type2)
        .base_font(base)
        .system_info(SystemInfo {
            registry: Str(b"Adobe"),
            ordering: Str(b"Identity"),
            supplement: 0,
        })
        .font_descriptor(descriptor)
        .default_width(0.0)
        .cid_to_gid_map_predefined(Name(b"Identity"));
    let mut widths = cid_font.widths();
    for (gid, (_, advance)) in &face.used {
        widths.consecutive(*gid, [*advance]);
    }
    widths.finish();
    cid_font.finish();

    pdf.font_descriptor(descriptor)
        .name(base)
        .flags(FontFlags::NON_SYMBOLIC)
        .bbox(bbox)
        .italic_angle(angle)
        .ascent(ascent)
        .descent(descent)
        .cap_height(cap)
        .stem_v(80.0)
        .font_file2(file);

    let packed = compress_to_vec_zlib(&face.data, 6);
    let mut stream = pdf.stream(file, &packed);
    stream.filter(Filter::FlateDecode);
    stream.pair(Name(b"Length1"), face.data.len() as i32);
    stream.finish();

    let mut cmap = UnicodeCmap::new(
        Name(b"Adobe-Identity-UCS"),
        SystemInfo {
            registry: Str(b"Adobe"),
            ordering: Str(b"UCS"),
            supplement: 0,
        },
    );
    for (gid, (c, _)) in &face.used {
        cmap.pair(*gid, *c);
    }
    pdf.cmap(to_unicode, &cmap.finish());
    type0
}
