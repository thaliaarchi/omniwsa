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

/// An overloaded interpretation of the arguments of a Whitespace assembly
/// instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Overload {
    /// Unary operation with constant value:
    /// `op n` => `push n / op`.
    /// - `retrieve n`: Burghard, littleBugHunter, Palaiologos, rdebath-Burghard, voliva, Whitelips
    /// - `printc n`: Palaiologos
    /// - `printi n`: Palaiologos
    /// - `readc n`: Palaiologos, Whitelips
    /// - `readi n`: Palaiologos, Whitelips
    UnaryConst,
    /// Unary operation with reference value:
    /// `op var` => `push addr / retrieve / op`.
    /// Not used by any dialect.
    UnaryRef,
    /// Binary operation with constant LHS:
    /// `op n` => `push n / swap / op`.
    /// - `store n`: Burghard, littleBugHunter, voliva, rdebath-Burghard
    BinaryConstLhs,
    /// Binary operation with constant RHS:
    /// `op n` => `push n / op`.
    /// - `add n`: Burghard, littleBugHunter, Palaiologos, rdebath-Burghard, voliva, Whitelips
    /// - `sub n`: Burghard, littleBugHunter, Palaiologos, rdebath-Burghard, voliva, Whitelips
    /// - `mul n`: Burghard, littleBugHunter, Palaiologos, voliva, Whitelips
    /// - `div n`: Burghard, littleBugHunter, Palaiologos, voliva, Whitelips
    /// - `mod n`: Burghard, littleBugHunter, Palaiologos, voliva, Whitelips
    /// - `or n`: voliva
    /// - `and n`: voliva
    /// - `store n`: Palaiologos
    BinaryConstRhs,
    /// Binary operation with reference LHS:
    /// `op var` => `push addr / retrieve / swap / op`.
    /// Not used by any dialect.
    BinaryRefLhs,
    /// Binary operation with reference RHS:
    /// `op var` => `push addr / retrieve / op`.
    /// - `add var`: littleBugHunter
    /// - `sub var`: littleBugHunter
    /// - `mul var`: littleBugHunter
    /// - `div var`: littleBugHunter
    /// - `mod var`: littleBugHunter
    BinaryRefRhs,
    /// Binary operation with constant LHS and RHS:
    /// `op x y` => `push y / push x / op`.
    /// - `store x y`: littleBugHunter, Palaiologos
    BinaryConstConst,
    /// Binary operation with reference LHS and constant RHS:
    /// `op var n` => `push addr / retrieve / push n / op`.
    /// - `add var n`: littleBugHunter
    /// - `sub var n`: littleBugHunter
    /// - `mul var n`: littleBugHunter
    /// - `div var n`: littleBugHunter
    /// - `mod var n`: littleBugHunter
    BinaryRefConst,
    /// Binary operation with constant LHS and reference RHS:
    /// `op n var` => `push n / push addr / retrieve / op`.
    /// - `add n var`: littleBugHunter
    /// - `sub n var`: littleBugHunter
    /// - `mul n var`: littleBugHunter
    /// - `div n var`: littleBugHunter
    /// - `mod n var`: littleBugHunter
    BinaryConstRef,
    /// Binary operation with reference LHS and reference RHS:
    /// `op var1 var2` => `push addr1 / retrieve / push addr2 / retrieve / op`.
    /// - `add var1 var2`: littleBugHunter
    /// - `sub var1 var2`: littleBugHunter
    /// - `mul var1 var2`: littleBugHunter
    /// - `div var1 var2`: littleBugHunter
    /// - `mod var1 var2`: littleBugHunter
    BinaryRefRef,
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
