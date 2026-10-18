//! Screen layouts (`.gui`) and the tag file (`kqGuiTag.TTAG`), M4 step
//! 4b-2.
//!
//! A layout is a tree of blocks on a 640×480 screen. Every block starts
//! with a 4-letter tag and a size, then a version, an id, flags and a
//! rectangle (left, top, right, bottom); after that come the fields of
//! its kind, each kind adding its own fields after those of the kind it
//! builds on. A dialog (`UIDL` and the kinds built on it) holds child
//! dialogs and child controls. Ids, fonts, pictures and texts are named
//! by numbers ("tags") that the tag file lists with their names.
//!
//! Layout from the community notes `kq8gui.txt` and `kq8tag.txt`
//! (libkq8fpc, GPL-3.0: facts about the layout only; code and wording
//! are our own). Checked by `kq8 gui-verify`. Sources:
//! docs/provenance/README.md.

use std::collections::BTreeMap;

use crate::chunks::{tag_string, u32_at};

/// `flags` bit that marks a dialog (`kq8gui.txt`).
pub const FLAG_DIALOG: u32 = 0x0002_0000;

/// The fields a block's kind adds after the common header.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// `UIDL` (and the caption fields of `UICD`, `CCBD`, `CCAD`).
    Dialog { caption: Option<Caption>, picture: Option<Picture> },
    /// `UIBM`: a picture.
    Bitmap(Picture),
    /// `UITX`, `UIBT`, `UICB`, `UIRB`: text; `CCAB` / `CCBB` add a
    /// bitmap array (a button's pictures).
    Text { text: Text, pictures: Option<Picture> },
    /// `UILI`: a list.
    List { text: Text, items: Vec<i32> },
    /// `UIEB`: an edit box.
    EditBox { font: i32, blink_rate: u32, max_chars: u32 },
    /// `UIAV` / `UIFL`: a video or animation.
    Flic { flic: i32, attribute: u32, modifier: u32, speed: i32 },
    /// `UIVS` / `UIHS`: a scroll bar.
    Scroll([i32; 6]),
}

/// A font, its justification and a text, by tag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Text {
    pub font: i32,
    pub justification: u32,
    pub text: i32,
}

/// A caption dialog's own fields (`UICD`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caption {
    pub y: i32,
    pub text: Text,
}

/// A picture (or bitmap array) by tag, with its three numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Picture {
    pub bitmap: i32,
    pub modifier: u32,
    pub attribute: u32,
    pub reserved: u32,
}

/// One block of a layout.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    pub tag: String,
    pub version: u32,
    pub id: u32,
    pub flags: u32,
    /// Left, top, right, bottom on the 640×480 screen (children's too:
    /// whether they are relative to the parent is not known).
    pub rect: [i32; 4],
    pub kind: Kind,
    pub dialogs: Vec<Control>,
    pub controls: Vec<Control>,
}

impl Control {
    /// Every block of the tree, depth first, with its depth.
    pub fn walk(&self) -> Vec<(usize, &Control)> {
        let mut out = vec![(0, self)];
        for c in self.dialogs.iter().chain(&self.controls) {
            out.extend(c.walk().into_iter().map(|(d, c)| (d + 1, c)));
        }
        out
    }

    /// The block with this id, anywhere in the tree.
    pub fn find(&self, id: u32) -> Option<&Control> {
        self.walk().into_iter().map(|(_, c)| c).find(|c| c.id == id)
    }
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
    end: usize,
}

impl Reader<'_> {
    fn u32(&mut self) -> Result<u32, String> {
        if self.at + 4 > self.end {
            return Err(format!("field at 0x{:X} runs past the block's end 0x{:X}", self.at, self.end));
        }
        let v = u32_at(self.b, self.at);
        self.at += 4;
        Ok(v)
    }
    fn i32(&mut self) -> Result<i32, String> {
        self.u32().map(|v| v as i32)
    }
    fn text(&mut self) -> Result<Text, String> {
        Ok(Text { font: self.i32()?, justification: self.u32()?, text: self.i32()? })
    }
    fn picture(&mut self) -> Result<Picture, String> {
        Ok(Picture { bitmap: self.i32()?, modifier: self.u32()?, attribute: self.u32()?, reserved: self.u32()? })
    }
}

/// Reads one block at `at`; returns it and where it ends.
fn read_block(b: &[u8], at: usize) -> Result<(Control, usize), String> {
    if at + 8 > b.len() {
        return Err(format!("block header at 0x{at:X} runs past the end"));
    }
    let tag = tag_string(&b[at..at + 4].try_into().unwrap());
    let size = u32_at(b, at + 4) as usize;
    let end = at + 8 + size;
    if end > b.len() {
        return Err(format!("'{tag}' at 0x{at:X}: size {size} runs past the end"));
    }
    let mut r = Reader { b, at: at + 8, end };
    let version = r.u32()?;
    let id = r.u32()?;
    let flags = r.u32()?;
    let rect = [r.i32()?, r.i32()?, r.i32()?, r.i32()?];
    let (mut dialogs, mut controls) = (Vec::new(), Vec::new());
    let kind = match tag.as_str() {
        "UIDL" | "UICD" | "CCBD" | "CCAD" => {
            let n_dialogs = r.i32()?;
            let n_controls = r.i32()?;
            if !(0..=1000).contains(&n_dialogs) || !(0..=1000).contains(&n_controls) {
                return Err(format!("'{tag}' at 0x{at:X}: odd child counts {n_dialogs}, {n_controls}"));
            }
            for (n, list) in [(n_dialogs, &mut dialogs), (n_controls, &mut controls)] {
                for _ in 0..n {
                    let (c, next) = read_block(b, r.at)?;
                    if next > end {
                        return Err(format!("a child of '{tag}' at 0x{at:X} runs past its end"));
                    }
                    list.push(c);
                    r.at = next;
                }
            }
            let caption = if tag != "UIDL" { Some(Caption { y: r.i32()?, text: r.text()? }) } else { None };
            let picture = if matches!(tag.as_str(), "CCBD" | "CCAD") { Some(r.picture()?) } else { None };
            Kind::Dialog { caption, picture }
        }
        "UIBM" => Kind::Bitmap(r.picture()?),
        "UITX" | "UIBT" | "UICB" | "UIRB" => Kind::Text { text: r.text()?, pictures: None },
        "CCAB" | "CCBB" => Kind::Text { text: r.text()?, pictures: Some(r.picture()?) },
        "UILI" => {
            let (font, justification) = (r.i32()?, r.u32()?);
            let n = r.i32()?;
            if !(0..=1000).contains(&n) {
                return Err(format!("'{tag}' at 0x{at:X}: odd item count {n}"));
            }
            let items = (0..n).map(|_| r.i32()).collect::<Result<_, _>>()?;
            Kind::List { text: Text { font, justification, text: 0 }, items }
        }
        "UIEB" => Kind::EditBox { font: r.i32()?, blink_rate: r.u32()?, max_chars: r.u32()? },
        "UIAV" | "UIFL" => Kind::Flic { flic: r.i32()?, attribute: r.u32()?, modifier: r.u32()?, speed: r.i32()? },
        "UIVS" | "UIHS" => Kind::Scroll([r.i32()?, r.i32()?, r.i32()?, r.i32()?, r.i32()?, r.i32()?]),
        _ => return Err(format!("unknown block '{tag}' at 0x{at:X}")),
    };
    if r.at != end {
        return Err(format!("'{tag}' at 0x{at:X}: {} bytes left unread in the block", end - r.at));
    }
    Ok((Control { tag, version, id, flags, rect, kind, dialogs, controls }, end))
}

/// Reads a whole `.gui` file: one block that must end with the file.
pub fn parse(bytes: &[u8]) -> Result<Control, String> {
    let (c, end) = read_block(bytes, 0)?;
    if end != bytes.len() {
        return Err(format!("the layout ends at {end}, the file has {} bytes", bytes.len()));
    }
    Ok(c)
}

/// The text tag file (`.TTAG`): tag number → (name, string).
#[derive(Debug, Clone, Default)]
pub struct Tags {
    pub by_id: BTreeMap<i32, (String, String)>,
}

impl Tags {
    /// `[TAG]` line, then `<id> <name> <q><string><q>` lines up to `[END]`;
    /// blank lines and `//` comments skipped; the quote is whatever
    /// character follows the name (`kq8tag.txt`). Returns the lines it
    /// could not read, for the log.
    pub fn parse(text: &str) -> (Tags, Vec<String>) {
        let mut tags = Tags::default();
        let mut bad = Vec::new();
        for line in text.lines().skip(1) {
            let l = line.trim();
            if l.is_empty() || l.starts_with("//") {
                continue;
            }
            if l == "[END]" {
                break;
            }
            let mut parts = l.splitn(2, char::is_whitespace);
            let (Some(id), Some(rest)) = (parts.next(), parts.next()) else {
                bad.push(line.to_string());
                continue;
            };
            let rest = rest.trim_start();
            let mut parts = rest.splitn(2, char::is_whitespace);
            let (Some(name), Some(quoted)) = (parts.next(), parts.next()) else {
                bad.push(line.to_string());
                continue;
            };
            let quoted = quoted.trim_start();
            let Some(q) = quoted.chars().next() else {
                bad.push(line.to_string());
                continue;
            };
            let inner = &quoted[q.len_utf8()..];
            let (Ok(id), Some(close)) = (id.parse::<i32>(), inner.find(q)) else {
                bad.push(line.to_string());
                continue;
            };
            tags.by_id.insert(id, (name.to_string(), inner[..close].to_string()));
        }
        (tags, bad)
    }

    pub fn name(&self, id: i32) -> Option<&str> {
        self.by_id.get(&id).map(|(n, _)| n.as_str())
    }

    pub fn string(&self, id: i32) -> Option<&str> {
        self.by_id.get(&id).map(|(_, s)| s.as_str())
    }

    pub fn id_of(&self, name: &str) -> Option<i32> {
        self.by_id.iter().find(|(_, (n, _))| n == name).map(|(id, _)| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(tag: &[u8; 4], body: &[u32]) -> Vec<u8> {
        let mut b = tag.to_vec();
        b.extend((body.len() as u32 * 4).to_le_bytes());
        body.iter().for_each(|v| b.extend(v.to_le_bytes()));
        b
    }

    #[test]
    fn a_dialog_with_a_bitmap_button_and_a_text() {
        let button = block(b"CCBB", &[1, 1076, 3, 0, 0, 48, 48, 0, 12, 0, 147, 0x8000, 8, 0]);
        let text = block(b"UITX", &[1, 542, 3, 10, 400, 630, 470, 383, 4, 0]);
        let mut body = Vec::new();
        for v in [1u32, 176, 0x0012_0003, 0, 0, 640, 480, 0, 2] {
            body.extend(v.to_le_bytes());
        }
        body.extend(button);
        body.extend(text);
        let mut f = b"UIDL".to_vec();
        f.extend((body.len() as u32).to_le_bytes());
        f.extend(body);
        let g = parse(&f).unwrap();
        assert_eq!((g.id, g.rect, g.flags & FLAG_DIALOG != 0), (176, [0, 0, 640, 480], true));
        assert_eq!(g.controls.len(), 2);
        assert_eq!(g.controls[0].kind, Kind::Text { text: Text { font: 0, justification: 12, text: 0 }, pictures: Some(Picture { bitmap: 147, modifier: 0x8000, attribute: 8, reserved: 0 }) });
        assert_eq!(g.find(542).map(|c| c.rect), Some([10, 400, 630, 470]));
        assert_eq!(g.walk().len(), 3);
        // A block with a field too many is refused.
        let mut bad = f.clone();
        bad.extend([0, 0, 0, 0]);
        assert!(parse(&bad).is_err());
    }

    #[test]
    fn the_tag_file_gives_names_and_strings() {
        let (t, bad) = Tags::parse("[TAG]\r\n// note\r\n00000383 IDFNT_20 '20.pft'\r\n  646\tIDSTR_SUBTITLES  ^#27 Subtitles^ trailing\r\nnonsense\r\n[END]\r\n1 AFTER 'x'\r\n");
        assert_eq!(t.name(383), Some("IDFNT_20"));
        assert_eq!(t.string(383), Some("20.pft"));
        assert_eq!(t.string(646), Some("#27 Subtitles"));
        assert_eq!(t.id_of("IDSTR_SUBTITLES"), Some(646));
        assert_eq!(t.by_id.len(), 2, "[END] stops the reading");
        assert_eq!(bad, vec!["nonsense".to_string()]);
    }
}
