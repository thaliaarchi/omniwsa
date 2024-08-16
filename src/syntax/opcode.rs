//! Instruction and predefined macro opcodes.

/// Instruction or predefined macro opcode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opcode {
    /// Whitespace `push`.
    Push,
    /// Whitespace `dup`.
    Dup,
    /// Whitespace `copy`.
    Copy,
    /// Whitespace `swap`.
    Swap,
    /// Whitespace `drop`.
    Drop,
    /// Whitespace `slide`.
    Slide,
    /// Whitespace `add`.
    Add,
    /// Whitespace `sub`.
    Sub,
    /// Whitespace `mul`.
    Mul,
    /// Whitespace `div`.
    Div,
    /// Whitespace `mod`.
    Mod,
    /// Whitespace `store`.
    Store,
    /// Whitespace `retrieve`.
    Retrieve,
    /// Whitespace `label`.
    Label,
    /// Whitespace `call`.
    Call,
    /// Whitespace `jmp`.
    Jmp,
    /// Whitespace `jz`.
    Jz,
    /// Whitespace `jn`.
    Jn,
    /// Whitespace `ret`.
    Ret,
    /// Whitespace `end`.
    End,
    /// Whitespace `printc`.
    Printc,
    /// Whitespace `printi`.
    Printi,
    /// Whitespace `readc`.
    Readc,
    /// Whitespace `readi`.
    Readi,

    /// Burghard `pushs`.
    PushString0,

    /// Burghard `option`.
    DefineOption,
    /// Burghard `ifoption` and Respace `@ifdef`.
    IfOption,
    /// Burghard `elseifoption`.
    ElseIfOption,
    /// Burghard `elseoption` and Respace `@else`.
    ElseOption,
    /// Burghard `endoption` and Respace `@endif`.
    EndOption,

    /// Burghard `include`.
    BurghardInclude,
    /// Respace `@include`.
    RespaceInclude,

    /// Burghard `valueinteger`.
    BurghardValueInteger,
    /// Burghard `valuestring`.
    BurghardValueString,

    /// Burghard `debug_printstack`.
    BurghardPrintStack,
    /// Burghard `debug_printheap`.
    BurghardPrintHeap,

    /// Burghard `jumpp`.
    BurghardJmpP,
    /// Burghard `jumpnp` or `jumppn`.
    BurghardJmpNP,
    /// Burghard `jumpnz`.
    BurghardJmpNZ,
    /// Burghard `jumppz`.
    BurghardJmpPZ,
    /// Burghard `test`.
    BurghardTest,

    /// Palaiologos `rep`.
    PalaiologosRep,

    /// An invalid mnemonic.
    Invalid,
}
