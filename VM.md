# VM

## Block & Sequence

```
'a' 1
'b' 2
'c' {
    'd' 3
    'e' 4
}
```

001     Alt(2)                      # Pushes Alt-Frame, 2 instructions
002     CallStatic(0)  # 'a'        #
003     Push1          # 1          # commit
004     Alt(2)                      # Pushes Alt-Frame, 2 instructions
005     CallStatic(1)  # 'b'        #
006     LoadStatic(2)  # 2          # commit
007     Alt(7)                      # Pushes Alt-Frame, 7 instructions
008     CallStatic(3)  # 'c'        #
009         Alt(2)                  # Pushes Alt-Frame, 2 instructions
010         CallStatic(4)  # 'd'    #
011         LoadStatic(5)  # 3      # commit, commit
012         Alt(2)                  # Pushes Alt-Frame, 2 instructions
013         CallStatic(6)  # 'e'    #
014         LoadStatic(7)  # 4      # commit, commit


## Expect

## If

## Loop

## Not

## Repeat
