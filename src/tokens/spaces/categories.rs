//! Categories of whitespace characters.

use enumset::{enum_set, EnumSet, EnumSetType};

/// A grouping of Unicode codepoints variously treated as whitespace characters.
#[derive(EnumSetType, Debug)]
pub enum SpaceCategory {
    /// Null (U+0000)
    Nul,
    /// Character Tabulation (U+0009)
    Tab,
    /// Line Feed (U+000A)
    LineFeed,
    /// Line Tabulation (U+000B)
    VerticalTab,
    /// Form Feed (U+000C)
    FormFeed,
    /// Carriage Return (U+000D)
    CarriageReturn,
    /// Space (U+0020)
    Space,
    /// Next Line (U+0085)
    NextLine,
    /// Unicode Space Separator (Zs) category, minus Space (U+0020).
    /// - No-Break Space (U+00A0)
    /// - Ogham Space Mark (U+1680)
    /// - En Quad (U+2000)
    /// - Em Quad (U+2001)
    /// - En Space (U+2002)
    /// - Em Space (U+2003)
    /// - Three-Per-Em Space (U+2004)
    /// - Four-Per-Em Space (U+2005)
    /// - Six-Per-Em Space (U+2006)
    /// - Figure Space (U+2007)
    /// - Punctuation Space (U+2008)
    /// - Thin Space (U+2009)
    /// - Hair Space (U+200A)
    /// - Narrow No-Break Space (U+202F)
    /// - Medium Mathematical Space (U+205F)
    /// - Ideographic Space (U+3000)
    SpaceSeparatorMinusSpace,
    /// Unicode Line Separator (Zl) category.
    /// - Line Separator (U+2028)
    LineSeparator,
    /// Unicode Paragraph Separator (Zp) category.
    /// - Paragraph Separator (U+2029)
    ParagraphSeparator,
    /// Zero Width No-Break Space (U+FEFF), i.e., the codepoint for the byte
    /// order mark.
    ZeroWidthNoBreakSpace,
}

impl SpaceCategory {
    /// Whitespace characters according to C [`isspace`](https://en.cppreference.com/w/c/string/byte/isspace)
    /// from `<ctype.h>`.
    pub const C_ISSPACE: EnumSet<SpaceCategory> = enum_set!(
        SpaceCategory::Tab
            | SpaceCategory::LineFeed
            | SpaceCategory::VerticalTab
            | SpaceCategory::FormFeed
            | SpaceCategory::CarriageReturn
            | SpaceCategory::Space
    );

    /// Whitespace characters according to C# [`Char.IsWhitespace`](https://learn.microsoft.com/en-us/dotnet/api/system.char.iswhitespace?view=net-9.0#system-char-iswhitespace(system-char)),
    /// which is used by [`String.Trim`](https://learn.microsoft.com/en-us/dotnet/api/system.string.trim?view=net-9.0#system-string-trim).
    pub const CSHARP_IS_WHITESPACE: EnumSet<SpaceCategory> = Self::UNICODE_WHITE_SPACE;

    /// Whitespace characters according to GNU sed `\s`, which defers to
    /// `ìsspace` via [Gnulib](https://git.savannah.gnu.org/cgit/gnulib.git/tree/lib/regcomp.c?id=38b5fabdfcf0ddd516fdd9105ccb1b2ac38cb62c#n3515).
    pub const GNU_SED_SLASH_S: EnumSet<SpaceCategory> = Self::C_ISSPACE;

    /// Whitespace characters according to JavaScript, which is used by
    /// `RegExp` [`\s`](https://tc39.es/ecma262/multipage/text-processing.html#sec-compiletocharset)
    /// and [`String.prototype.trim`](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.trim),
    /// among others. Specifically, this is the union of the ECMAScript
    /// [`WhiteSpace`](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#prod-WhiteSpace)
    /// and [`LineTerminator`](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#prod-LineTerminator)
    /// productions.
    pub const JAVASCRIPT: EnumSet<SpaceCategory> = enum_set!(
        SpaceCategory::Tab
            | SpaceCategory::LineFeed
            | SpaceCategory::VerticalTab
            | SpaceCategory::FormFeed
            | SpaceCategory::CarriageReturn
            | SpaceCategory::Space
            | SpaceCategory::SpaceSeparatorMinusSpace
            | SpaceCategory::LineSeparator
            | SpaceCategory::ParagraphSeparator
            | SpaceCategory::ZeroWidthNoBreakSpace
    );

    /// Whitespace characters according to Ruby `Regexp` [`\s`](https://docs.ruby-lang.org/en/master/Regexp.html#class-Regexp-label-Shorthand+Character+Classes).
    /// Unlike `String#strip`, it does not include NUL.
    pub const RUBY_SLASH_S: EnumSet<SpaceCategory> = Self::C_ISSPACE;

    /// Whitespace characters according to Ruby [`String#strip`](https://docs.ruby-lang.org/en/master/String.html#class-String-label-Whitespace+in+Strings).
    /// Unlike `Regexp` `\s`, it includes NUL.
    ///
    /// Before [PR#4164](https://github.com/ruby/ruby/pull/4164)
    /// (Make String#{strip,lstrip}{,!} strip leading NUL bytes, 2021-02-19),
    /// `String#strip` only stripped NUL bytes on the right, even though
    /// documentation indicated that it trimmed NUL on both ends.
    pub const RUBY_STRIP: EnumSet<SpaceCategory> = enum_set!(
        SpaceCategory::Nul
            | SpaceCategory::Tab
            | SpaceCategory::LineFeed
            | SpaceCategory::VerticalTab
            | SpaceCategory::FormFeed
            | SpaceCategory::CarriageReturn
            | SpaceCategory::Space
    );

    /// Whitespace characters according to Rust [`char::is_ascii_whitespace`]
    /// and [`u8::is_ascii_whitespace`].
    pub const RUST_IS_ASCII_WHITESPACE: EnumSet<SpaceCategory> = enum_set!(
        SpaceCategory::Tab
            | SpaceCategory::LineFeed
            | SpaceCategory::FormFeed
            | SpaceCategory::CarriageReturn
            | SpaceCategory::Space
    );

    /// Whitespace characters according to Rust [`char::is_whitespace`].
    pub const RUST_IS_WHITESPACE: EnumSet<SpaceCategory> = Self::UNICODE_WHITE_SPACE;

    /// Whitespace characters according to the Unicode White_Space property.
    ///
    /// White_Space is specified in the [Unicode Character Database](https://www.unicode.org/reports/tr44/)
    /// [`PropList.txt`](https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt).
    pub const UNICODE_WHITE_SPACE: EnumSet<SpaceCategory> = enum_set!(
        SpaceCategory::Tab
            | SpaceCategory::LineFeed
            | SpaceCategory::VerticalTab
            | SpaceCategory::FormFeed
            | SpaceCategory::CarriageReturn
            | SpaceCategory::Space
            | SpaceCategory::NextLine
            | SpaceCategory::SpaceSeparatorMinusSpace
            | SpaceCategory::LineSeparator
            | SpaceCategory::ParagraphSeparator
    );
}
