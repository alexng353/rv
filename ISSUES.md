todo:
- chords
- the rest of the currently existing commands

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
