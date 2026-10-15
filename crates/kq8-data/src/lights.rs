//! The world's lights for models (`.kqs`): ambient lights, suns and point
//! lights from the `.kq` files in `data/<World>/KQ/Light/`.
//!
//! Rules from the behaviour note `docs/behaviour/object-lighting.md`
//! (session A); rule numbers below are that note's. See
//! docs/provenance/README.md.
//!
//! - On hardware the world runs `swtohwlt.cs`, whose `loadKQ` lines load
//!   `light/SunlgtHW.kq` and `light/AmblgtHW.kq` (rule 1).
//! - The ambient is the sum of all ambient lights; suns and point lights
//!   go into one shared list (rule 3).
//! - Light at a corner = ambient + each sun's colour x max(0, -aim . normal),
//!   each channel capped to 0..1 (rules 4, 5). The aim is used as stored,
//!   not made unit length (rule 5).

use crate::kq_text::Section;

/// A sun: `classType=KQLight`, `tsLightType=1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sun {
    /// `tsLightIntensity.red/green/blue`.
    pub colour: [f32; 3],
    /// `tsLightAim.x/y/z`, as stored: **not** unit length (rule 5).
    pub aim: [f32; 3],
}

/// A point light: `classType=KQPointLight`, `tsLightType=2` (rule 24).
/// Read and logged; not used for drawing yet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub colour: [f32; 3],
    /// `tsLightPosition.x/y/z`, world units.
    pub position: [f32; 3],
    /// `tsAttenuationBegin`: full strength up to here.
    pub begin: f32,
    /// `tsAttenuationEnd`: no light from here on.
    pub end: f32,
}

/// All lights of a world, gathered as rule 3 says.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Lights {
    /// Sum of all ambient lights' colours (rule 3).
    pub ambient: [f32; 3],
    /// How many ambient lights were added.
    pub ambient_count: usize,
    pub suns: Vec<Sun>,
    pub points: Vec<PointLight>,
    /// Things that were not understood (missing values, unknown classes),
    /// for the log.
    pub problems: Vec<String>,
}

impl Lights {
    /// Adds every light in a `.kq` file's sections. `file` names the file
    /// in problem messages.
    pub fn add_sections(&mut self, file: &str, sections: &[Section]) {
        for s in sections {
            let class = s.get("classType").unwrap_or("");
            let mut values = |prefix: &str, keys: [&str; 3]| -> [f32; 3] {
                keys.map(|k| {
                    let key = format!("{prefix}.{k}");
                    // STAND-IN: a missing value counts as 0; the note does
                    // not say (none is missing in the data).
                    match s.number(&key) {
                        Some(Ok(v)) => v as f32,
                        Some(Err(e)) => {
                            self.problems.push(format!("{file} [{}]: {e}, used 0", s.name));
                            0.0
                        }
                        None => {
                            self.problems.push(format!("{file} [{}]: {key} missing, used 0", s.name));
                            0.0
                        }
                    }
                })
            };
            const RGB: [&str; 3] = ["red", "green", "blue"];
            const XYZ: [&str; 3] = ["x", "y", "z"];
            if class.eq_ignore_ascii_case("KQAmbientLight") {
                let c = values("intensity", RGB);
                for (a, v) in self.ambient.iter_mut().zip(c) {
                    *a += v;
                }
                self.ambient_count += 1;
            } else if class.eq_ignore_ascii_case("KQLight") && s.get("tsLightType") == Some("1") {
                let colour = values("tsLightIntensity", RGB);
                let aim = values("tsLightAim", XYZ);
                self.suns.push(Sun { colour, aim });
            } else if class.eq_ignore_ascii_case("KQPointLight") && s.get("tsLightType") == Some("2") {
                let colour = values("tsLightIntensity", RGB);
                let position = values("tsLightPosition", XYZ);
                let [begin, end] = ["tsAttenuationBegin", "tsAttenuationEnd"].map(|k| match s.number(k) {
                    Some(Ok(v)) => v as f32,
                    _ => {
                        self.problems.push(format!("{file} [{}]: {k} missing or not a number, used 0", s.name));
                        0.0
                    }
                });
                self.points.push(PointLight { colour, position, begin, end });
            } else {
                self.problems.push(format!(
                    "{file} [{}]: classType={class} tsLightType={} is not a light this reader handles; skipped",
                    s.name,
                    s.get("tsLightType").unwrap_or("-")
                ));
            }
        }
    }

    /// Light at a model corner whose **unit** normal is `normal`, in the
    /// same space as the suns' aims (world space): ambient + each sun's
    /// colour x max(0, -aim . normal), each channel capped to 0..1
    /// (rules 4, 5). Point lights are left out (not drawn yet).
    pub fn corner_light(&self, normal: [f32; 3]) -> [f32; 3] {
        let mut light = self.ambient;
        for sun in &self.suns {
            let facing = -(sun.aim[0] * normal[0] + sun.aim[1] * normal[1] + sun.aim[2] * normal[2]);
            if facing > 0.0 {
                for (l, c) in light.iter_mut().zip(sun.colour) {
                    *l += c * facing;
                }
            }
        }
        light.map(|v| v.clamp(0.0, 1.0))
    }
}

/// One grey value for a flat-shaded triangle (rule 7): 0.299 R + 0.587 G +
/// 0.114 B, capped at 1. The usual weights, not the original's swapped
/// green/blue ("Do not reproduce").
pub fn flat_grey(light: [f32; 3]) -> f32 {
    (0.299 * light[0] + 0.587 * light[1] + 0.114 * light[2]).min(1.0)
}

/// The files a script loads with `loadKQ <path>` (rule 1: `swtohwlt.cs`
/// loads the hardware lights), in order. Paths are as written, relative
/// to the world's `KQ` folder. Lines starting with `#` and text after
/// `//` are comments.
pub fn load_kq_lines(script: &[u8]) -> Vec<String> {
    let text: String = script.iter().map(|&b| b as char).collect();
    text.lines()
        .map(|l| l.split("//").next().unwrap_or("").trim())
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let mut words = l.split_whitespace();
            let first = words.next()?;
            first.eq_ignore_ascii_case("loadKQ").then(|| words.next()).flatten().map(str::to_string)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kq_text;

    /// Daventry's hardware light files, as in the data.
    fn daventry() -> Lights {
        let mut l = Lights::default();
        l.add_sections(
            "SunlgtHW.kq",
            &kq_text::parse(
                b"[KQLightHW]\nclassType=KQLight\ntsLightType=1\ntsLightAim.x=0\ntsLightAim.y=1\ntsLightAim.z=-0.66\n\
                  tsLightIntensity.red=0.1\ntsLightIntensity.green=0.1\ntsLightIntensity.blue=0.1\n",
            ),
        );
        l.add_sections(
            "AmblgtHW.kq",
            &kq_text::parse(b"[KQAmbientLightHW]\nclassType=KQAmbientLight\nintensity.red=0.3\nintensity.green=0.3\nintensity.blue=0.3\n"),
        );
        l
    }

    fn close(a: [f32; 3], b: f32) -> bool {
        a.iter().all(|v| (v - b).abs() < 1e-4)
    }

    #[test]
    fn daventry_corner_light() {
        let l = daventry();
        assert!(l.problems.is_empty(), "{:?}", l.problems);
        assert_eq!((l.ambient_count, l.suns.len()), (1, 1));
        // The note's table: facing away 0.3, facing up 0.366, facing the
        // sun squarely 0.3 + 0.1 x 1.198 = 0.42.
        let aim = l.suns[0].aim;
        let len = (aim[0] * aim[0] + aim[1] * aim[1] + aim[2] * aim[2]).sqrt();
        assert!((len - 1.198).abs() < 1e-3);
        let away = aim.map(|v| v / len);
        let towards = away.map(|v| -v);
        assert!(close(l.corner_light(away), 0.3));
        assert!(close(l.corner_light([0.0, 0.0, 1.0]), 0.366));
        assert!(close(l.corner_light(towards), 0.3 + 0.1 * len));
    }

    #[test]
    fn ambients_add_and_light_is_capped() {
        // Swamp: ambient 0.3, sun 0.6 -> 0.3 + 0.6 x 1.198 = 1.02 -> 1.
        let mut l = Lights::default();
        let sun = b"[s]\nclassType=KQLight\ntsLightType=1\ntsLightAim.x=0\ntsLightAim.y=1\ntsLightAim.z=-0.66\n\
                    tsLightIntensity.red=0.6\ntsLightIntensity.green=0.6\ntsLightIntensity.blue=0.6\n";
        l.add_sections("sun", &kq_text::parse(sun));
        let amb = b"[a]\nclassType=KQAmbientLight\nintensity.red=0.2\nintensity.green=0.1\nintensity.blue=0.0\n";
        l.add_sections("a1", &kq_text::parse(amb));
        let amb2 = b"[a]\nclassType=KQAmbientLight\nintensity.red=0.1\nintensity.green=0.2\nintensity.blue=0.3\n";
        l.add_sections("a2", &kq_text::parse(amb2));
        assert!(close(l.ambient, 0.3));
        assert_eq!(l.ambient_count, 2);
        let len = (1.0f32 + 0.66 * 0.66).sqrt();
        let towards = [0.0, -1.0 / len, 0.66 / len];
        assert_eq!(l.corner_light(towards), [1.0; 3]);
        // A negative light value is capped at 0.
        let dark = Lights { ambient: [-0.5, 0.2, 2.0], ..Default::default() };
        assert_eq!(dark.corner_light([0.0, 0.0, 1.0]), [0.0, 0.2, 1.0]);
    }

    #[test]
    fn point_lights_and_unknown_classes() {
        let mut l = Lights::default();
        let point = b"[p]\nclassType=KQPointLight\ntsLightType=2\ntsLightPosition.x=236333\ntsLightPosition.y=147697\n\
                      tsLightPosition.z=9000\ntsLightIntensity.red=1\ntsLightIntensity.green=0.5\ntsLightIntensity.blue=0\n\
                      tsAttenuationBegin=500.0\ntsAttenuationEnd=5000.0\n";
        l.add_sections("facelt01.KQ", &kq_text::parse(point));
        assert_eq!(
            l.points,
            vec![PointLight { colour: [1.0, 0.5, 0.0], position: [236333.0, 147697.0, 9000.0], begin: 500.0, end: 5000.0 }]
        );
        // Point lights don't change the corner light (not drawn yet).
        assert_eq!(l.corner_light([0.0, 0.0, 1.0]), [0.0; 3]);
        l.add_sections("torchlight.kq", &kq_text::parse(b"[f]\nclassType=KQFlickerLight\ntsLightType=1\n"));
        assert_eq!(l.problems.len(), 1);
        assert!(l.suns.is_empty());
        // A missing value is 0 and reported.
        l.add_sections("x", &kq_text::parse(b"[a]\nclassType=KQAmbientLight\nintensity.red=0.5\n"));
        assert_eq!(l.ambient, [0.5, 0.0, 0.0]);
        assert_eq!(l.problems.len(), 3);
    }

    #[test]
    fn flat_grey_weights() {
        assert!((flat_grey([0.4; 3]) - 0.4).abs() < 1e-6);
        assert!((flat_grey([0.0, 1.0, 0.0]) - 0.587).abs() < 1e-6);
        assert!((flat_grey([0.0, 0.0, 1.0]) - 0.114).abs() < 1e-6);
        assert_eq!(flat_grey([1.0; 3]), 1.0);
    }

    #[test]
    fn script_load_lines() {
        let script = b"\r\n# Delete software's sunlight and ambient light\r\ndeleteObject KQLight\r\n\
                       deleteObject KQAmbientLight\r\n\r\n# Now load up the hardware's sunlight and ambient\r\n\
                       loadKQ light/SunlgtHW.kq\r\nloadKQ light/AmblgtHW.kq\r\n// loadKQ light/no.kq\r\n";
        assert_eq!(load_kq_lines(script), vec!["light/SunlgtHW.kq", "light/AmblgtHW.kq"]);
    }
}
