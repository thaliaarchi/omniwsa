//! Instruction and predefined macro opcodes.

macro_rules! opcodes[
    (
        $(
            $(#[doc = $doc:expr])*
            $opcode:ident $( ( $($arg:ident),+ ) )?
        ),* $(,)?
    ) => {
        /// An opcode for a Whitespace assembly instruction or predefined macro.
        /// Overloaded opcodes are resolved to different variants with distinct
        /// Whitespace code generation.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Opcode {
            $($(#[doc = $doc])* $opcode),*
        }

        impl Opcode {
            /// Returns the argument types expected by this opcode.
            pub fn arg_types(&self) -> &'static [ArgType] {
                match self {
                    $(Opcode::$opcode => &[$($(ArgType::$arg),+)?]),*
                }
            }
        }
    };
];

opcodes! {
    // Standard Whitespace instructions:
    /// Whitespace `push`.
    Push(Integer),
    /// Whitespace `dup`.
    Dup,
    /// Whitespace `copy`.
    Copy(Integer),
    /// Whitespace `swap`.
    Swap,
    /// Whitespace `drop`.
    Drop,
    /// Whitespace `slide`.
    Slide(Integer),
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
    Label(Label),
    /// Whitespace `call`.
    Call(Label),
    /// Whitespace `jmp`.
    Jmp(Label),
    /// Whitespace `jz`.
    Jz(Label),
    /// Whitespace `jn`.
    Jn(Label),
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
    /// voliva `or`.
    VolivaOr,
    /// voliva `not`.
    VolivaNot,
    /// voliva `and`.
    VolivaAnd,
    /// voliva `dbg`.
    VolivaBreakpoint,

    // Standard instructions with overloaded arguments:
    /// `push` with zero value: `push` => `push 0`
    /// (Palaiologos).
    Push0,
    /// `add` with constant RHS: `add n` => `push n / add`
    /// (Burghard, littleBugHunter, Palaiologos, rdebath-Burghard, voliva, Whitelips).
    AddConstRhs(Integer),
    /// `sub` with constant RHS: `sub n` => `push n / sub`
    /// (Burghard, littleBugHunter, Palaiologos, rdebath-Burghard, voliva, Whitelips).
    SubConstRhs(Integer),
    /// `mul` with constant RHS: `mul n` => `push n / mul`
    /// (Burghard, littleBugHunter, Palaiologos, voliva, Whitelips).
    MulConstRhs(Integer),
    /// `div` with constant RHS: `div n` => `push n / div`
    /// (Burghard, littleBugHunter, Palaiologos, voliva, Whitelips).
    DivConstRhs(Integer),
    /// `mod` with constant RHS: `mod n` => `push n / mod`
    /// (Burghard, littleBugHunter, Palaiologos, voliva, Whitelips).
    ModConstRhs(Integer),
    /// voliva `or` with constant RHS: `or n` => `push n / or`
    /// (voliva).
    VolivaOrConstRhs,
    /// voliva `and` with constant RHS: `and n` => `push n / and`
    /// (voliva).
    VolivaAndConstRhs,
    /// `add` with a referenced RHS:
    /// `add var` => `push addr / retrieve / add` (littleBugHunter).
    AddRefRhs(Integer),
    /// `sub` with a referenced RHS:
    /// `sub var` => `push addr / retrieve / sub` (littleBugHunter).
    SubRefRhs(Integer),
    /// `mul` with a referenced RHS:
    /// `mul var` => `push addr / retrieve / mul` (littleBugHunter).
    MulRefRhs(Integer),
    /// `div` with a referenced RHS:
    /// `div var` => `push addr / retrieve / div` (littleBugHunter).
    DivRefRhs(Integer),
    /// `mod` with a referenced RHS:
    /// `mod var` => `push addr / retrieve / mod` (littleBugHunter).
    ModRefRhs(Integer),
    /// `add` with a referenced LHS and constant RHS:
    /// `add var n` => `push addr / retrieve / push n / add` (littleBugHunter).
    AddRefConst(Integer, Integer),
    /// `sub` with a referenced LHS and constant RHS:
    /// `sub var n` => `push addr / retrieve / push n / sub` (littleBugHunter).
    SubRefConst(Integer, Integer),
    /// `mul` with a referenced LHS and constant RHS:
    /// `mul var n` => `push addr / retrieve / push n / mul` (littleBugHunter).
    MulRefConst(Integer, Integer),
    /// `div` with a referenced LHS and constant RHS:
    /// `div var n` => `push addr / retrieve / push n / div` (littleBugHunter).
    DivRefConst(Integer, Integer),
    /// `mod` with a referenced LHS and constant RHS:
    /// `mod var n` => `push addr / retrieve / push n / mod` (littleBugHunter).
    ModRefConst(Integer, Integer),
    /// `add` with a constant LHS and referenced RHS:
    /// `add n var` => `push n / push addr / retrieve / add` (littleBugHunter).
    AddConstRef(Integer, Integer),
    /// `sub` with a constant LHS and referenced RHS:
    /// `sub n var` => `push n / push addr / retrieve / sub` (littleBugHunter).
    SubConstRef(Integer, Integer),
    /// `mul` with a constant LHS and referenced RHS:
    /// `mul n var` => `push n / push addr / retrieve / mul` (littleBugHunter).
    MulConstRef(Integer, Integer),
    /// `div` with a constant LHS and referenced RHS:
    /// `div n var` => `push n / push addr / retrieve / div` (littleBugHunter).
    DivConstRef(Integer, Integer),
    /// `mod` with a constant LHS and referenced RHS:
    /// `mod n var` => `push n / push addr / retrieve / mod` (littleBugHunter).
    ModConstRef(Integer, Integer),
    /// `add` with a referenced LHS and referenced RHS:
    /// `add var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / add` (littleBugHunter).
    AddRefRef(Integer),
    /// `sub` with a referenced LHS and referenced RHS:
    /// `sub var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / sub` (littleBugHunter).
    SubRefRef(Integer),
    /// `mul` with a referenced LHS and referenced RHS:
    /// `mul var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / mul` (littleBugHunter).
    MulRefRef(Integer),
    /// `div` with a referenced LHS and referenced RHS:
    /// `div var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / div` (littleBugHunter).
    DivRefRef(Integer),
    /// `mod` with a referenced LHS and referenced RHS:
    /// `mod var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / mod` (littleBugHunter).
    ModRefRef(Integer),
    /// `store` with constant LHS: `store n` => `push n / swap / store`
    /// (Burghard, littleBugHunter, voliva, rdebath-Burghard).
    StoreConstLhs(Integer),
    /// `store` with constant RHS: `store n` => `push n / store`
    /// (Palaiologos).
    StoreConstRhs(Integer),
    /// `store` with constant LHS and RHS: `store x y` => `push y / push x / store`
    /// (littleBugHunter, Palaiologos).
    StoreConstConst(Integer, Integer),
    /// `retrieve` with constant: `retrieve n` => `push n / retrieve`
    /// (Burghard, littleBugHunter, Palaiologos, rdebath-Burghard, voliva, Whitelips).
    RetrieveConst(Integer),
    /// `printc` with constant: `printc n` => `push n / printc`
    /// (Palaiologos).
    PrintcConst(Integer),
    /// `printi` with constant: `printi n` => `push n / printi`
    /// (Palaiologos).
    PrintiConst(Integer),
    /// `readc` with constant: `readc n` => `push n / readc`
    /// (Palaiologos, Whitelips).
    ReadcConst(Integer),
    /// `readi` with constant: `readi n` => `push n / readi`
    /// (Palaiologos, Whitelips).
    ReadiConst(Integer),

    // Predefined macros:
    /// Whitelips `push` with a string: `push s` => `push c` for each character
    /// in `s` in reverse order.
    PushString(String),
    /// Burghard `pushs`: `pushs s` => `push c` for each character in `s` with a
    /// terminating 0, in reverse order.
    PushString0(String),
    /// voliva `storestr`: `storestr s` => `dup / push c / store / push 1 / add`
    /// for each character in `s` with a terminating 0.
    StoreString0(String),
    /// Burghard `jumpp`: `jumpp l` =>
    /// ```wsa
    ///     dup / jn __trans__{pc}__0__
    ///     dup / jz __trans__{pc}__0__
    ///     drop / jmp l
    /// __trans__{pc}__0__:
    ///     drop
    /// ```
    BurghardJmpPos(Label),
    /// Burghard `jumpnp` or `jumppn`. `jumpnp l` =>
    /// ```wsa
    ///     jz __trans__{pc}__1__
    ///     jmp l
    /// __trans__{pc}__1__:
    /// ```
    BurghardJmpNonZero(Label),
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
    BurghardJmpNonPos(Label),
    /// Burghard `jumppz`:
    /// `jumppz l` => `jn __trans__{pc}__4__ / jmp l / __trans__{pc}__4__:`
    BurghardJmpNonNeg(Label),
    /// voliva `jumpp`:
    /// `jumpp l` => `push 0 / swap / sub / jn l`.
    VolivaJmpPos(Label),
    /// voliva `jumpnp` or `jumppn`:
    /// `jumpnp l` => `jz __internal_label_{id} / jmp l / __internal_label_{id}:`.
    VolivaJmpNonZero(Label),
    /// voliva `jumpnz`:
    /// `jumpnz l` => `push 1 / sub / jn l`.
    VolivaJmpNonPos(Label),
    /// voliva `jumppz`:
    /// `jumppz l` => `jn __internal_label_{id} / jmp l / __internal_label_{id}:`.
    VolivaJmpNonNeg(Label),
    /// Burghard `test`:
    /// `test n` => `dup / push n / sub` (Burghard and rdebath-Burghard).
    BurghardTest(Integer),
    /// Palaiologos `rep`.
    PalaiologosRep(Mnemonic, Integer),

    /// Burghard `include`.
    BurghardInclude(Include),
    /// Respace `@include`.
    RespaceInclude(Include),
    /// voliva `include`.
    VolivaInclude(Include),
    /// Whitelips `include`.
    WhitelipsInclude(Include),

    /// Burghard `option`.
    DefineOption(Option),
    /// Burghard `ifoption` and Respace `@ifdef`.
    IfOption(Option),
    /// Burghard `elseifoption`.
    ElseIfOption(Option),
    /// Burghard `elseoption` and Respace `@else`.
    ElseOption,
    /// Burghard `endoption` and Respace `@endif`.
    EndOption,

    /// Burghard `valueinteger`.
    BurghardValueInteger(Variable, Integer),
    /// Burghard `valuestring`.
    BurghardValueString(Variable, String),
    /// Voliva `valueinteger`.
    VolivaValueInteger(Variable, Integer),
    /// Voliva `valuestring`.
    VolivaValueString(Variable, String),

    /// No operation, e.g., an empty line.
    Nop,

    /// An invalid mnemonic.
    Invalid,
}

/// The type of an argument in an instruction. A variable reference is
/// considered to be the type of ite referent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgType {
    /// An integer value.
    Integer,
    /// A string value.
    String,
    /// A label.
    Label,
    /// A variable identifier.
    Variable,
    /// An include path.
    Include,
    /// An option identifier.
    Option,
    /// An opcode mnemonic.
    Mnemonic,
}
