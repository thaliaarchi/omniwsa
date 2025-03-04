//! Word sequence, separated by spaces.

use std::{
    fmt::{self, Debug, Formatter},
    ops::{Index, IndexMut},
};

use crate::{
    syntax::HasError,
    tokens::{Token, spaces::Spaces},
};

/// A sequence of words, separated and surrounded by optional spaces.
#[derive(Clone, PartialEq, Eq)]
pub struct Words<'s> {
    /// The spaces preceding the first word.
    pub space_before: Spaces<'s>,
    /// Word tokens followed by spaces.
    pub words: Vec<(Token<'s>, Spaces<'s>)>,
}

impl<'s> Words<'s> {
    /// Constructs a sequence with only spaces.
    #[inline]
    pub fn new(space_before: Spaces<'s>) -> Self {
        Words {
            space_before,
            words: Vec::new(),
        }
    }

    /// Gets a reference to the word at the given index and the space
    /// surrounding it.
    #[inline]
    pub fn get_spaced(&self, index: usize) -> (&Spaces<'s>, &Token<'s>, &Spaces<'s>) {
        let (word, space_after) = &self.words[index];
        let space_before = self
            .words
            .get(index - 1)
            .map(|(_, s)| s)
            .unwrap_or(&self.space_before);
        (space_before, word, space_after)
    }

    /// Gets a mutable reference to the word at the given index and the space
    /// surrounding it.
    #[inline]
    pub fn get_spaced_mut(
        &mut self,
        index: usize,
    ) -> (&mut Spaces<'s>, &mut Token<'s>, &mut Spaces<'s>) {
        let (l, r) = self.words.split_at_mut(index);
        let (word, space_after) = &mut r[0];
        let space_before = match l.last_mut() {
            Some((_, space)) => space,
            None => &mut self.space_before,
        };
        (space_before, word, space_after)
    }

    /// Returns the first word in the sequence, if non-empty.
    #[inline]
    pub fn first(&self) -> Option<&Token<'s>> {
        self.words.first().map(|(word, _)| word)
    }

    /// Returns a mutable reference to the first word in the sequence, if
    /// non-empty.
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut Token<'s>> {
        self.words.first_mut().map(|(word, _)| word)
    }

    /// Returns the last word in the sequence, if non-empty.
    #[inline]
    pub fn last(&self) -> Option<&Token<'s>> {
        self.words.last().map(|(word, _)| word)
    }

    /// Returns a mutable reference to the last word in the sequence, if
    /// non-empty.
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut Token<'s>> {
        self.words.last_mut().map(|(word, _)| word)
    }

    /// Gets the spaces before the first word.
    #[inline]
    pub fn leading_spaces(&self) -> &Spaces<'s> {
        &self.space_before
    }

    /// Gets a mutable reference to the spaces before the first word.
    #[inline]
    pub fn leading_spaces_mut(&mut self) -> &mut Spaces<'s> {
        &mut self.space_before
    }

    /// Gets the spaces after the last word.
    #[inline]
    pub fn trailing_spaces(&self) -> &Spaces<'s> {
        self.words
            .last()
            .map(|(_, space)| space)
            .unwrap_or(&self.space_before)
    }

    /// Gets a mutable reference to the spaces after the last word.
    #[inline]
    pub fn trailing_spaces_mut(&mut self) -> &mut Spaces<'s> {
        self.words
            .last_mut()
            .map(|(_, space)| space)
            .unwrap_or(&mut self.space_before)
    }

    /// Appends a word and spaces to the end of the sequence.
    pub fn push(&mut self, word: Token<'s>, space_after: Spaces<'s>) {
        self.words.push((word, space_after));
    }

    /// Appends a word to the end of the sequence.
    pub fn push_word(&mut self, word: Token<'s>) {
        self.words.push((word, Spaces::new()));
    }

    /// Appends a space token to the end of the sequence.
    pub fn push_space(&mut self, space: Token<'s>) {
        self.trailing_spaces_mut().push(space);
    }

    /// Appends a token to the end of the sequence.
    pub fn push_token(&mut self, token: Token<'s>) {
        match token {
            Token::Mnemonic(_)
            | Token::Integer(_)
            | Token::String(_)
            | Token::Char(_)
            | Token::Variable(_)
            | Token::Label(_)
            | Token::LabelColon(_)
            | Token::Word(_)
            | Token::Group(_)
            | Token::Splice(_)
            | Token::Error(_) => self.push_word(token),
            Token::Space(_)
            | Token::LineTerm(_)
            | Token::Eof(_)
            | Token::InstSep(_)
            | Token::ArgSep(_)
            | Token::LineComment(_)
            | Token::BlockComment(_) => self.push_space(token),
            Token::Placeholder => panic!("placeholder"),
        }
    }

    /// Returns the number of words in this sequence.
    #[inline]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Returns whether this sequence contains no words.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Gets a reference to the word at the given index.
impl<'s> Index<usize> for Words<'s> {
    type Output = Token<'s>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.words[index].0
    }
}

/// Gets a mutable reference to the word at the given index.
impl IndexMut<usize> for Words<'_> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.words[index].0
    }
}

impl HasError for Words<'_> {
    fn has_error(&self) -> bool {
        self.space_before.has_error()
            || self
                .words
                .iter()
                .any(|(word, space)| word.has_error() || space.has_error())
    }
}

impl Debug for Words<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Words ")?;
        let mut l = f.debug_list();
        l.entry(&self.space_before);
        for (word, space) in &self.words {
            l.entry(word);
            l.entry(space);
        }
        l.finish()
    }
}
