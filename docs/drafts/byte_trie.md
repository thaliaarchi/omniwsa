# Byte trie

Problem: Keywords in the Palaiologos dialect are not necessarily delimited by
spaces and need to be lexed by matching iteratively longer chunks until a
keyword is identified. This is the same problem as with lexing a fixed number of
strings to Whitespace tokens.

This is well suited for a trie. It would have branches, which have 2 to 256
children, and leaves, which have the tail of the key and the corresponding data.
Alternatively, the tail could store the full key and slice it starting at the
depth to get the tail, if keys are one-to-one to values.

Since the trie is sparse, we don't want to have `[NodeId; 256]` at every branch;
for a 32-bit ID, that would be 8 KiB. Instead, it could be a 256-bit set, which
indicates whether a child exists at that byte, paired with a dense array of the
children. The index of the child is the pop count of the bitset, masked up the
bit corresponding to the byte value.

If the trie is in terms of bytes, then the scanner needs to separately decode
UTF-8 codepoints for span tracking. Instead, each leaf should record its
line/column delta. However, if a key is a partial prefix of a valid UTF-8
sequence, then this delta is unreliable depending on the following text. Thus,
the trie should restrict its keys to valid UTF-8.

This sparse trie is probably not useful for parsing Whitespace tokens to
instructions, since that grammar is quite dense with few token kinds. It also
wouldn't work for tokens described by regular expressions of infinitely many
texts.

An alternative could be to use a rolling hash function, looking up in a hash
table after each successive byte is read. It should be bounded by the length of
the maximum key, as the hash would give no indication of whether it is a valid
prefix of something to come. Ultimately, this seems to make dubious tradeoffs
for a slightly simpler data structure.

If expensive to construct on demand, the data structures for parsing a dialect
could be saved to disk with a layout that can easily be mapped into memory.
(However, this would require validation before execution for it to be safe.)
