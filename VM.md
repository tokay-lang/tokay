# VM

Considerations about the new Tokay VM.

## Block

```
'a' 1
'b' 2
'c' {
    'd' 3
    'e' 4
}
```

Fuse: On soft reject of an Op inside the fused area, goto relative continuation address
Forward: Jump forward to relative offset, remove all fuses within.
Backward: Jump backward to relative offset, remove all fuses within.

000     Fused(2, 12)
001     CallStatic(0)  # 'a'
002     Push1          # 1
003     Fused(2, 9)
004     CallStatic(1)  # 'b'
005     LoadStatic(2)  # 2
006     CallStatic(3)  # 'c'
007         Fused(2, 5)
008         CallStatic(4)  # 'd'
009         LoadStatic(5)  # 3
010         CallStatic(6)  # 'e'
011         LoadStatic(7)  # 4

```
'a' if x next "true"
'a' if y next "false"
```

000     Fuse(7)
001     CallStatic(0)  # 'a'
002     LoadGlobal(0)  # x
003     IfFalse(2)
004     Next
005     LoadStatic(1)  # "true"
006     Forward(6)
008     CallStatic(0)  # 'a'
009     LoadGlobal(1)  # y
010     IfFalse(2)
011     Next
012     LoadStatic(2)  # "false"


## Expect

## If

## Loop

## Not

## Repeat

## Sequence
