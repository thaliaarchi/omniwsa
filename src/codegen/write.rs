//! Serialization of Whitespace instructions.

use std::{convert::Infallible, mem};

use crate::{
    codegen::{Inst, Opcode},
    tokens::integer::Sign,
};

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
    fn write_inst(&mut self, inst: &Inst<'_>) -> Result<(), Self::Error> {
        use Token::*;
        let tokens: &[Token] = match inst.opcode {
            Opcode::Push => &[S, S],
            Opcode::Dup => &[S, L, S],
            Opcode::Copy => &[S, T, S],
            Opcode::Swap => &[S, L, T],
            Opcode::Drop => &[S, L, L],
            Opcode::Slide => &[S, T, L],
            Opcode::Add => &[T, S, S, S],
            Opcode::Sub => &[T, S, S, T],
            Opcode::Mul => &[T, S, S, L],
            Opcode::Div => &[T, S, T, S],
            Opcode::Mod => &[T, S, T, T],
            Opcode::Store => &[T, T, S],
            Opcode::Retrieve => &[T, T, T],
            Opcode::Label => &[L, S, S],
            Opcode::Call => &[L, S, T],
            Opcode::Jmp => &[L, S, L],
            Opcode::Jz => &[L, T, S],
            Opcode::Jn => &[L, T, T],
            Opcode::Ret => &[L, T, L],
            Opcode::End => &[L, L, L],
            Opcode::Printc => &[T, L, S, S],
            Opcode::Printi => &[T, L, S, T],
            Opcode::Readc => &[T, L, T, S],
            Opcode::Readi => &[T, L, T, T],
            Opcode::BurghardPrintStack => &[L, L, S, S, S],
            Opcode::BurghardPrintHeap => &[L, L, S, S, T],
            Opcode::VolivaOr => &[T, S, L, S],
            Opcode::VolivaNot => &[T, S, L, T],
            Opcode::VolivaAnd => &[T, S, L, L],
            Opcode::VolivaBreakpoint => &[L, L, S],
            Opcode::VolivaBreakpointAlt => &[L, L, T],
        };
        for &token in tokens {
            self.write_token(token)?;
        }

        if let Some(arg) = &inst.arg {
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
    use rug::Integer;

    use crate::{
        codegen::{IntegerBits, Opcode, Token::*, TokenWrite},
        tokens::integer::Sign,
    };

    #[test]
    fn write_inst() {
        let mut s = Vec::new();
        s.write_inst(
            &Opcode::Push.integer(
                &Integer::parse("31415926535897932384626433832795028841971693993751")
                    .unwrap()
                    .into(),
            ),
        )
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
        s.write_inst(&Opcode::Slide.integer(IntegerBits::zero(Sign::None)))
            .unwrap();
        assert_eq!(s, [S, T, L, L]);
    }
}
