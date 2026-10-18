# Operator + Motion Design

Design notes for vim-style operator+motion composition in rv. Not yet
implemented; this doc is the architecture spec for when we land it.

## The Big Idea

Vim has three orthogonal concepts that compose:

1. **Operators** — actions that consume a range (`d`, `c`, `y`, `>`, `<`, etc.).
2. **Motions** — produce a target position; with an operator, define a range
   from cursor to target (`w`, `$`, `gg`, etc.).
3. **Text objects** — describe a self-contained region around the cursor,
   independent of direction (`iw`, `a"`, `i{`, etc.).

The composition: **operator + (motion | text object) = action over range.**

- `dw` = delete + word-forward = "delete from cursor to start of next word."
- `ca"` = change + around-quotes = "delete quoted-string-with-quotes, then enter insert mode."
- `y2j` = yank + 2-lines-down = "copy cursor through 2 lines below, linewise."
- `>}` = indent + paragraph-forward = "indent from cursor to end of paragraph."

N operators × M motions × K text objects → N×(M+K) user-visible actions, but
only N+M+K things to implement. The matrix is emergent.

## The Operators

For v0 we ship `d`, `c`, `y`. Full vim set listed for completeness:

| Operator | Action |
|---|---|
| `d` | Delete (yank to register, remove from buffer) |
| `c` | Change (delete + enter Insert mode) |
| `y` | Yank (copy to register, no buffer mutation) |
| `>` | Indent right |
| `<` | Indent left |
| `=` | Auto-indent |
| `g~` | Toggle case |
| `gu` | Lowercase |
| `gU` | Uppercase |
| `!` | Pipe through external command |

## Motions Produce Targets, Operators Build Ranges

Architecturally, **motions stay pure-cursor-target**. They don't return
ranges. The Range is a derived concept that operator dispatch builds from
(current cursor, motion target, motion kind, motion inclusivity).

```rust
impl Motion {
    fn target(&self, cursor, buffer, mode) -> BufferCursor;  // already exists
    fn kind(&self) -> RangeKind;        // static — Charwise/Linewise/Blockwise
    fn inclusive(&self) -> bool;        // static — is end position included
}
```

- **Move-only callers** use `target()` and ignore the rest.
- **Operator callers** build the Range:

```rust
let from = window.cursor;
let to = motion.target(from, buffer, mode);
let range = Range { from, to, kind: motion.kind(), inclusive: motion.inclusive() };
apply_operator(op, range, buffer, register);
```

Range never appears in the motion return type. Motion signature stays
single-purpose. Operator dispatch site does the assembly.

### Static metadata per motion variant

| Motion | kind | inclusive |
|---|---|---|
| `Cursor(Right)` / `Cursor(Left)` | Charwise | false |
| `Cursor(Up)` / `Cursor(Down)` | Linewise | true |
| `Word { end: false }` (`w`/`b`/`W`/`B`) | Charwise | false |
| `Word { end: true }` (`e`/`ge`/`E`/`gE`) | Charwise | true |
| `LineZero` / `LineStart` (`0`/`^`) | Charwise | false |
| `LineEnd` (`$`) | Charwise | true |
| `FileTop` / `FileBottom` (`gg`/`G`) | Linewise | false |

These are constants per variant. `kind()` and `inclusive()` can be plain
`match self` returning literals.

## Range Type

```rust
struct Range {
    from: BufferCursor,
    to: BufferCursor,
    kind: RangeKind,
    inclusive: bool,
}

enum RangeKind {
    Charwise,   // exact char positions; partial lines allowed
    Linewise,   // operates on whole lines spanning from..to inclusive
    Blockwise,  // rectangular region (Ctrl-V visual; defer for v0)
}
```

- **Charwise** ranges respect column positions; operators delete/copy/etc.
  character-by-character from `from` to `to` (exclusive or inclusive of `to`
  based on `inclusive` flag).
- **Linewise** ranges snap to whole lines spanning `from.line..=to.line`.
  Column positions in `from`/`to` are ignored for the operator's effect.
- **Blockwise** is a v2 concern.

When `from > to` (backward motion), the operator should swap them. Conceptually
ranges don't have direction; the underlying positions can be in either order.

## Text Objects (Defer to v1)

Text objects produce `(start, end, kind, inclusive)` directly, without using
"current cursor as start." Different code path from motions:

```rust
impl TextObject {
    fn range(&self, cursor, buffer) -> Range;
}
```

`i` ("inner") excludes delimiters. `a` ("around") includes them. So `di"`
deletes the content inside quotes; `da"` deletes content + quotes.

Common text objects:
- `iw` / `aw` — word
- `is` / `as` — sentence
- `ip` / `ap` — paragraph
- `i"` / `a"` — quoted string
- `i(` / `a(` / `i{` / `a{` / `i[` / `a[` — bracket pairs
- `it` / `at` — HTML/XML tag

From the operator's perspective, motions and text objects are
interchangeable — both produce Range. Different code paths; same output type.

## Operators

```rust
enum Operator {
    Delete,
    Change,
    Yank,
}

fn apply_operator(op: Operator, range: Range, buffer: &mut Buffer, register: &mut Register) {
    let (start, end) = normalize(range.from, range.to);
    match (op, range.kind) {
        (Delete, Charwise) => { register.set(extract_chars(buffer, start, end, range.inclusive));
                                buffer.delete_chars(start, end, range.inclusive); }
        (Delete, Linewise) => { register.set(extract_lines(buffer, start.line, end.line));
                                buffer.delete_lines(start.line, end.line); }
        (Yank, _)          => { register.set(extract(buffer, range)); }
        (Change, _)        => { apply_operator(Delete, range, ...); editor.mode = Insert; }
        // ...
    }
}
```

`Change` is delete + enter insert mode. `Yank` is "delete but don't actually
delete." Lots of code sharing between operators.

## Dispatch State Machine

State on `Editor`:

```rust
struct Editor {
    pending_operator: Option<Operator>,  // None outside operator-pending
    register: Register,                  // unnamed register `"` for v0
    // ...
}
```

Dispatch flow (in `handle_key`):

```
TrieMatch::Word(Command::Operator(op)) =>
    pending_operator = Some(op)
    // (don't clear pending_keys yet; operator-pending state holds until motion arrives)

TrieMatch::Word(Command::Move(motion)) =>
    if let Some(op) = pending_operator.take() {
        // operator-pending: motion is operand
        let from = window.cursor;
        let to = motion.target(from, buffer, mode);
        let range = Range::new(from, to, motion.kind(), motion.inclusive());
        apply_operator(op, range, buffer, &mut register);
    } else {
        // plain move
        window.cursor = motion.target(window.cursor, buffer, mode);
    }
    adjust_scroll(...);
```

Esc in operator-pending state cancels: `pending_operator = None`, clear
pending keys, return to Normal mode.

## Operator Doubling

`dd`, `yy`, `cc` — same operator twice → linewise operation on current line.

Special-case in dispatch:

```
if let Some(op) = pending_operator {
    if new_key matches the operator's key {
        // doubled — linewise current line
        let range = Range {
            from: BufferCursor { line: window.cursor.line, col: 0 },
            to:   BufferCursor { line: window.cursor.line, col: 0 },
            kind: Linewise,
            inclusive: true,
        };
        apply_operator(op, range, ...);
        pending_operator = None;
        return;
    }
}
```

Each operator handler checks "was the second key me?" and handles linewise
inline. No special "operator is also a motion" abstraction needed.

## Counts (Defer or Land with Operators)

Counts can prefix operator, motion, or both:

- `3dw` = 3 × delete word = delete 3 words (count before operator)
- `d3w` = delete (3 × word) = delete 3 words (count before motion)
- `3d3w` = (3 × 3) words = delete 9 words (counts multiply)
- `3dd` = delete 3 lines

Implementation: digit accumulator in dispatch state.

```rust
struct Editor {
    pending_count: Option<u32>,
    pending_operator: Option<Operator>,
    // ...
}
```

Before any operator or motion dispatch, eat digit keys (`0-9`, but `0` alone is
the LineZero motion — so digits other than 0 start the count, then more digits
extend it). When operator/motion fires, apply the count.

For motions: motion runs N times (or computes the Nth occurrence). For
operator+motion: pre-op count × pre-motion count = total iterations.

Defer until d/y/c work with default count 1.

## Register

For v0, one register (the unnamed `"` register):

```rust
struct Register {
    content: String,
    kind: RangeKind,   // for paste — linewise pastes on new lines, charwise inline
}
```

`p`/`P` paste from register:
- Charwise paste: insert at cursor (`p` after current char, `P` before).
- Linewise paste: insert as new lines (`p` below current line, `P` above).

Register also captures `kind` from the source operation so paste can do the
right thing.

V1+: named registers (`"a`-`"z`), numbered registers (`"0`-`"9` — kill ring
style), system clipboard (`"+`, `"*`).

## Implementation Order

1. **Range struct + Motion metadata methods.** Add `kind()` and `inclusive()` to
   each Motion variant. No behavior change yet.
2. **Operator enum + pending_operator state + dispatch wiring.** Recognize
   `d`/`c`/`y` as operators in the keymap, set pending state, consume next motion.
3. **`apply_delete`** for charwise and linewise. Now `dw`, `dd`, `d$`, `dgg`,
   `dG` work.
4. **Register + `apply_yank` + `p`/`P` paste.** Now cut/copy/paste.
5. **`apply_change`** = delete + enter Insert. Now `cw`, `cc`, `c$` work.
6. **Counts.** Digit accumulator. Multiply pre-op and pre-motion counts.
7. **Text objects.** New range-producing path. `iw`/`aw` first, then quoted/
   bracketed pairs, then tag.
8. **Visual mode.** Different entry but reuses operator-on-range model.

Steps 1-5 give you the operator/motion core. That's the big payoff. Every
existing motion (and every motion added later) automatically becomes
`d{motion}`/`y{motion}`/`c{motion}`.

## How Neovim Does It (For Reference)

Neovim's implementation lives in `src/nvim/normal.c` and `src/nvim/ops.c`.
Core data structure is `oparg_T`:

```c
oparg_T {
    op_type;         // OP_DELETE / OP_YANK / OP_CHANGE / etc.
    motion_type;     // MCHAR / MLINE / MBLOCK
    inclusive;       // bool
    start, end;      // pos_T positions
    regname;         // register identifier
    motion_force;    // 'v'/'V'/Ctrl-V — force motion type
    // ...
}
```

Differences from our design:

- **Motions mutate cursor and set flags** rather than returning a target.
  C-era imperative pattern.
- **Range is implicit:** `start` saved before motion, `end` is current cursor
  after motion runs, `motion_type` and `inclusive` were set by the motion.
- **One shared `oparg_T` struct** read/written by all of dispatch, motions,
  text objects, and operator functions. Single source of truth, mutated
  throughout.

Rust idiom: pure motion + explicit Range built at operator dispatch site (our
design). Same architecture, less mutation.

Files worth reading in nvim source:
- `src/nvim/normal.c` — dispatch loop, motion functions.
- `src/nvim/ops.c` — operator implementations.
- `src/nvim/textobject.c` — text object implementations.
- `src/nvim/buffer_defs.h` — `oparg_T` definition.

## Niche: Motion Forcing

Vim has motion forcing via `v`/`V`/`Ctrl-V` after the operator: `dvj` forces
charwise even though `j` is normally linewise. `dV0` forces linewise even
though `0` is normally charwise.

Mechanism: a `motion_force` field that overrides `motion_type`. Defer to
v2+; almost no one uses this.

## The Insight That Makes This Click

Vim isn't "a text editor with a lot of commands." It's a **language**:

- Operators are verbs.
- Motions are nouns (or destinations).
- Text objects are noun phrases.
- Counts are quantifiers.

`daw` reads as "delete a word." `ci{` reads as "change inner brace block."
`3y2j` reads as "yank three groups of two lines down." Every editing action
is a small composable sentence.

Once internalized, the grammar is why vim users find it hard to go back to
modeless editing. The composability is the value.
