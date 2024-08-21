use std::{fs::File, io::Read};

use bstr::ByteSlice;
use glob::glob;

use crate::{dialects::Burghard, syntax::Pretty};

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
