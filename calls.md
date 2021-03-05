# Calls in Tokay

Parselets are very versatile, and can either be used as pure functions or for parsing units, which might also use any other parsable construct.

## Argument-less parselets

### Constant parselet

```
a : @{ 'Hello' }
```

```
a                                   CallStatic(0)               <parselet 0>

(a)                                 LoadStatic(0)               <parselet 0>
```

### Variable parselet

```
a = @{ 'Hello' }
```

```
a                                   LoadGlobal(0)               <parselet 0>
                                    TryCall                     <parselet 0>

(a)                                 LoadGlobal(0)               <parselet 0>
```

## Parselets with arguments

### Constant parselet with variable-only arguments

```
a : @a=10 b=100 {
    a + b
}
```

```
a                                   CallStatic(0)               <parselet 0>

(a)                                 LoadStatic(0)               <parselet 0>

a()                                 CallStatic(0)               <parselet 0>

a(20)                               LoadStatic(1)               20
                                    CallStaticArg((0, 1))

a(b=200)                            LoadStatic(2)               200
                                    LoadStatic(3)               "b"
                                    MakeDict(1)
                                    CallStaticArgX((0, 0))

a(30, b=300)                        LoadStatic(4)               30
                                    LoadStatic(5)               300
                                    LoadStatic(3)               "b"
                                    MakeDict(1)
                                    CallStaticArgX((0, 1))
```

### Variable parselet with variable-only arguments

```
a = @a=10 b=100 {
    a + b
}
```

```
a                                   LoadGlobal(0)               <parselet 0>
                                    TryCall

(a)                                 LoadGlobal(0)               <parselet 0>

a()                                 LoadStatic(1)               <parselet 0>
                                    Call

a(20)                               LoadStatic(1)               20
                                    LoadGlobal(0)
                                    CallArg(1)

a(b=200)                            LoadStatic(2)               200
                                    LoadStatic(3)               "b"
                                    MakeDict(1)
                                    LoadGlobal(0)
                                    CallArgX(0)

a(30, b=300)                        LoadStatic(4)               30
                                    LoadStatic(5)               300
                                    LoadStatic(3)               "b"
                                    MakeDict(1)
                                    LoadGlobal(0)
                                    CallArgX(1)
```


### Constant parselet with constant-only arguments

```
a : @a:10 b:100 {
    a + b
}
```

In this case parameters 

```
a                                   CallStatic(0)               <parselet 0>

(a)                                 LoadStatic(0)               <parselet 0>

a()                                 CallStatic(0)               <parselet 0>

a(20)                               LoadStatic(1)               20
                                    CallStaticArg((0, 1))

a(b=200)                            LoadStatic(2)               200
                                    LoadStatic(3)               "b"
                                    MakeDict(1)
                                    CallStaticArgX((0, 0))

a(30, b=300)                        LoadStatic(4)               30
                                    LoadStatic(5)               300
                                    LoadStatic(3)               "b"
                                    MakeDict(1)
                                    CallStaticArgX((0, 1))
```
