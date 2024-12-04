use std::{
    collections::HashSet,
    fmt::{self, Debug, Formatter},
    fs::File,
    io::Read,
    path::Path,
};

use bstr::ByteSlice;
use glob::glob;

use crate::{
    dialects::{Burghard, Dialect as _, Palaiologos},
    syntax::Pretty,
};

#[test]
fn roundtrip_burghard() {
    let dialect = Burghard::new();
    let mut src = Vec::new();
    let mut pretty = Vec::new();
    for path in glob("tests/third_party/burghard/*.wsa").unwrap() {
        let path = path.unwrap();
        src.clear();
        File::open(&path).unwrap().read_to_end(&mut src).unwrap();
        let cst = dialect.parse(&src);
        pretty.clear();
        cst.pretty(&mut pretty);
        assert_eq!(
            pretty.as_bstr(),
            src.as_bstr(),
            "parse({:?}).pretty()",
            path,
        );
    }
}

#[test]
fn codegen() {
    let dialect = Palaiologos::new();
    let mut src = Vec::new();
    let mut ws_expect = Vec::new();
    let mut ws_generated = String::new();
    for path in [
        "tests/palaiologos/pass/juxtapose.asm",
        "tests/third_party/palaiologos/ws-build-run/rep_putn.asm",
        "tests/third_party/palaiologos/ws-rebuild/copy.bak",
        "tests/third_party/palaiologos/ws-rebuild/slide.bak",
    ] {
        let path = Path::new(path);
        src.clear();
        File::open(&path).unwrap().read_to_end(&mut src).unwrap();
        ws_expect.clear();
        File::open(path.with_extension("ws"))
            .unwrap()
            .read_to_end(&mut ws_expect)
            .unwrap();
        let cst = dialect.parse(&src);
        ws_generated.clear();
        cst.codegen(&mut ws_generated, &HashSet::new()).unwrap();
        assert_eq!(
            DebugStl(ws_generated.as_bytes()),
            DebugStl(&ws_expect),
            "parse({:?}).codegen()",
            path,
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct DebugStl<'a>(&'a [u8]);

impl Debug for DebugStl<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for &b in self.0 {
            match b {
                b' ' => write!(f, "S")?,
                b'\t' => write!(f, "T")?,
                b'\n' => write!(f, "L")?,
                b'S' => write!(f, "\\S")?,
                b'T' => write!(f, "\\T")?,
                b'L' => write!(f, "\\L")?,
                b'\\' => write!(f, "\\\\")?,
                ..=b'\x7f' => write!(f, "{}", char::from(b))?,
                _ => write!(f, "\\x{b:2x}")?,
            }
        }
        Ok(())
    }
}
