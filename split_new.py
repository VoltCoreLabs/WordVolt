import re

P = 'crates/wv-ui/src/window/state.rs'
NL = chr(10)
s = open(P, encoding='utf-8', newline='').read().replace(chr(13) + NL, NL)
fields = dict(re.findall(r'^    pub (\w+): Entity<([^>]+)>,', s, flags=re.M))
fields.pop('editor', None)

start = s.index('    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {')
body_start = s.index(NL, start) + 1
body_end = s.index('        Self {' + NL + '            editor: editor.clone(),')
lines = s[body_start:body_end].split(NL)

idx = [i for i, l in enumerate(lines) if re.match(r'^        let \w+', l)]
pre = lines[:idx[0]]
real = []
for k, a in enumerate(idx):
    b = idx[k + 1] if k + 1 < len(idx) else len(lines)
    real.append(lines[a:b])

locals_defined = []
out_funcs = []
new_lines = list(pre)
for blk in real:
    text = NL.join(blk)
    name = re.match(r'^        let (\w+)', blk[0]).group(1)
    first = blk[0] + (' ' + blk[1].strip() if len(blk) > 1 else '')
    is_widget = name in fields and re.match(r'^        let \w+ = cx\s*\.new\(', first) is not None
    body_text = text.replace('let ' + name, '', 1)
    deps = []
    for l in locals_defined:
        if l in (name, 'lang'):
            continue
        # a use of the local itself, not a field access (`this.editor`) or a struct field name
        if re.search(r'(?<![.\w])' + l + r'\b(?!:)', body_text):
            deps.append(l)
    if is_widget and not deps and 'subscribe_in' not in text:
        ty = fields[name]
        func = ('pub(super) fn make_' + name + '(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<' + ty + '> {' + NL
                + text.rstrip(NL) + NL + '    ' + name + NL + '}' + NL)
        out_funcs.append(func)
        new_lines.append('        let ' + name + ' = widgets::make_' + name + '(lang, cx);')
    else:
        new_lines.extend(blk)
    locals_defined.append(name)

uses = [l for l in s.split(NL) if l.startswith('use ')]
hdr = NL.join(uses) + NL + NL + 'use super::state::MainWindow;' + NL + NL
open('crates/wv-ui/src/window/widgets.rs', 'w', encoding='utf-8', newline='').write(hdr + NL.join(out_funcs))
s2 = s[:body_start] + NL.join(new_lines) + NL + s[body_end:]
open(P, 'w', encoding='utf-8', newline='').write(s2)
print(len(out_funcs), 'moved')
