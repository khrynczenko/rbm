# rbm

`rbm` is a WIP compiler for the B-Minor language. It contains hand written
lexer and recursive descent parser. In the grammar.md file one can find
the grammar used by the `rbm`.

`rbm` is a project of mine that I used to learn about both compiler construction
and rust language. It is not something you should use professionally.

## What is B-Minor

B-Minor is a C like language. It was introduced in the "Introduction to
Compilers and Language Design" by Douglas Thain. You can think of it as less
capable C. That means it doesn't have pointers, structures or unions. B-Minor
is statically and strongly typed language. You can find more infromation about
it in [the book](https://www3.nd.edu/~dthain/compilerbook/).

## Running rbm

To perform lexical analysis on some source file run

```text
rbm lex <path_to_source_file>
```

To perform syntactic analysis and obtain AST as a result run

```text
rbm parse <path_to_source_file>
```

For more details run

```text
> rbm --help
```

## WIP

`rbm` is work-in-progress. Current state is as follows:

- [x] lexical analysis
- [x] syntactic analysis
- [x] AST generation
- [] semantic analysis
- [] optimization
- [] code generation
