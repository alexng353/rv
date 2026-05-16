todo:
- chords
    - operators (d, c, y, etc)
    - motions --- refactor to produce a range (PURE) rather than do an action
    - text objects
    - count + operator + (motion | text object)
- motions as ranges
    - right now, motions return a target cursor position (for movements)
    - need to change this to return a range (start, end, metadata)
        - Charwise
        - Linewise
        - Blockwise (defer)
    - inclusive | exclusive
- text objects
    - iw, aw, etc. (inner/around word)
- operator doubling (operate over the current line)
    - dd - delete current line
    - yy - yank current line
    - cc - change current line
- counts (defer, trivial)
- visual mode
- new state machine dispatch changes:
    -   Normal
    press operator (d/c/y)
      → OperatorPending(operator)
          press motion (w/e/$/gg/...)
              → compute range → apply operator → Normal
          press text object (iw/a"/...)
              → compute range → apply operator → Normal
          press the same operator (dd/cc/yy)
              → linewise current line → apply operator → Normal
          press another operator
              → cancel; treat first as no-op (or some other semantics)
          press Esc
              → cancel → Normal
- right side of chin bar - current position in buffer - scroll position
  indicator
```
In Neovim's statusline, All is the scroll position indicator — it tells you what
portion of the buffer is currently visible on screen.

The possible values:
- All — the entire buffer fits on screen (no scrolling needed)
- Top — viewport is at the top of the buffer
- Bot — viewport is at the bottom of the buffer
- NN% — percentage showing how far down the buffer the viewport is (e.g. 45%)
```

done: 
- the rest of the currently existing commands
- cursor doesn't respect buffer chin bar
- need to be able to navigate between splits
- leave an extra row at the bottom of the screen for mode indicator + command
  buffer
- split rendering doesn't respect cursor bounding
- pressing `:` in normal mode when the last line is not empty does not wipe the
  line
- editing the code causes flickering
- show some kind of difference between the two splits, perchance a chin bar
  would be a good idea (per buf)
