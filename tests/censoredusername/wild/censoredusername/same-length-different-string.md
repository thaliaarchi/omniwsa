#Whitespace, 59 bytes

Visible representation

    NSSNSSSTSSSSSNSNSSNSSNSTNTSTTTTSSTNTTSNSNSTSSSNSSSNTNSSNSNN

What it does:

For every character it reads it prints a space, except when it's a space, then it prints a @.

Disassembly:

    loop:
        push 32
         dup
          dup
           dup
            ichr
           get
           sub
          jn not_32
         dup
          add
    not_32:
         pchr
        jmp loop

