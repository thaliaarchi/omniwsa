#Whitespace, 95 bytes

Visible representation:

    SSNSNSTNTTNSSSNSSSTNTSSSSNSSNSSSSTNTSSSTSSNSSSTSNTSTSSSNTTTTSSTSNSNTSNNTTSNSSSTNTNSTNSSNSSNTNST

Reads the number from stdin. If it is triangular, it outputs 0 to stdout. Otherwise, it outputs 10.

Disassembly:

        push 0
         dup
          inum
    loop:
         push 1
          add
         dup
          dup
           push 1
            add
           mul
          push 2
           div
          push 0
           get
           sub
          dup
           jz match
          jn loop
         push 1
          pnum
    match:
         push 0
          pnum

The used technique is a simple brute force search through all possible triangular numbers, but for the given limits that is plenty. Using my own [whitespace JIT compiler][1] it takes 0.6 milliseconds to prove that 10<sup>6</sup> is not a triangular number and about 20 seconds to prove that 10<sup>18</sup> is not a triangular number.


  [1]: https://github.com/CensoredUsername/whitespace-rs