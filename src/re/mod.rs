use regex_automata::{dfa::Automaton, Anchored, Input};

mod file_extension_split;
mod replace;
pub use crate::re::replace::CompoundReplacer as CompoundReplacer;

/// 将文件名拆分为主名和扩展名 FILE_EXTENSION_SPLIT  
/// 使用 regex_cli 对正则表达式进行预编译，运行时通过读取字节反序列化，减少90%的运行时开销  
/// https://github.com/rust-lang/regex/blob/master/regex-cli/README.md
/// ```powershell
/// regex-cli generate serialize dense dfa `
///  --minimize `
///  --shrink `
///  --start-kind anchored `
///  --rustfmt `
///  --safe `
///  --reverse `
///  --captures none `
///  FILE_EXTENSION_SPLIT `
///  ./src/re/ `
///  "\.(tar\.(?:gz|xz|bz2)|cpio\.(?:gz|bz2)|(?:7z|rar|zip)\.\d{3}|[^.]+)"
/// ```
pub fn split_filename(filename: &str) -> (String, String) {
    let input = Input::new(filename).anchored(Anchored::Yes);
    match file_extension_split::FILE_EXTENSION_SPLIT.try_search_rev(&input) {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_2025_02_17_16_36_27() {
        assert_eq!(split_filename(""), ("".into(), "".into()));
        assert_eq!(split_filename("."), (".".into(), "".into()));
        assert_eq!(split_filename("f"), ("f".into(), "".into()));
        assert_eq!(split_filename(".f"), ("".into(), "f".into()));
        assert_eq!(split_filename("f."), ("f.".into(), "".into()));
        assert_eq!(split_filename("a.b.c.d.f"), ("a.b.c.d".into(), "f".into()));
        assert_eq!(split_filename("abc.tar.gz"), ("abc".into(), "tar.gz".into()));
        assert_eq!(split_filename("abc.7z.001"), ("abc".into(), "7z.001".into()));
        assert_eq!(split_filename("file.with.dots.txt"), ("file.with.dots".into(), "txt".into()));
        assert_eq!(split_filename("no_extension"), ("no_extension".into(), "".into()));
    }
}