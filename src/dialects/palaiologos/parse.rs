//! Parser for the Palaiologos Whitespace assembly dialect.

use std::collections::HashSet;

use enumset::EnumSet;

use crate::{
    dialects::{dialect::DialectState, palaiologos::lex::Lexer, Palaiologos},
    lex::TokenStream,
    syntax::{ArgLayout, Cst, Dialect, Inst, Opcode},
    tokens::{
        label::{LabelError, LabelStyle, LabelToken},
        spaces::{ArgSepError, InstSepError, Spaces},
        words::Words,
        Token,
    },
};

// TODO:
// - Check for argument errors.
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
    pub fn new(src: &'s [u8], dialect: &'d DialectState<Palaiologos>) -> Self {
        Parser {
            toks: TokenStream::new(Lexer::new(src, dialect)),
        }
    }

    /// Parses the CST.
    pub fn parse(&mut self) -> Cst<'s> {
        let mut label_defs = HashSet::new();
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
                    | Token::Error(_) => {
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
                        label,
                        ..
                    }) => {
                        if words.is_empty() {
                            separated = true;
                        }
                        let ident = label.clone();
                        let mut label = self.toks.advance();
                        if !label_defs.insert(ident) {
                            let Token::Label(l) = &mut label else {
                                unreachable!();
                            };
                            l.errors |= LabelError::Redefined;
                        }
                        words.push_word(label);
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
            let mut inst = Inst {
                opcode: Opcode::Invalid,
                arg_layout: ArgLayout::Mnemonic,
                words,
                errors: EnumSet::empty(),
            };
            analyze_inst(&mut inst);
            nodes.push(Cst::from(inst));
        }
        Cst::Dialect {
            dialect: Dialect::Palaiologos,
            inner: Box::new(Cst::Block { nodes }),
        }
    }
}

/// Analyzes the opcode and arguments of an instruction.
fn analyze_inst(inst: &mut Inst<'_>) {
    if inst.words.is_empty() {
        inst.opcode = Opcode::Nop;
        inst.arg_layout = ArgLayout::Bare;
        return;
    }

    (inst.opcode, inst.arg_layout) = match &inst.words[0] {
        Token::Mnemonic(m) => (m.opcode, ArgLayout::Mnemonic),
        Token::Label(LabelToken {
            style: LabelStyle::AtSigil,
            ..
        }) => (Opcode::Label, ArgLayout::Bare),
        Token::Integer(_) | Token::Char(_) => (Opcode::Push, ArgLayout::Bare),
        Token::Label(LabelToken {
            style: LabelStyle::PercentSigil,
            ..
        })
        | Token::String(_)
        | Token::Error(_) => (Opcode::Invalid, ArgLayout::Bare),
        _ => panic!("unhandled token"),
    };

    let args_start = if inst.opcode == Opcode::PalaiologosRep {
        2
    } else {
        1
    };
    let last = inst.words.words.len() - 1;
    analyze_spaces(&mut inst.words.space_before, true, false, false);
    for (i, (_, spaces)) in &mut inst.words.words.iter_mut().enumerate() {
        analyze_spaces(spaces, false, i == last, i >= args_start);
    }
}

/// Analyze spaces to attach errors.
fn analyze_spaces(spaces: &mut Spaces<'_>, leading: bool, trailing: bool, between_args: bool) {
    let mut has_comma = false;
    let mut has_slash = false;
    for space in &mut spaces.tokens {
        match space {
            Token::ArgSep(sep) => {
                if has_comma {
                    sep.errors |= ArgSepError::Multiple;
                } else if between_args {
                    sep.errors |= ArgSepError::NotBetweenArguments;
                }
                has_comma = true;
            }
            Token::InstSep(sep) => {
                if has_slash {
                    sep.errors |= InstSepError::Multiple;
                }
                if leading {
                    sep.errors |= InstSepError::StartOfLine;
                } else if trailing {
                    sep.errors |= InstSepError::EndOfLine;
                }
                has_slash = true;
            }
            _ => {}
        }
    }
}
