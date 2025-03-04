//! Whitespace assembly dialect description.

use std::marker::PhantomData;

use crate::{
    syntax::{Cst, Opcode},
    tokens::{
        Token,
        integer::IntegerSyntax,
        mnemonics::{FoldedStr, MnemonicMap},
    },
};

// TODO:
// - Unify `Dialect::parse` implementations.
// - Remove `Dialect::lex`.

/// A description of how to parse a Whitespace assembly dialect.
pub trait Dialect {
    /// The mnemonic map for this dialect.
    const MNEMONICS: &[(FoldedStr<'_>, &[Opcode])];

    /// Constructs state for the dialect. Only one needs to be constructed for
    /// parsing any number of programs.
    fn new() -> DialectState<Self> {
        DialectState {
            dialect: PhantomData,
            mnemonics: MnemonicMap::from(Self::MNEMONICS),
            integers: Self::make_integers(),
        }
    }

    /// Parses a Whitespace assembly program in the dialect.
    fn parse<'s>(src: &'s [u8], dialect: &DialectState<Self>) -> Cst<'s>;

    /// Lexes a Whitespace assembly program in the dialect.
    fn lex<'s>(src: &'s [u8], dialect: &DialectState<Self>) -> Vec<Token<'s>>;

    /// Constructs an integer syntax description for this dialect.
    fn make_integers() -> IntegerSyntax;
}

/// State for parsing in a Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct DialectState<D: ?Sized> {
    dialect: PhantomData<D>,
    mnemonics: MnemonicMap,
    integers: IntegerSyntax,
}

impl<D: Dialect> DialectState<D> {
    /// Parses a Whitespace assembly program in the dialect.
    pub fn parse<'s>(&self, src: &'s [u8]) -> Cst<'s> {
        D::parse(src, self)
    }

    /// Lexes a Whitespace assembly program in the dialect.
    pub fn lex<'s>(&self, src: &'s [u8]) -> Vec<Token<'s>> {
        D::lex(src, self)
    }

    /// Gets the mnemonic map for this dialect.
    pub fn mnemonics(&self) -> &MnemonicMap {
        &self.mnemonics
    }

    /// Gets the integer syntax description for this dialect.
    pub fn integers(&self) -> &IntegerSyntax {
        &self.integers
    }
}

macro_rules! define_mnemonics {
    (fold = $default_folding:ident, $($($folding:ident)? $mnemonic:literal => [$($opcode:ident),+],)+) => {
        const MNEMONICS: &[($crate::tokens::mnemonics::FoldedStr<'_>, &[$crate::syntax::Opcode])] = {
            static MNEMONICS: &[($crate::tokens::mnemonics::FoldedStr<'_>, &[$crate::syntax::Opcode])] = &[
                $((
                    $crate::tokens::mnemonics::FoldedStr::new_detect(
                        $mnemonic,
                        define_mnemonics!(@fold $($folding)?, $default_folding)
                    ),
                    &[$($crate::syntax::Opcode::$opcode),+],
                )),+
            ];
            MNEMONICS
        };
    };
    (@fold , $default_folding:ident) => {
        $crate::tokens::mnemonics::CaseFold::$default_folding
    };
    (@fold $folding:ident, $default_folding:ident) => {
        $crate::tokens::mnemonics::CaseFold::$folding
    };
}
pub(crate) use define_mnemonics;
