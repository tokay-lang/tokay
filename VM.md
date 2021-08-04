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

Fuse: On soft reject of an Op inside the fused area, goto relative continuation address
Forward: Jump forward to relative offset, remove all fuses within.
Backward: Jump backward to relative offset, remove all fuses within.

000     Fuse(4)
001     CallStatic(0)  # 'a'
002     Push1          # 1
003     Forward(12)
004     Fuse(4)
005     CallStatic(1)  # 'b'
006     LoadStatic(2)  # 2
007     Forward(8)
008     CallStatic(3)  # 'c'
009         Fuse(4)
010         CallStatic(4)  # 'd'
011         LoadStatic(5)  # 3
012         Forward(2)
013         CallStatic(6)  # 'e'
014         LoadStatic(7)  # 4

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
