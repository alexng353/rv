# Architecture

rv is split into multiple crates to enforce better API design and strict
non-recursive dependencies. It's also split into multiple crates so I can learn
how large projects in rust are actually built and designed well.

Crates:

<!-- TODO: make this into a table -->
- `rv-core` - the editor itself
- `rv-rope` - rope data structure, not optimized well. realistically, should be
  using ropey
- `rv-script` - scripting language, basically the shape of how we're going to
  declare keymaps and stuff like vimscript or lua but actually good
- `rv-tui` - reusable screen drawing stuff
