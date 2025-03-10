use crate::logger::LogUnwrap;
use regex::{Captures, Regex, Replacer};
use std::borrow::Borrow;

/// 一种支持多个正则表达式替换的替换器
#[derive(Debug, Clone)]
pub struct CompoundReplacer {
    compound_re: Regex,
    group_names: Vec<String>,
    replacements: Vec<String>,
}

impl CompoundReplacer {
    pub fn new<I, T, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let (patterns, replacements): (Vec<String>, Vec<String>) = pairs
            .into_iter()
            .map(|t| {
                let (k, v) = t.borrow();
                (k.as_ref().to_string(), v.as_ref().to_string())
            })
            .unzip();

        let group_names: Vec<String> = (0..patterns.len())
            .map(|i| format!("_group{}", i))
            .collect();
        let regex_str = patterns
            .iter()
            .enumerate()
            .map(|(i, pat)| format!("(?P<{}>{})", group_names[i], pat))
            .collect::<Vec<_>>()
            .join("|");

        let compound_re = Regex::new(&regex_str).log_unwrap(&format!("Invalid regex: {}", regex_str));

        Self {
            compound_re,
            group_names,
            replacements,
        }
    }

    pub fn replace(&self, text: &str) -> String {
        struct GroupReplacer<'a>(&'a [String], &'a [String]);

        impl Replacer for GroupReplacer<'_> {
            fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
                for (name, rep) in self.0.iter().zip(self.1.iter()) {
                    if caps.name(name).is_some() {
                        dst.push_str(rep);
                        return;
                    }
                }
                dst.push_str(&caps[0]);
            }
        }

        self.compound_re.replace_all(text, GroupReplacer(&self.group_names, &self.replacements)).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2025_02_18_11_33_08() {
        fn compound_replacement(text: &str, replacer: &[(&str, &str)]) -> String {
            let replacer = CompoundReplacer::new(replacer);
            replacer.replace(text)
        }

        assert_eq!(
            compound_replacement(
                "a b c a b c c b a b b a a c a b e f g",
                &[("a", "1"), ("b", "2"), ("c", "3"), (r"[^abc\s]", "4")]
            ),
            "1 2 3 1 2 3 3 2 1 2 2 1 1 3 1 2 4 4 4"
        );
        assert_eq!(
            compound_replacement(
                "【高清影视之家发布 www.WHATMV.com】小丑2：双重妄想[HDR+杜比视界双版本][中文字幕].2024.2160p.UHD.BluRay.Remux.DV.HEVC.TrueHD7.1-ParkHD",
                &[(r"[\[【].*(电影|高清|原盘|蓝光|发布).*?[】\]]", ""), (r"\.", " ")]
            ),
            "小丑2：双重妄想[HDR+杜比视界双版本][中文字幕] 2024 2160p UHD BluRay Remux DV HEVC TrueHD7 1-ParkHD"
        );
    }
}
