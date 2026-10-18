## the shape of an edit

Vim's sequences are shaped something like this:

```sh
[count] <command>
# or
["register] [count] <operator> [count] {text object | motion}
```

At some point, I do want to support the full extent of vim's editing
capabilities, but for the time being, I'll begin with this shape:

```
<operator> {text object | motion}
```

Because `[count]` is trivial\* to implement, and text objects/motions are the
most difficult things to implement. Motions, I've already got a pretty decent
design for how I want them to work. Text Objects, I have no idea how to do for
the time being.

---

Footnotes:

\*: nothing is trivial
