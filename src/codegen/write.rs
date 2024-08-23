//! Serialization of Whitespace instructions.

use std::{convert::Infallible, mem};

use crate::{codegen::Inst, tokens::integer::Sign};

/// A Whitespace token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    /// Space.
    S,
    /// Tab.
    T,
    /// Line feed.
    L,
}

/// A type, which can write tokens.
pub trait TokenWrite {
    /// An error returned from writing a token.
    type Error;

    /// Writes a token.
    fn write_token(&mut self, token: Token) -> Result<(), Self::Error>;

    /// Writes an instruction.
    fn write_inst(&mut self, inst: Inst<'_>) -> Result<(), Self::Error> {
        use Token::*;
        let (tokens, arg): (&[Token], _) = match &inst {
            Inst::Push(n) => (&[S, S], Some(&n.0)),
            Inst::Dup => (&[S, L, S], None),
            Inst::Copy(n) => (&[S, T, S], Some(&n.0)),
            Inst::Swap => (&[S, L, T], None),
            Inst::Drop => (&[S, L, L], None),
            Inst::Slide(n) => (&[S, T, L], Some(&n.0)),
            Inst::Add => (&[T, S, S, S], None),
            Inst::Sub => (&[T, S, S, T], None),
            Inst::Mul => (&[T, S, S, L], None),
            Inst::Div => (&[T, S, T, S], None),
            Inst::Mod => (&[T, S, T, T], None),
            Inst::Store => (&[T, T, S], None),
            Inst::Retrieve => (&[T, T, T], None),
            Inst::Label(l) => (&[L, S, S], Some(&l.0)),
            Inst::Call(l) => (&[L, S, T], Some(&l.0)),
            Inst::Jmp(l) => (&[L, S, L], Some(&l.0)),
            Inst::Jz(l) => (&[L, T, S], Some(&l.0)),
            Inst::Jn(l) => (&[L, T, T], Some(&l.0)),
            Inst::Ret => (&[L, T, L], None),
            Inst::End => (&[L, L, L], None),
            Inst::Printc => (&[T, L, S, S], None),
            Inst::Printi => (&[T, L, S, T], None),
            Inst::Readc => (&[T, L, T, S], None),
            Inst::Readi => (&[T, L, T, T], None),
            Inst::BurghardPrintStack => (&[L, L, S, S, S], None),
            Inst::BurghardPrintHeap => (&[L, L, S, S, T], None),
            Inst::VolivaOr => (&[T, S, L, S], None),
            Inst::VolivaNot => (&[T, S, L, T], None),
            Inst::VolivaAnd => (&[T, S, L, L], None),
            Inst::VolivaBreakpoint => (&[L, L, S], None),
            Inst::VolivaBreakpointAlt => (&[L, L, T], None),
        };
        for &token in tokens {
            self.write_token(token)?;
        }

        if let Some(arg) = arg {
            match arg.sign {
                Sign::None => {}
                Sign::Pos => self.write_token(S)?,
                Sign::Neg => self.write_token(T)?,
            }
            for _ in 0..arg.leading_zeros {
                self.write_token(S)?;
            }
            let limbs = arg.value.as_limbs();
            if let Some(&high) = limbs.last() {
                let bits = mem::size_of_val(&high) * 8;
                let mut i = bits - high.leading_zeros() as usize;
                for &limb in limbs.iter().rev() {
                    while i != 0 {
                        i -= 1;
                        self.write_token(if limb & (1 << i) == 0 { S } else { T })?;
                    }
                    i = bits;
                }
            }
            self.write_token(L)?;
        }
        Ok(())
    }
}

impl TokenWrite for Vec<Token> {
    type Error = Infallible;

    fn write_token(&mut self, token: Token) -> Result<(), Self::Error> {
        self.push(token);
        Ok(())
    }
}

impl TokenWrite for String {
    type Error = Infallible;

    fn write_token(&mut self, token: Token) -> Result<(), Self::Error> {
        self.push(match token {
            Token::S => ' ',
            Token::T => '\t',
            Token::L => '\n',
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rug::{Complete, Integer};

    use crate::{
        codegen::{Inst, IntegerBits, Token::*, TokenWrite},
        tokens::integer::Sign,
    };

    #[test]
    fn write_inst() {
        let mut s = Vec::new();
        s.write_inst(Inst::Push(IntegerBits::from(
            &Integer::parse("31415926535897932384626433832795028841971693993751")
                .unwrap()
                .complete(),
        )))
        .unwrap();
        assert_eq!(
            s,
            [
                S, S, S, T, S, T, S, T, S, T, T, T, T, T, T, S, T, T, T, S, S, S, T, S, T, T, S, T,
                T, T, T, S, S, S, S, T, S, T, S, T, T, T, T, T, T, T, S, T, S, T, T, T, S, S, T, T,
                S, S, T, T, S, S, T, T, T, S, T, S, T, T, T, T, S, S, T, S, S, S, T, T, T, T, S, T,
                S, T, T, T, S, T, T, S, T, T, T, S, S, S, S, T, T, S, S, T, T, T, S, S, T, S, S, T,
                T, S, S, T, S, S, S, S, S, T, S, T, S, S, S, T, T, T, S, S, T, T, T, T, T, T, S, T,
                S, S, S, T, T, T, T, T, T, T, S, T, S, S, T, T, T, T, T, T, S, S, S, T, S, T, T, T,
                L,
            ],
        );

        s.clear();
        s.write_inst(Inst::Slide(IntegerBits::zero(Sign::None)))
            .unwrap();
        assert_eq!(s, [S, T, L, L]);
    }
}
