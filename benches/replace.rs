use criterion::{black_box, criterion_group, criterion_main, Criterion};
use regex::Replacer;
use regex::{Captures, Regex};

const RULES: &[(&str, &str)] = &[
    (
        r"[\u{5b}\u{3010}].*(电影|高清|原盘|蓝光|发布).*?[\u{3011}\u{5d}]",
        "",
    ),
    (r"\.", " "),
];

fn compound_replacement(text: &str, compound_re: &Regex, replacements: &[&str]) -> String {
    struct CompoundSwapper<'a> {
        replacements: &'a [&'a str],
        group_names: Vec<String>,
    }

    impl Replacer for CompoundSwapper<'_> {
        fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
            for (i, group_name) in self.group_names.iter().enumerate() {
                if caps.name(group_name).is_some() {
                    dst.push_str(self.replacements[i]);
                    return;
                }
            }
        }
    }

    let group_names = (0..replacements.len())
        .map(|i| format!("group{}", i))
        .collect();

    compound_re
        .replace_all(
            text,
            CompoundSwapper {
                replacements,
                group_names,
            },
        )
        .into_owned()
}

fn build_compound_regex(rules: &[(&str, &str)]) -> Regex {
    let pattern = rules
        .iter()
        .enumerate()
        .map(|(i, (pat, _))| format!(r"(?P<group{}>{})", i, pat))
        .collect::<Vec<_>>()
        .join("|");
    Regex::new(&pattern).unwrap()
}

fn compile_rules<'a>(rules: &[(&'a str, &'a str)]) -> Vec<(Regex, &'a str)> {
    rules
        .iter()
        .map(|(pat, repl)| (Regex::new(pat).unwrap(), *repl))
        .collect()
}

fn apply_rename_rules(name: &str, compiled_rules: &[(Regex, &str)]) -> String {
    let mut new_name = name.to_string();
    for (re, replacement) in compiled_rules {
        new_name = re.replace_all(&new_name, *replacement).into_owned();
    }
    new_name.trim().to_string()
}

fn criterion_benchmark(c: &mut Criterion) {
    let text = "【高清影视之家发布 www.WHATMV.com】小丑2：双重妄想[HDR+杜比视界双版本][中文字幕].2024.2160p.UHD.BluRay.Remux.DV.HEVC.TrueHD7.1-ParkHD";

    // Benchmark compound replacement
    let compound_re = build_compound_regex(RULES);
    let replacements: Vec<_> = RULES.iter().map(|(_, repl)| *repl).collect();
    assert_eq!(
        compound_replacement(text, &compound_re, &replacements),
        "小丑2：双重妄想[HDR+杜比视界双版本][中文字幕] 2024 2160p UHD BluRay Remux DV HEVC TrueHD7 1-ParkHD"
    );
    c.bench_function("compound_replacement", |b| {
        b.iter(|| {
            compound_replacement(
                black_box(text),
                black_box(&compound_re),
                black_box(&replacements),
            )
        })
    });

    assert_eq!(
        apply_rename_rules(text, &compile_rules(RULES)),
        "小丑2：双重妄想[HDR+杜比视界双版本][中文字幕] 2024 2160p UHD BluRay Remux DV HEVC TrueHD7 1-ParkHD"
    );
    // Benchmark sequential replacement
    let compiled_rules = compile_rules(RULES);
    c.bench_function("apply_rename_rules", |b| {
        b.iter(|| apply_rename_rules(black_box(text), black_box(&compiled_rules)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
