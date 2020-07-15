# Tokay Programming Language

This document is a scribble and draft for the syntax and semantics for Tokay, a new programming language for text-processing, influenced by AWK, Python and Rust.

## Hello World

```
print("Hello World")                        # function form
print "Hello World"                         # awk-style statement form (only for some functions!)
```

## Simple values

```
i = 42                                      # Integer (i64, bigint)
b = true                                    # boolean (bool)
s = "Tokay"                                 # String (String)
f = 1.337                                   # Float (f64)
v = void                                    # Defines "set to nothing"
u = unset                                   # Defines "not even void"
```

## Complex values

The complex object is a combination of a list and an ordered map.

```
c = [1, 2, 3]                               # List
c = [2 = "a", 1 = "b"]                      # Ordered map (order 2, 1 is kept)
c = [                                       # Mixed list and map with different keys
    "Hello" = 42,
    1337 = "World",
    23                                      # this is even with unset=23
]
```

## Expressions

Simple expressions

```
i
i + 1
i = j + k * (3 - z)
i++
--i
i += 42
(x)
```

Control-flow expressions are also part of expressions.

- `if... else ...` - conditional
- `loop ...` - endless loop
- `while ...` - head-controlled loop
- `do...while...` - foot-controlled loop
- `for...` - C-style iterator loop 
- `for...in...` - Rust-style iterator loop

Returned is either the last value of the provided block, or an explicit value when `break` or `continue` keywords are used.

```
# non-blocked in-line form
d = if i > 10 42 else -42

# blocked form
d = if i > 10 {
    print true
    42
} else {
    print false
    -42
}

# a loop
x = while d > 0 {
    if --d == 1337
        break "big"                         # return "big" to x
    else if d > 42
        break "okay"                        # return "okay" to x
}                                           #otherwise, returns just unset to x
```

## Tokens

In short, tokens are anything that is taken from the input, altought some tokens may not consume any input. A token in terms of Tokay is an entity, which matches something from the input, and returns a result. A result is always returned, and is either a value or a capture, which could be turned later into a value (when required).

```
|A-Za-z0-9_|                                # Character class
'Hello'                                     # Touch (match without a capture)
"World"                                     # Match
```

Tokens are used to make calls to Parselets.
(like EOF or None).

```
Integer                                     # Call parselet
None                                        # The empty word
EOF                                         # Only true when at end-of-file.
```

By design, regular expressions shall not be part of the language. Regular expressions are oftenly heavy to read and can be expressed in Tokay itself more verbosely.

## Modifiers

Tokens can be configured for repetition or optional using standard regex-style modifiers.

```
|A-Z|+
"hello"*
Negate? Expression
```

## Patterns

In short, every line in Tokay is a pattern, but expressions or statements can be just interpreted as `None expression`.

```
'Hello' "World"+                            # Touch "Hello", multiple match "World"
Integer print                               # Match parselet named Integer, on success print $0
'x'     a++                                 # Increment a when 'x' is touched in input.
```

## Blocks

Blocks are used to define areas with subsequent patterns or statements.

```
"Hello" {                                   # first matches "Hello"...
    "World"+                                # ...then matches "HelloWorld", "HelloWorldWorld"...
    "Universe"                              # ...otherwise, matches "HelloUniverse"
}

res = if x < 100                            # when x < 100...
{                                
    {
        "alpha"+                            # when multiple tokens of "alpha" are in the input..
        "delta" {                           # or when delta is in the input...
            "golf"                          # ...followed by "golf" or
            "echo"                          # ..."echo"
        } print                             # then print $0, returns unset to res
    }
}
else
{
    "bravo" ("charlie")                     # else match "bravo" and return "charlie" to res
}
```

## Repeatable blocks

Tokays fundamental building block is the repeatable block ```@{...}```, which is also the way functions are being represented. A repeatable block runs like a loop over the input, when it is accordingly. When a repeatable block occurs as a token without being assigned to a variable, it is immediatelly called.

This block has the following features:

- Any captures relate to a repeatable block
- The block is repeatable, it behaves like a loop over the input
- The keywords `accept`, `reject`, `repeat` and `next` are always bound to this scope
- The main program is an implicit repeatable block
- Repeatable blocks can be called with parameters

The repeatable block represents the scope of a function or parselet.

```
"hello" @{"world" repeat}                   # Matches hello world+
"world" @{"hello" repeat}?                  # Matches world hello*
```

Because variables in a repeatable block are being replaced when the block restarts, variables need to be defined on 

```
x = 1
@{
    "inc X"    x++ repeat   # consume "inc X", increment x and repeat
    .          repeat       # consume anything else 
}
```

## Constants

Constants are identifiers beginning with an upper-case first letter `A-Z` or `_`.

When a repeatable block is assigned to a constant, it is considered to be a static function.


- mostly used for parselets, but can also be used any other unchangeable variables.
- Constants can be defined as function or parselet parameters, therefore they're scopable same as variables.
- parselets being created from other parselets, which are even constants, can be left-recursive parsing functions

## Variables

Variables are identifiers beginning with a lower-case first letter.

## Parselets

Parselets are similar to parsing functions in a recursive-descent parser. They relate to a non-terminal in a grammar.

## Functions

Functions 

### Simple parselets

```
Hellos = @{ "hello"+ }                      # Define parselet Hellos with one pattern matching arbitrary "hello"

Factor = @{                                   # define parselet Factor with
    Float                                   # ... either a pattern with a Float
    '(' Factor ')'                          # ... or a pattern calling a bracketed Factor recursively
}
```

### Generic parselets

A parselet is generic, when it involves other constant parselets.

```
Repeat = @(Token) {

}
```


### Whitespace and Lexemes

Whitespace is handled by a special parselet called _. It defaults to a parselet matching standard whitespace like space, tab, newline. The _ parselet is automatically inserted as _? between tokens in a pattern.

```
_ = @{ | \t\r\n\v| repeat }                   # Default
```

In the Factor example from above, the explicit version is

```
Factor = @{                                   # define parselet Factor with
    Float _                                 # ... either a pattern with a Float
    '(' _ Factor _ ')'                      # ... or a pattern calling a bracketed Factor recursively
}
```

There can be other Parselets starting with _ which are threatened as lexemes, they don't automatically 



## Control flow

## The golden rules

1. Constant
2. variable
3. @{repeating}

