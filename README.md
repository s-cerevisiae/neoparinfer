# Neoparinfer (WIP)

A new protocol for inferring parentheses and communicating the results,
currently incomplete (not quite usable yet).

TODO:
- [x] [Paren mode](./src/parens_only.rs)
- [x] Indent mode (needs more testing)
- [ ] Lexer module for a few languages and interface design
- [ ] Smart mode
- [ ] Clean up the repo so it looks less like a laboratory disaster
- [ ] Multi-line tokens
- [ ] Unicode width

## Idea

Currently most implementations of parinfer do the IO naively. They take in the
whole buffer and emits the whole processed text. And this leads to two
problems:

- Editor plugins have to copy the whole buffer back and forth on each key
  stroke, which can hardly be efficient.
- It need to be fully aware of the syntax of every Lisp dialect to handle the
  syntactic subtleties like char literals and comments. This both complicates
  the core implementation, and makes it buggy.

Neoparinfer changes the representations of input and output to improve the
situation.

The core of the parinfer algorithm, as explained by its author, only need to consume

- The position of first non-blank character of each line (indent size)
- The position of each parenthesis and the exact character
- A map of pairing characters

and output a series of editing commands that

- either change the indentation of a line (paren mode)
- or insert/delete parentheses at the end of a line (indent mode)

By using a representation that only contains the necessary info, the core can
be both simpler and more efficient. It also opens up possibility for modular
and correct support for different languages:

### Language support

Languages should have their own (simplified) lexers to determine which parens
are part of lists. The core of Neoparinfer only cares about parens and
indentation, no config toggles or global variables should be required to get
correct behavior.

Example: the long standing issue of Racket "s-exp comments" can be solved in
this way
```racket
#\( ;; don't send to neoparinfer
;; ( don't send to neoparinfer
#;() ;; DO send to neoparinfer
```

### Multi-line tokens (in progress)

Multi-line tokens can be seen as a special case of tokens that "can end at a
column earlier than its start"; and where it ends the tokens following it can
be treated as if the lines are concatenated. 

```scheme
;;           v logical start
(define x (f "a
x
x
b" "c")
;^ logical end
  "g h i")
;; ^ indentation rules work as normal
;;   as if it's a single line
```

The lines spanned by these tokens should not be touched by neoparinfer, so
there might be a need for "logical line" mappings that converts between actual
text lines and the lines neoparinfer operates on.

TODO: A representation of "logical lines" in the core, or find a way to eliminate it.

### Locality and incremental changes (unverified)

I *think* the effect of the parinfer algorithm is local on each editing action
that changes part of the text. For a piece of text that previously already
satisfy parinfer rules, an edit that results in no invalid tokens (e.g.
unclosed strings) should only affect the "containing indented block" of all
text it changed.

```scheme
(define-language Let
  (terminals        ;; the change at most propagates to here
    (constant (c))
    (variable (v)))
  ;; ...
  )
```

For example if I modify the `variable` line in the snippet above without
dedenting it further left than the `terminals` block, it should not affect any
code outside `terminals`.

If true, this can be used to further strip down the input and make it
incremental based on a diff with the previous version, or a span of changed
lines and their current indentation.

## License

The project is distributed under the terms of [Apache-2.0 License](./LICENSE).

<!-- vi:spell: -->
