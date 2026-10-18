# MVP

What we need for a minimally viable editor:

open file -> move around -> insert/delete text -> save -> quit

This requires:

1. Rope data structure
2. MVP core (open file, save file, insert and delete text, mode)
3. MVP motions (move around, just h/j/k/l)
4. MVP Events (motions)
5. MVP TUI (No actual TUI, just the shape, plus rendering a single window)
6. Entrypoint (event loop)
