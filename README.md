_WARNING: This repo is in active development._

# `boba-script`

boba-script is a general purpose interpreted programming language intended to be used with another one of my projects boba-engine. While it does not contain any code that references the engine, its internals are built in a way that allows it to be transformed into a domain specific language for that purpose. This means that it does not have to be a DSL for boba-engine specifically, but it can be used as a general purpose language or transformed into a DSL for another purpose.

## Try it out!

The best way to try out the current version of boba-script is to use the interpreter.

To run the interpreter, first clone the repo.

```shell
git clone https://github.com/rhedgeco/boba-script.git
```

Then navigate into the interpreter folder.

```shell
cd boba-script/interpreter
```

Then run the code!

```shell
cargo run --release
```

## Features

The current feature set is incredibly limited and will be developed over time.

### Values

There are a few different value types in bobs-script

- None Type - e.g. `none`

  - boba-script does not have nulls, and instead an explicit none type can be used to represent nothing.

- Int Type

  - int types are any whole number. e.g. `5`, `42`
  - integer types have no bound in the size of number they can hold

- Float Type

  - float types are any decimal number
  - these can be constructed by their full representation like `12.5`  
    a trailing decimal notation like `14.`  
    or a trailing 'f' notation like this `20f`
  - internally the float uses 64 bit precision

- Bool Type

  - the bool type can be one of two values. `true` or `false`

- String Type

  - string types are a buffer of characters
  - they can be constructed with single quotes `'hello world'`  
    or double quotes `"hello world"`

- Tuple Type
  - tuples are a grouped collection of items
  - they can be created as a comma seperated list surrounded by braces like this `(5, 'hello', 8.9)`

### Variables

To create a variable use the 'let' syntax and assign a value

```
let my_var = 5
```

Variables can also be assigned to after they are initialized like this

```
my_var = 20
```

Assignments can destructure their items on the right hand side

```
let (a, b, c) = (10, 42, 750)
```

This also means that swapping variable vales does not need a temp variable

```
let a = 10
let b = 42
(a, b) = (b, a) # this line swaps the values
```

### While Loops

While loops may be constructed like this

```
while my_var > 0:
  # do something here
```

### Functions

Functions may be constructed like this

```
fn my_func(value):
  # do something here
```

Functions can be called like this

```
my_func('hello world')
```
