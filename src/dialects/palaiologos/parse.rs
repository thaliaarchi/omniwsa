//! Parser for the Palaiologos Whitespace assembly dialect.

use enumset::EnumSet;

use crate::{
    dialects::{palaiologos::lex::Lexer, Palaiologos},
    lex::TokenStream,
    syntax::{Cst, Dialect, Inst, Opcode},
    tokens::{
        label::{LabelStyle, LabelToken},
        spaces::Spaces,
        words::Words,
        ErrorToken, Token,
    },
};

// TODO:
// - Check for argument and space errors.
// - Add `enum ArgStyle { Mnemonic, Bare, WsfWord }`.
// - Error recovery:
//   - Allow interchanging label defs and refs in both positions, e.g., `%l call
//     @l / %l` => `@l call %l / @l` with errors on respective tokens. This
//     should be handled after parsing. Switch % => @ for labels that are not
//     already @-defined. Next, switch @ => % for labels that are defined and
//     not already switched.
//   - Allow using invalid mnemonics as label defs and refs.
//   - Allow adjacent instructions without slashes. At every misplaced mnemonic,
//     unless the previous was `rep` or it's used as a label, start a new token.

/// A parser for the Palaiologos Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Parser<'s, 'd> {
    toks: TokenStream<'s, Lexer<'s, 'd>>,
}

impl<'s, 'd> Parser<'s, 'd> {
    /// Constructs a new parser for Palaiologos-dialect source text.
    pub fn new(src: &'s [u8], dialect: &'d Palaiologos) -> Self {
        Parser {
            toks: TokenStream::new(Lexer::new(src, dialect)),
        }
    }

    /// Parses the CST.
    pub fn parse(&mut self) -> Cst<'s> {
        let mut nodes = Vec::new();
        let mut next_spaces = Spaces::new();
        while !self.toks.eof() {
            let mut words = Words::new(next_spaces);
            next_spaces = Spaces::new();
            let mut separated = false;
            loop {
                match self.toks.curr() {
                    Token::Mnemonic(_)
                    | Token::Integer(_)
                    | Token::Char(_)
                    | Token::String(_)
                    | Token::Label(LabelToken {
                        style: LabelStyle::PercentSigil,
                        ..
                    })
                    | Token::Error(ErrorToken::UnrecognizedChar { .. }) => {
                        if separated && !words.is_empty() {
                            // Split off trailing spaces into the next
                            // instruction.
                            let trailing = &mut words.trailing_spaces_mut().tokens;
                            let i = trailing
                                .iter()
                                .rposition(|tok| !matches!(tok, Token::Space(_) | Token::ArgSep(_)))
                                .map(|i| i + 1)
                                .unwrap_or(0);
                            next_spaces.tokens.extend(trailing.drain(i..));
                            break;
                        }
                        words.push_word(self.toks.advance());
                    }
                    Token::Label(LabelToken {
                        style: LabelStyle::AtSigil,
                        ..
                    }) => {
                        if words.is_empty() {
                            separated = true;
                        }
                        words.push_word(self.toks.advance());
                    }
                    Token::Space(_) | Token::ArgSep(_) => {
                        words.push_space(self.toks.advance());
                    }
                    Token::InstSep(_) => {
                        words.push_space(self.toks.advance());
                        separated = true;
                    }
                    Token::LineComment(_) => {
                        words.push_space(self.toks.advance());
                        words.push_space(self.toks.advance());
                        break;
                    }
                    Token::LineTerm(_) | Token::Eof(_) => {
                        words.push_space(self.toks.advance());
                        break;
                    }
                    _ => panic!("unhandled token"),
                }
            }
            nodes.push(Cst::from(parse_inst(words)));
        }
        Cst::Dialect {
            dialect: Dialect::Palaiologos,
            inner: Box::new(Cst::Block { nodes }),
        }
    }
}

/// Parses the opcode and arguments of an instruction.
fn parse_inst<'s>(words: Words<'s>) -> Inst<'s> {
    if words.is_empty() {
        return Inst {
            opcode: Opcode::Nop,
            words,
            errors: EnumSet::empty(),
        };
    }
    let opcode = match &words[0] {
        Token::Mnemonic(m) => m.opcode,
        Token::Label(LabelToken {
            style: LabelStyle::AtSigil,
            ..
        }) => Opcode::Label,
        Token::Integer(_) | Token::Char(_) => Opcode::Push,
        Token::Label(LabelToken {
            style: LabelStyle::PercentSigil,
            ..
        })
        | Token::String(_)
        | Token::Error(ErrorToken::UnrecognizedChar { .. }) => Opcode::Invalid,
        _ => panic!("unhandled token"),
    };
    Inst {
        opcode,
        words,
        errors: EnumSet::empty(),
    }
}
