## Whitespace, 11111111111111111111111111111111111111 (38 ones)

Visible representation:

    SSTTSSTTSNNSSNSSSTNSNSTNSTTSSSSNSNTTN

What it does:

         push -38
    loop:
         push 1
          dup
           pnum
          add
         dup
          jn loop

Surprisingly short for a whitespace program. Other considered approaches were pushing a big integer followed by a sequence of duplicate-multiply but this proved to be less efficient.
