//! Instruction and predefined macro opcodes.

/// Instruction or predefined macro opcode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opcode {
    // Whitespace instructions:
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

    // Extension instructions:
    /// Burghard `debug_printstack`.
    BurghardPrintStack,
    /// Burghard `debug_printheap`.
    BurghardPrintHeap,
    /// voliva `and`.
    VolivaAnd,
    /// voliva `or`.
    VolivaOr,
    /// voliva `not`.
    VolivaNot,
    /// voliva `debugger`.
    VolivaBreakpoint,

    // Additional arguments for standard instructions:
    /// `add` with constant RHS: `add n` => `push n / add`
    /// (Burghard, littleBugHunter, Palaiologos, rdebath-Burghard, voliva, Whitelips).
    AddConstRhs,
    /// `sub` with constant RHS: `sub n` => `push n / sub`
    /// (Burghard, littleBugHunter, Palaiologos, rdebath-Burghard, voliva, Whitelips).
    SubConstRhs,
    /// `mul` with constant RHS: `mul n` => `push n / mul`
    /// (Burghard, littleBugHunter, Palaiologos, voliva, Whitelips).
    MulConstRhs,
    /// `div` with constant RHS: `div n` => `push n / div`
    /// (Burghard, littleBugHunter, Palaiologos, voliva, Whitelips).
    DivConstRhs,
    /// `mod` with constant RHS: `mod n` => `push n / mod`
    /// (Burghard, littleBugHunter, Palaiologos, voliva, Whitelips).
    ModConstRhs,
    /// `add` with a referenced RHS:
    /// `add var` => `push addr / retrieve / add` (littleBugHunter).
    AddRefRhs,
    /// `sub` with a referenced RHS:
    /// `sub var` => `push addr / retrieve / sub` (littleBugHunter).
    SubRefRhs,
    /// `mul` with a referenced RHS:
    /// `mul var` => `push addr / retrieve / mul` (littleBugHunter).
    MulRefRhs,
    /// `div` with a referenced RHS:
    /// `div var` => `push addr / retrieve / div` (littleBugHunter).
    DivRefRhs,
    /// `mod` with a referenced RHS:
    /// `mod var` => `push addr / retrieve / mod` (littleBugHunter).
    ModRefRhs,
    /// `add` with a referenced LHS and constant RHS:
    /// `add var n` => `push addr / retrieve / push n / add` (littleBugHunter).
    AddRefConst,
    /// `sub` with a referenced LHS and constant RHS:
    /// `sub var n` => `push addr / retrieve / push n / sub` (littleBugHunter).
    SubRefConst,
    /// `mul` with a referenced LHS and constant RHS:
    /// `mul var n` => `push addr / retrieve / push n / mul` (littleBugHunter).
    MulRefConst,
    /// `div` with a referenced LHS and constant RHS:
    /// `div var n` => `push addr / retrieve / push n / div` (littleBugHunter).
    DivRefConst,
    /// `mod` with a referenced LHS and constant RHS:
    /// `mod var n` => `push addr / retrieve / push n / mod` (littleBugHunter).
    ModRefConst,
    /// `add` with a constant LHS and referenced RHS:
    /// `add n var` => `push n / push addr / retrieve / add` (littleBugHunter).
    AddConstRef,
    /// `sub` with a constant LHS and referenced RHS:
    /// `sub n var` => `push n / push addr / retrieve / sub` (littleBugHunter).
    SubConstRef,
    /// `mul` with a constant LHS and referenced RHS:
    /// `mul n var` => `push n / push addr / retrieve / mul` (littleBugHunter).
    MulConstRef,
    /// `div` with a constant LHS and referenced RHS:
    /// `div n var` => `push n / push addr / retrieve / div` (littleBugHunter).
    DivConstRef,
    /// `mod` with a constant LHS and referenced RHS:
    /// `mod n var` => `push n / push addr / retrieve / mod` (littleBugHunter).
    ModConstRef,
    /// `add` with a referenced LHS and referenced RHS:
    /// `add var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / add` (littleBugHunter).
    AddRefRef,
    /// `sub` with a referenced LHS and referenced RHS:
    /// `sub var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / sub` (littleBugHunter).
    SubRefRef,
    /// `mul` with a referenced LHS and referenced RHS:
    /// `mul var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / mul` (littleBugHunter).
    MulRefRef,
    /// `div` with a referenced LHS and referenced RHS:
    /// `div var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / div` (littleBugHunter).
    DivRefRef,
    /// `mod` with a referenced LHS and referenced RHS:
    /// `mod var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / mod` (littleBugHunter).
    ModRefRef,
    /// `store` with constant LHS: `store n` => `push n / swap / store`
    /// (Burghard, littleBugHunter, rdebath-Burghard).
    StoreConstLhs,
    /// `store` with constant RHS: `store n` => `push n / store`
    /// (Palaiologos).
    StoreConstRhs,
    /// `store` with constant LHS and RHS: `store x y` => `push y / push x / store`
    /// (littleBugHunter, Palaiologos).
    StoreConstConst,
    /// `retrieve` with constant: `retrieve n` => `push n / retrieve`
    /// (Burghard, littleBugHunter, Palaiologos, rdebath-Burghard, Whitelips).
    RetrieveConst,
    /// `printc` with constant: `printc n` => `push n / printc`
    /// (Palaiologos).
    PrintcConst,
    /// `printi` with constant: `printi n` => `push n / printi`
    /// (Palaiologos).
    PrintiConst,
    /// `readc` with constant: `readc n` => `push n / readc`
    /// (Palaiologos, Whitelips).
    ReadcConst,
    /// `readi` with constant: `readi n` => `push n / readi`
    /// (Palaiologos, Whitelips).
    ReadiConst,

    // Predefined macros:
    /// Whitelips `push` with a string: `push s` => `push c` for each character
    /// in `s` in reverse order.
    PushString,
    /// Burghard `pushs`: `pushs s` => `push c` for each character in `s` and 0
    /// in reverse order.
    PushString0,
    /// voliva `storestr`: `storestr s` => `dup / push c / store / push 1 / add`
    /// for each character in `s` and 0.
    StoreString0,
    /// Burghard `jumpp`: `jumpp l` =>
    /// ```wsa
    ///     dup / jn __trans__{pc}__0__
    ///     dup / jz __trans__{pc}__0__
    ///     drop / jmp l
    /// __trans__{pc}__0__:
    ///     drop
    /// ```
    BurghardJmpP,
    /// Burghard `jumpnp` or `jumppn`. `jumpnp l` =>
    /// ```wsa
    ///     jz __trans__{pc}__1__
    ///     jmp l
    /// __trans__{pc}__1__:
    /// ```
    BurghardJmpNP,
    /// Burghard `jumpnz`: `jumpnz l` =>
    /// ```wsa
    ///     dup / jn __trans__{pc}__2__
    ///     dup / jz __trans__{pc}__2__
    ///     jmp __trans__{pc}__3__
    /// __trans__{pc}__2__:
    ///     drop / jmp l
    /// __trans__{pc}__3__:
    ///     drop
    /// ```
    BurghardJmpNZ,
    /// Burghard `jumppz`:
    /// `jumppz l` => `jn __trans__{pc}__4__ / jmp l / __trans__{pc}__4__:`
    BurghardJmpPZ,
    /// voliva `jumpp`:
    /// `jumpp l` => `push 0 / swap / sub / jn l`.
    VolivaJmpP,
    /// voliva `jumpnp` or `jumppn`:
    /// `jumpnp l` => `jz __internal_label_{id} / jmp l / __internal_label_{id}:`.
    VolivaJmpNP,
    /// voliva `jumpnz`:
    /// `jumpnz l` => `push 1 / sub / jn l`.
    VolivaJmpNZ,
    /// voliva `jumppz`:
    /// `jumppz l` => `jn __internal_label_{id} / jmp l / __internal_label_{id}:`.
    VolivaJmpPZ,
    /// Burghard `test`:
    /// `test n` => `dup / push n / sub` (Burghard and rdebath-Burghard).
    BurghardTest,
    /// Palaiologos `rep`.
    PalaiologosRep,

    /// Burghard `include`.
    BurghardInclude,
    /// Respace `@include`.
    RespaceInclude,
    /// Whitelips `include`.
    WhitelipsInclude,

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

    /// Burghard `valueinteger`.
    BurghardValueInteger,
    /// Burghard `valuestring`.
    BurghardValueString,
    /// Voliva `valueinteger`.
    VolivaValueInteger,
    /// Voliva `valuestring`.
    VolivaValueString,

    /// An invalid mnemonic.
    Invalid,
}
