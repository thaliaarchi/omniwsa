//! Byte trie for lexing keywords.

use std::{
    fmt::{self, Debug, Formatter},
    iter,
    ops::BitAnd,
};

use crate::tokens::mnemonics::FoldedStr;

// TODO:
// - Insert branches for each folded byte, instead of folding then inserting.
//   Mixing case foldings can lead to missing some conflicts.
// - A fallback case should be added to allow a valid prefix.
// - Replace panic with error type.

/// A prefix tree for lexing keywords.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByteTrie<'s, T> {
    root: Node<'s>,
    entries: Vec<Entry<'s, T>>,
}

/// An entry in a [`ByteTrie`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry<'s, T> {
    /// The text of the keyword.
    pub key: FoldedStr<'s>,
    /// The associated value.
    pub value: T,
}

/// A node in a `ByteTrie`.
#[derive(Clone, PartialEq, Eq)]
enum Node<'s> {
    Branch {
        sparse: ByteSet,
        dense: Vec<Node<'s>>,
    },
    Leaf {
        tail: FoldedStr<'s>,
        entry_index: usize,
    },
}

/// A set of bytes with one bit per byte.
#[derive(Clone, Copy, PartialEq, Eq)]
struct ByteSet([u64; 4]);

impl<'s, T> ByteTrie<'s, T> {
    /// Constructs a new, empty byte trie.
    pub fn new() -> Self {
        ByteTrie {
            root: Node::empty(),
            entries: Vec::new(),
        }
    }

    /// Gets the entry at the key.
    pub fn get(&self, key: &[u8]) -> Option<&Entry<'s, T>> {
        self.root.get(key).map(|i| &self.entries[i])
    }

    /// Inserts the value at the key.
    pub fn insert(&mut self, key: FoldedStr<'s>, value: T) {
        self.root.insert(key, self.entries.len());
        self.entries.push(Entry { key, value });
    }

    /// Returns the number of entries in this trie.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<'s, T> From<Vec<Entry<'s, T>>> for ByteTrie<'s, T> {
    fn from(entries: Vec<Entry<'s, T>>) -> Self {
        let mut root = Node::empty();
        for (i, entry) in entries.iter().enumerate() {
            root.insert(entry.key, i);
        }
        ByteTrie { root, entries }
    }
}

impl<'s> Node<'s> {
    /// Constructs a branch node with no children.
    fn empty() -> Self {
        Node::Branch {
            sparse: ByteSet::new(),
            dense: Vec::new(),
        }
    }

    /// Gets the index of the entry at the key.
    fn get(&self, key: &[u8]) -> Option<usize> {
        match self {
            Node::Branch { sparse, dense } => {
                let Some((&b, key)) = key.split_first() else {
                    return None;
                };
                if sparse.contains(b) {
                    dense[sparse.dense_index(b)].get(key)
                } else {
                    None
                }
            }
            &Node::Leaf { tail, entry_index } => {
                if tail.fold.starts_with(key, tail.bytes) {
                    Some(entry_index)
                } else {
                    None
                }
            }
        }
    }

    /// Inserts the index of the entry at the key.
    fn insert(&mut self, key: FoldedStr<'s>, entry_index: usize) {
        match self {
            Node::Branch { sparse, dense } => {
                let mut key = key.iter();
                let Some(b) = key.next() else {
                    panic!("conflicting keys");
                };
                let i = sparse.dense_index(b);
                if sparse.contains(b) {
                    dense[i].insert(key.as_str(), entry_index);
                } else {
                    sparse.insert(b);
                    let leaf = Node::Leaf {
                        tail: key.as_str(),
                        entry_index,
                    };
                    dense.insert(i, leaf);
                }
            }
            &mut Node::Leaf {
                tail: key1,
                entry_index: entry_index1,
            } => {
                *self = Node::empty();
                if entry_index1 < entry_index {
                    self.insert(key1, entry_index1);
                    self.insert(key, entry_index);
                } else {
                    self.insert(key, entry_index);
                    self.insert(key1, entry_index1);
                }
            }
        }
    }
}

impl ByteSet {
    /// Constructs a new, empty byte set.
    #[inline]
    fn new() -> Self {
        ByteSet([0; 4])
    }

    /// Returns whether the set contains the byte.
    #[inline]
    fn contains(&self, value: u8) -> bool {
        self.0[(value / 64) as usize] & (1 << (value % 64)) != 0
    }

    /// Inserts the byte into the set.
    #[inline]
    fn insert(&mut self, value: u8) {
        self.0[(value / 64) as usize] |= 1 << (value % 64);
    }

    /// Returns the dense index of `value`, i.e., the number of 1 bits in the
    /// set below `value`.
    #[inline]
    fn dense_index(&self, value: u8) -> usize {
        (*self & Self::mask_below(value)).len()
    }

    /// Constructs a mask with 1 at any bit index less than `value`, i.e., the
    /// mask has `value` number of least-significant ones.
    #[inline]
    fn mask_below(value: u8) -> Self {
        /// Shift-left is not defined cross-platform for shifts greater than or
        /// equal to the bit width. For such shifts, return 0.
        #[inline(always)]
        fn saturating_shl(lhs: u64, shift: u32) -> u64 {
            lhs.wrapping_shl(shift) & ((shift < 64) as u64).wrapping_mul(u64::MAX)
        }
        // For each word, when `value` is within it, the mask is `shift` number
        // of ones. When below, the mask is all ones, and when above, all zeros.
        ByteSet([
            saturating_shl(1, value.saturating_sub(0) as u32).wrapping_sub(1),
            saturating_shl(1, value.saturating_sub(64) as u32).wrapping_sub(1),
            saturating_shl(1, value.saturating_sub(128) as u32).wrapping_sub(1),
            saturating_shl(1, value.saturating_sub(192) as u32).wrapping_sub(1),
        ])
    }

    /// Returns the number of elements in this set.
    #[inline]
    fn len(&self) -> usize {
        (self.0[0].count_ones()
            + self.0[1].count_ones()
            + self.0[2].count_ones()
            + self.0[3].count_ones()) as usize
    }

    /// Iterates the bytes in this set.
    fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        (0u8..).zip(&self.0).flat_map(|(i, &word)| {
            let mut word = word;
            iter::from_fn(move || {
                let tz = word.trailing_zeros();
                if tz == 64 {
                    None
                } else {
                    word &= !(1 << tz);
                    Some(tz as u8 + i * 64)
                }
            })
        })
    }
}

impl BitAnd for ByteSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let (l, r) = (&self.0, &rhs.0);
        ByteSet([l[0] & r[0], l[1] & r[1], l[2] & r[2], l[3] & r[3]])
    }
}

/// A debug representation of a quoted byte.
struct DebugByte(u8);
impl Debug for DebugByte {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'", self.0.escape_ascii())
    }
}

impl Debug for Node<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Node::Branch { sparse, dense } => {
                f.write_str("Branch ")?;
                f.debug_map()
                    .entries(sparse.iter().map(DebugByte).zip(dense))
                    .finish()
            }
            Node::Leaf { tail, entry_index } => f
                .debug_struct("Leaf")
                .field("tail", tail)
                .field("entry_index", entry_index)
                .finish(),
        }
    }
}

impl Debug for ByteSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("ByteSet ")?;
        f.debug_set().entries(self.iter().map(DebugByte)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        // TODO: "j" and "b" are prefixes of other mnemonics.
        let mnemonics = [
            "psh", "push", "dup", "copy", "take", "pull", "xchg", "swp", "swap", "drop", "dsc",
            "slide", "add", "sub", "mul", "div", "mod", "sto", "rcl", "call", "gosub", "jsr",
            "jmp", /* "j", "b", */ "jz", "bz", "jltz", "bltz", "ret", "end", "putc", "putn",
            "getc", "getn", "rep",
        ];
        let mut trie = ByteTrie::new();
        for mnemonic in &mnemonics {
            trie.insert(FoldedStr::ascii(mnemonic.as_bytes()), mnemonic);
        }
        assert_eq!(trie.len(), mnemonics.len());
        for mnemonic in &mnemonics {
            assert_eq!(
                trie.get(mnemonic.as_bytes()),
                Some(&Entry {
                    key: FoldedStr::ascii(mnemonic.as_bytes()),
                    value: mnemonic,
                }),
            );
        }
        // assert_eq!(
        //     trie.get(b"PUSH"),
        //     Some(&Entry {
        //         key: FoldedStr::ascii(b"push"),
        //         value: "push",
        //     }),
        // );
    }
}
