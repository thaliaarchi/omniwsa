#Whitespace, 123 bytes

Visible representation:

    SSNNSSNSNSSSNSNSTNTSTTTSSSTSSSSSNTSSTSNSNTSSNSSSTSSTTSNTSSTNTSTNSSSTNTSSSNSSTNSSNSNSSNSTNTSTNTSTNTSTSSSNSNNNSSSNSNTTNSSNSNN

Unobfuscated program:

        push  0
    loop:
        dup
        push  0
        dup
        ichr
        get
        push  32
        sub
        dup
        jz    space
        push  38
        sub
        jz    fizz
        push  1
        add
    fizz:
        push  0
        dup
        dup
        ichr
        ichr
        ichr
        add
        jmp   loop
    space:
        swap
        pchr
        jmp   loop

There's nothing particularly odd about the implementation, the only real golfing is in some strange reuse of temporaries as well as not caring about the unbounded stack growth to skim down some more bytes.