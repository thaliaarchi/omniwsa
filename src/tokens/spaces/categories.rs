//! Categories of whitespace characters.

use std::fmt::{self, Debug, Formatter};

use enumset::{enum_set, EnumSet, EnumSetType};

use crate::{lex::Scanner, tokens::spaces::SpaceToken};

/// A set of Unicode codepoints treated as whitespace characters.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SpaceSet(pub EnumSet<SpaceCategory>);

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

impl SpaceSet {
    /// Whitespace characters according to C [`isspace`](https://en.cppreference.com/w/c/string/byte/isspace)
    /// from `<ctype.h>`.
    pub const C_ISSPACE: Self = SpaceSet(enum_set!(
        SpaceCategory::Tab
            | SpaceCategory::LineFeed
            | SpaceCategory::VerticalTab
            | SpaceCategory::FormFeed
            | SpaceCategory::CarriageReturn
            | SpaceCategory::Space
    ));

    /// Whitespace characters according to C# [`Char.IsWhitespace`](https://learn.microsoft.com/en-us/dotnet/api/system.char.iswhitespace?view=net-9.0#system-char-iswhitespace(system-char)),
    /// which is used by [`String.Trim`](https://learn.microsoft.com/en-us/dotnet/api/system.string.trim?view=net-9.0#system-string-trim).
    pub const CSHARP_IS_WHITESPACE: Self = Self::UNICODE_WHITE_SPACE;

    /// Whitespace characters according to GNU sed `\s`, which defers to
    /// `ìsspace` via [Gnulib](https://git.savannah.gnu.org/cgit/gnulib.git/tree/lib/regcomp.c?id=38b5fabdfcf0ddd516fdd9105ccb1b2ac38cb62c#n3515).
    pub const GNU_SED_SLASH_S: Self = Self::C_ISSPACE;

    /// Whitespace characters according to JavaScript, which is used by
    /// `RegExp` [`\s`](https://tc39.es/ecma262/multipage/text-processing.html#sec-compiletocharset)
    /// and [`String.prototype.trim`](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.trim),
    /// among others. Specifically, this is the union of the ECMAScript
    /// [`WhiteSpace`](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#prod-WhiteSpace)
    /// and [`LineTerminator`](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#prod-LineTerminator)
    /// productions.
    pub const JAVASCRIPT: Self = SpaceSet(enum_set!(
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
    ));

    /// Whitespace characters according to Ruby `Regexp` [`\s`](https://docs.ruby-lang.org/en/master/Regexp.html#class-Regexp-label-Shorthand+Character+Classes).
    /// Unlike `String#strip`, it does not include NUL.
    pub const RUBY_SLASH_S: Self = Self::C_ISSPACE;

    /// Whitespace characters according to Ruby [`String#strip`](https://docs.ruby-lang.org/en/master/String.html#class-String-label-Whitespace+in+Strings).
    /// Unlike `Regexp` `\s`, it includes NUL.
    ///
    /// Before [PR#4164](https://github.com/ruby/ruby/pull/4164)
    /// (Make String#{strip,lstrip}{,!} strip leading NUL bytes, 2021-02-19),
    /// `String#strip` only stripped NUL bytes on the right, even though
    /// documentation indicated that it trimmed NUL on both ends.
    pub const RUBY_STRIP: Self = SpaceSet(enum_set!(
        SpaceCategory::Nul
            | SpaceCategory::Tab
            | SpaceCategory::LineFeed
            | SpaceCategory::VerticalTab
            | SpaceCategory::FormFeed
            | SpaceCategory::CarriageReturn
            | SpaceCategory::Space
    ));

    /// Whitespace characters according to Rust [`char::is_ascii_whitespace`]
    /// and [`u8::is_ascii_whitespace`].
    pub const RUST_IS_ASCII_WHITESPACE: Self = SpaceSet(enum_set!(
        SpaceCategory::Tab
            | SpaceCategory::LineFeed
            | SpaceCategory::FormFeed
            | SpaceCategory::CarriageReturn
            | SpaceCategory::Space
    ));

    /// Whitespace characters according to Rust [`char::is_whitespace`].
    pub const RUST_IS_WHITESPACE: Self = Self::UNICODE_WHITE_SPACE;

    /// Whitespace characters according to the Unicode White_Space property.
    ///
    /// White_Space is specified in the [Unicode Character Database](https://www.unicode.org/reports/tr44/)
    /// [`PropList.txt`](https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt).
    pub const UNICODE_WHITE_SPACE: Self = SpaceSet(enum_set!(
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
    ));

    /// Returns whether the character matches a whitespace character in this
    /// set.
    pub fn contains(self, ch: char) -> bool {
        if ch <= ' ' {
            self.contains_ascii(ch as u8)
        } else {
            self.0.is_superset(match ch {
                '\u{0085}' => enum_set!(SpaceCategory::NextLine),
                '\u{00A0}' | '\u{1680}' | '\u{2000}' | '\u{2001}' | '\u{2002}' | '\u{2003}'
                | '\u{2004}' | '\u{2005}' | '\u{2006}' | '\u{2007}' | '\u{2008}' | '\u{2009}'
                | '\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => {
                    enum_set!(SpaceCategory::SpaceSeparatorMinusSpace)
                }
                '\u{2028}' => enum_set!(SpaceCategory::LineSeparator),
                '\u{2029}' => enum_set!(SpaceCategory::ParagraphSeparator),
                '\u{FEFF}' => enum_set!(SpaceCategory::ZeroWidthNoBreakSpace),
                _ => EnumSet::empty(),
            })
        }
    }

    /// Returns whether the byte matches an ASCII whitespace character in this
    /// set.
    pub fn contains_ascii(&self, b: u8) -> bool {
        self.0.is_superset(match b {
            b'\0' => enum_set!(SpaceCategory::Nul),
            b'\t' => enum_set!(SpaceCategory::Tab),
            b'\n' => enum_set!(SpaceCategory::LineFeed),
            b'\x0b' => enum_set!(SpaceCategory::VerticalTab),
            b'\x0c' => enum_set!(SpaceCategory::FormFeed),
            b'\r' => enum_set!(SpaceCategory::CarriageReturn),
            b' ' => enum_set!(SpaceCategory::Space),
            _ => EnumSet::empty(),
        })
    }

    /// Returns whether this set of whitespace characters is entirely ASCII.
    pub fn all_ascii(self) -> bool {
        self.0.is_subset(
            SpaceCategory::Nul
                | SpaceCategory::Tab
                | SpaceCategory::LineFeed
                | SpaceCategory::VerticalTab
                | SpaceCategory::FormFeed
                | SpaceCategory::CarriageReturn
                | SpaceCategory::Space,
        )
    }
}

impl From<EnumSet<SpaceCategory>> for SpaceSet {
    fn from(spaces: EnumSet<SpaceCategory>) -> Self {
        SpaceSet(spaces)
    }
}

impl From<SpaceSet> for EnumSet<SpaceCategory> {
    fn from(spaces: SpaceSet) -> Self {
        spaces.0
    }
}

impl Debug for SpaceSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        f.write_str("SpaceSet(")?;
        for category in self.0.iter() {
            if !first {
                f.write_str(" | ")?;
            }
            first = false;
            Debug::fmt(&category, f)?;
        }
        f.write_str(")")
    }
}

impl<'s> Scanner<'s> {
    /// Scans whitespace characters matching the set.
    pub fn scan_spaces(&mut self, space_set: SpaceSet) -> Option<SpaceToken<'s>> {
        let text = if space_set.all_ascii() {
            self.bump_while_ascii(|ch| space_set.contains_ascii(ch))
        } else {
            self.bump_while_char(|ch| space_set.contains(ch))
        };
        if text.is_empty() {
            None
        } else {
            Some(SpaceToken { space: text.into() })
        }
    }
}
