use std::{
    collections::HashSet,
    error::Error,
    fmt::{self, Debug, Formatter},
    fs::File,
    io::Read,
    path::Path,
};

use bstr::ByteSlice;
use glob::glob;

use crate::{
    dialects::{Burghard, Dialect as _, DialectState, Palaiologos},
    syntax::Pretty,
};

#[test]
fn roundtrip_burghard() {
    let dialect = Burghard::new();
    let mut src = Vec::new();
    let mut pretty = Vec::new();
    let mut fail = false;
    for path in glob("tests/burghard/**/*.wsa").unwrap() {
        let path = path.unwrap();
        src.clear();
        File::open(&path).unwrap().read_to_end(&mut src).unwrap();
        let cst = dialect.parse(&src);
        pretty.clear();
        cst.pretty(&mut pretty);
        if pretty != src {
            println!(
                "parse({path:?}).pretty()\n pretty = {:?}\n    src = {:?}",
                pretty.as_bstr(),
                src.as_bstr(),
            );
            fail = true;
        }
    }
    if fail {
        panic!("fail");
    }
}

#[test]
fn roundtrip_palaiologos() {
    let dialect = Palaiologos::new();
    let mut src = Vec::new();
    let mut pretty = Vec::new();
    let mut fail = false;
    for path in glob("tests/palaiologos/**/*.asm")
        .unwrap()
        .chain(glob("tests/palaiologos/*.bak").unwrap())
    {
        let path = path.unwrap();
        src.clear();
        File::open(&path).unwrap().read_to_end(&mut src).unwrap();
        let cst = dialect.parse(&src);
        pretty.clear();
        cst.pretty(&mut pretty);
        if pretty != src {
            println!(
                "parse({path:?}).pretty()\n pretty = {:?}\n    src = {:?}",
                pretty.as_bstr(),
                src.as_bstr(),
            );
            fail = true;
        }
    }
    if fail {
        panic!("fail");
    }
}

#[test]
fn codegen() {
    #[track_caller]
    fn test(
        dialect: &DialectState<Palaiologos>,
        path: &str,
        src: &mut Vec<u8>,
        ws_expect: &mut Vec<u8>,
        ws_generated: &mut String,
    ) -> Result<bool, Box<dyn Error>> {
        let path = Path::new("tests/palaiologos").join(path);
        src.clear();
        File::open(&path)?.read_to_end(src)?;
        ws_expect.clear();
        File::open(path.with_extension("ws"))?.read_to_end(ws_expect)?;
        let cst = dialect.parse(&src);
        ws_generated.clear();
        cst.codegen(ws_generated, &HashSet::new())?;
        if ws_generated.as_bytes() != ws_expect {
            println!(
                "parse({path:?}).codegen()\n generated = {:?}\n    expect = {:?}",
                DebugStl(ws_generated.as_bytes()),
                DebugStl(&ws_expect),
            );
            Ok(false)
        } else {
            Ok(true)
        }
    }

    let dialect = Palaiologos::new();
    let mut src = Vec::new();
    let mut ws_expect = Vec::new();
    let mut ws_generated = String::new();
    let mut fail = false;
    for path in [
        "pass/integer_bounds/rep_-2^31+1.asm",
        "pass/integer_bounds/rep_-2^31.asm",
        "pass/integer_bounds/rep_negative.asm",
        "pass/juxtapose.asm",
        "pass/regress/no_final_lf.asm",
        "wild/ws-build-run/rep_putn.asm",
        "wild/ws-rebuild/copy.bak",
        "wild/ws-rebuild/slide.bak",
        //// Instruction overloads not resolved
        // "pass/integer_bounds/value_-2^31+1.asm",
        // "pass/integer_bounds/value_-2^31.asm",
        // "pass/integer_bounds/value_2^31-1.asm",
        // "pass/regress/ignored_line_comment.asm",
        // "wild/ws-build-run/divc.asm",
        // "wild/ws-rebuild/halve.bak",

        //// CharToken not handled in WsaInst::integer
        // "pass/char_escapes.asm",
        // "wild/ws-build-run/rep_putc.asm",

        //// Text labels not handled
        // "pass/regress/label_sort_freq.asm",
        // "pass/regress/label_sort_unstable.asm",
        // "wild/ws-build-run/mmltz.asm",
        // "wild/ws-rebuild/binary.bak",
    ] {
        println!("{path}");
        match test(&dialect, path, &mut src, &mut ws_expect, &mut ws_generated) {
            Ok(true) => {}
            Ok(false) => fail = true,
            Err(err) => {
                println!("{path}: {err:?}");
                fail = true;
            }
        }
    }
    if fail {
        panic!("fail");
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
