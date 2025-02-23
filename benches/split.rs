use std::sync::LazyLock;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use regex::Regex;
use regex_automata::{dfa::Automaton, Anchored, Input};

fn split_filename(filename: &str) -> (String, String) {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(.*?)\.(tar\.gz|tar\.xz|tar\.bz2|cpio\.gz|cpio\.bz2|(?:7z|rar|zip)\.\d{3}|[^.]+)$").unwrap()
    });

    match RE.captures(filename) {
        Some(caps) => {
            (caps.get(1).unwrap().as_str().to_string(), caps.get(2).unwrap().as_str().to_string())
        }
        _ => {
            (filename.to_string(), String::new()) // 无扩展名的情况
        }
    }
}

fn split_filename_old(name: &str) -> (String, String) {
    if let Some(dot_pos) = name.rfind('.') {
        if dot_pos == 0 || dot_pos == name.len() - 1 {
            (name.to_string(), String::new())
        } else {
            let (stem, ext_with_dot) = name.split_at(dot_pos);
            let ext = &ext_with_dot[1..];
            (stem.to_string(), ext.to_string())
        }
    } else {
        (name.to_string(), String::new())
    }
}

use regex_automata::{
    dfa::dense::DFA,
    util::{lazy::Lazy, wire::AlignAs},
};

pub static SPLIT: Lazy<DFA<&'static [u32]>> = Lazy::new(|| {
    static ALIGNED: &AlignAs<[u8], u32> = &AlignAs {
        _align: [],
        #[cfg(target_endian = "big")]
        bytes: *include_bytes!("../src/re/split.bigendian.dfa"),
        #[cfg(target_endian = "little")]
        bytes: *include_bytes!("../src/re/split.littleendian.dfa"),
    };
    let (dfa, _) = regex_automata::dfa::dense::DFA::from_bytes(&ALIGNED.bytes).expect("serialized DFA should be valid");
    dfa
});

/// 将文件名拆分为主名和扩展名 FILE_EXTENSION_SPLIT
fn split_filename_new(filename: &str) -> (String, String) {
    let input = Input::new(filename).anchored(Anchored::Yes);
    match SPLIT.try_search_rev(&input) {
        Ok(Some(index)) => {
            let (main, ext) = filename.split_at(index.offset());
            // 去除index位置的点
            (main.into(), ext[1..].into())
        }
        Ok(None) | Err(_) => {
            (filename.to_string(), String::new())
        },
    }
}


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("file.with.dots.txt", |b| b.iter(|| split_filename(black_box("file.with.dots.txt"))));
    c.bench_function("a.b.c.d.f", |b| b.iter(|| split_filename(black_box("a.b.c.d.f"))));

    c.bench_function("file.with.dots.txt.old", |b| b.iter(|| split_filename_old(black_box("file.with.dots.txt"))));
    c.bench_function("a.b.c.d.f.old", |b| b.iter(|| split_filename_old(black_box("a.b.c.d.f"))));

    c.bench_function("file.with.dots.txt.new", |b| b.iter(|| split_filename_new(black_box("file.with.dots.txt"))));
    c.bench_function("a.b.c.d.f.new", |b| b.iter(|| split_filename_new(black_box("a.b.c.d.f"))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
