# Operator-Pending State Machine

Design notes for how `dispatch` handles the multi-key sequences that operators
produce. Complements `docs/operators.md`, which covers the operator/motion
composition itself. This doc focuses on the *state machine* and *keymap
context* questions: what state does the editor hold between keypresses, and
how does the keymap know that `i` means "inner" instead of "enter insert
mode"?

## Why `Option<Operator>` Stops Working

For `dw` alone, `pending_operator: Option<Operator>` is enough — set on `d`,
consume on `w`. But as soon as you support more of Vim's grammar, the pending
state grows fields:

- a **count** (`3d`, `d3w`, `3d3w`)
- a **register** prefix (`"ad`, `"a3d3w`)
- the **operator** itself
- a **scope** modifier (`i` vs. `a`) once you reach text objects
- a "**waiting for object key**" flag after `di`

Stuffing all of that into parallel `Option<…>` fields on the editor produces
a soup of nonsense states (`scope = Inner` but `op = None`?
`waiting_for_object = true` but `scope = None`?). The fix is the standard
"make illegal states unrepresentable" move: collapse the phase information
into a single enum where each variant only carries the data meaningful in
that phase.

## The Three Phases

```rust
enum Pending {
    Idle,
    OperatorPending { op: Operator /*, count, register tacked on later */ },
    TextObjectPending { op: Operator, scope: Scope /*, count, register */ },
}

enum Scope { Inner, Around }
```

- **`Idle`** — nothing pending. Keys execute as standalone commands or
  motions.
- **`OperatorPending`** — an operator key has been typed. The next input is
  expected to be a motion, a text-object scope modifier (`i`/`a`), or the
  same operator again (doubling, e.g. `dd`).
- **`TextObjectPending`** — operator + scope modifier have been typed
  (e.g. `di`). The next input is expected to be a text-object key
  (`w`, `(`, `"`, …).

Count and register, when added, are decorations that ride along on the
enum's data — they don't add new variants. The phase count stays at three.

## The Grammar

The operator-mode subset of Vim's command grammar is:

```
(register?) (count?) operator (motion | text-object | operator)
```

Where `text-object` is itself `(i|a) object_key` at the keystroke level.

Things the grammar deliberately doesn't cover:

- **Pure motions** (`j`, `w`, `gg`) — these have no operator; they execute
  immediately from `Idle`.
- **Standalone commands** (`x`, `p`, `u`) — same.
- **Motion forcing** (`dvj`, `dV0`) — niche, deferred.

So `dispatch` has two top-level paths:

1. `Idle` + key → interpret as standalone command/motion, execute, stay
   `Idle`.
2. `OperatorPending`/`TextObjectPending` + key → interpret per the grammar
   above; either transition or execute-and-return-to-`Idle`.

The grammar is the contract for path (2) only.

## Transition Table

| From | Key | To | Effect |
|---|---|---|---|
| `Idle` | operator | `OperatorPending { op }` | — |
| `Idle` | motion | `Idle` | move cursor |
| `Idle` | standalone | `Idle` | execute command |
| `OperatorPending { op }` | motion | `Idle` | apply `op` over motion range |
| `OperatorPending { op }` | same op (e.g. `dd`) | `Idle` | apply `op` linewise on current line |
| `OperatorPending { op }` | `i` | `TextObjectPending { op, Inner }` | — |
| `OperatorPending { op }` | `a` | `TextObjectPending { op, Around }` | — |
| `OperatorPending { _ }` | anything else | `Idle` | cancel (Vim beeps; we silently drop) |
| `TextObjectPending { op, scope }` | object key | `Idle` | resolve object → range → apply `op` |
| `TextObjectPending { _, _ }` | anything else | `Idle` | cancel |

Invariant to assert in tests: every key event leaves `pending` in exactly
one of the three phases. There is no "halfway between phases."

## The Keymap Question

`i` and `a` mean different things in different phases:

- In `Idle`, `i` enters Insert mode and `a` enters Append.
- In `OperatorPending`, `i` and `a` are scope modifiers — they should *not*
  enter Insert/Append.

You cannot reconcile this in a single keymap without overloading commands
with context-dependent meanings. Vim's answer is that **Operator-pending is
a real mode** with its own keymap — that's what `:omap` exists for.

Design options, weighed:

1. **Operator-pending as a `Mode` variant.** Add `Mode::OperatorPending`
   alongside `Normal`/`Insert`/`Command`. Its keymap *inherits* motions from
   Normal (so `w`, `b`, `f`, `0`, `$` keep working without duplication) and
   *overrides* `i`/`a` to set scope. This is what Vim does. Clean
   separation; the keymap doesn't need to know about editor state.

2. **Sub-state checked at keymap-lookup time.** Keep `pending_operator` on
   the editor and have the keymap consult it during lookup. Same result, but
   the keymap layer now leaks knowledge of editor state. Avoid.

3. **Reinterpret in `dispatch`.** The Normal keymap returns
   `Command::EnterInsert`, and `dispatch` notices "oh, there's a pending op,
   so this actually means `Scope::Inner`." Cute, but it means every
   command's semantics depend on hidden state — fragile, and the overload
   list will grow.

**Go with #1.** OperatorPending is a real mode; its keymap inherits motions
from Normal and overrides `i`/`a`.

### TextObjectPending is *not* a third mode

The valid keys after `di` are a small fixed set of object identifiers
(`w`, `W`, `s`, `p`, `(`, `[`, `{`, `<`, `"`, `'`, `` ` ``, `t`, `f`, …).
These are not motions, and they only have meaning in this exact context.
Vim doesn't expose this as a user-configurable mode — there's no
`:tomap`. Hardcode the object lookup in the `TextObjectPending` arm of
dispatch. If a real need for configurability appears, promote it then.

### Keymap inheritance strategy

"OperatorPending inherits from Normal" is conceptually clean but the
implementation has to pick one of:

- **Reference / linked map.** OperatorPending keymap holds a pointer to
  Normal; lookup tries OperatorPending first, falls through to Normal.
  User-defined op-pending bindings naturally override inherited ones.
  Closest to what Vim does.
- **Copy at construction.** Walk Normal's trie into OperatorPending's at
  startup, then layer overrides on top. Simpler lookup (one trie), but
  changes to Normal at runtime don't propagate.
- **Two independent maps.** No inheritance; redefine everything. Don't —
  motion duplication will rot.

Pick the fall-through approach. Implementation cost is small and the
semantics match user intuition.

## Cancellation Policy

Anything that doesn't fit the grammar from `OperatorPending` or
`TextObjectPending` drops back to `Idle`. Vim beeps and discards; we just
discard. No partial execution, no fall-through to "interpret the key as a
fresh command." If someone types `di<bogus>`, the operator is consumed and
the bogus key is dropped on the floor.

(`Esc` from any pending phase is the same: cancel, return to `Idle`.)

## What `dispatch` Becomes

Once the table above is real, `dispatch` is essentially a `match` on
`self.pending`, with each arm being a transcription of a row block from
the table. The current `if let Some(op) = self.pending_operator.take()`
structure in `editor.rs` is the seed of this; it just needs to fan out into
phase-aware variants and stop carrying mid-dispatch fallthrough.

The lingering `self.pending_operator = None;` after a `.take()` (currently
at `editor.rs:111`) is dead either way and can go.

## Out of Scope For This Doc

- The operator/motion/range model itself — see `docs/operators.md`.
- Counts and registers — deferred until d/y/c work with default count 1
  (see `docs/operators.md` §Counts and §Register).
- Visual mode — different entry path but reuses the operator-on-range
  machinery; not a phase of this state machine.
