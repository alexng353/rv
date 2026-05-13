use bitflags::bitflags;
use crossterm::event::{KeyCode as CKeyCode, KeyEvent as CKeyEvent, KeyModifiers as CKeyModifiers};

bitflags! {
    /// Represents key modifiers (shift, control, alt, etc.).
    ///
    /// **Note:** `SUPER`, `HYPER`, and `META` can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct Modifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const SUPER = 0b0000_1000;
        const HYPER = 0b0001_0000;
        const META = 0b0010_0000;
        const NONE = 0b0000_0000;
    }
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum KeyCode {
    /// Backspace key (Delete on macOS, Backspace on other platforms).
    Backspace,
    /// Enter key.
    Enter,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page up key.
    PageUp,
    /// Page down key.
    PageDown,
    /// Tab key.
    Tab,
    /// Shift + Tab key.
    BackTab,
    /// Delete key. (Fn+Delete on macOS, Delete on other platforms)
    Delete,
    /// Insert key.
    Insert,
    /// F key.
    ///
    /// `KeyCode::F(1)` represents F1 key, etc.
    F(u8),
    /// A character.
    ///
    /// `KeyCode::Char('c')` represents `c` character, etc.
    Char(char),
    /// Null.
    Null,
    /// Escape key.
    Esc,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Key {
    pub code: KeyCode,
    pub mods: Modifiers,
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Bare char with no modifiers prints as-is, except for chars that
        // would be ambiguous in vim notation (space, <, |, \).
        if self.mods.is_empty() {
            if let KeyCode::Char(c) = self.code {
                return match c {
                    ' ' => write!(f, "<Space>"),
                    '<' => write!(f, "<lt>"),
                    '|' => write!(f, "<Bar>"),
                    '\\' => write!(f, "<Bslash>"),
                    _ => write!(f, "{}", c),
                };
            }
        }

        f.write_str("<")?;

        if self.mods.contains(Modifiers::CONTROL) {
            f.write_str("C-")?;
        }
        if self.mods.contains(Modifiers::SHIFT) {
            f.write_str("S-")?;
        }
        if self.mods.contains(Modifiers::ALT) {
            f.write_str("A-")?;
        }
        if self.mods.contains(Modifiers::META) {
            f.write_str("M-")?;
        }
        if self.mods.contains(Modifiers::SUPER) {
            f.write_str("D-")?;
        }
        if self.mods.contains(Modifiers::HYPER) {
            f.write_str("H-")?;
        }

        match self.code {
            KeyCode::Backspace => f.write_str("BS")?,
            KeyCode::Enter => f.write_str("CR")?,
            KeyCode::Left => f.write_str("Left")?,
            KeyCode::Right => f.write_str("Right")?,
            KeyCode::Up => f.write_str("Up")?,
            KeyCode::Down => f.write_str("Down")?,
            KeyCode::Home => f.write_str("Home")?,
            KeyCode::End => f.write_str("End")?,
            KeyCode::PageUp => f.write_str("PageUp")?,
            KeyCode::PageDown => f.write_str("PageDown")?,
            KeyCode::Tab => f.write_str("Tab")?,
            KeyCode::BackTab => f.write_str("S-Tab")?,
            KeyCode::Delete => f.write_str("Del")?,
            KeyCode::Insert => f.write_str("Insert")?,
            KeyCode::F(n) => write!(f, "F{}", n)?,
            KeyCode::Null => f.write_str("Null")?,
            KeyCode::Esc => f.write_str("Esc")?,
            KeyCode::Char(c) => match c {
                ' ' => f.write_str("Space")?,
                '<' => f.write_str("lt")?,
                '|' => f.write_str("Bar")?,
                '\\' => f.write_str("Bslash")?,
                _ => write!(f, "{}", c)?,
            },
        }

        f.write_str(">")
    }
}

/// Wrapper for displaying a slice of keys as a vim-notation sequence.
/// Use as `format!("{}", KeySeq(&keys))` to get e.g. `<C-w>hjk`.
pub struct KeySeq<'a>(pub &'a [Key]);

impl std::fmt::Display for KeySeq<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for k in self.0 {
            write!(f, "{}", k)?;
        }
        Ok(())
    }
}

impl TryFrom<CKeyEvent> for Key {
    type Error = anyhow::Error;

    fn try_from(value: CKeyEvent) -> std::result::Result<Self, Self::Error> {
        let code = match value.code {
            CKeyCode::Backspace => KeyCode::Backspace,
            CKeyCode::Enter => KeyCode::Enter,
            CKeyCode::Left => KeyCode::Left,
            CKeyCode::Right => KeyCode::Right,
            CKeyCode::Up => KeyCode::Up,
            CKeyCode::Down => KeyCode::Down,
            CKeyCode::Home => KeyCode::Home,
            CKeyCode::End => KeyCode::End,
            CKeyCode::PageUp => KeyCode::PageUp,
            CKeyCode::PageDown => KeyCode::PageDown,
            CKeyCode::Tab => KeyCode::Tab,
            CKeyCode::BackTab => KeyCode::BackTab,
            CKeyCode::Delete => KeyCode::Delete,
            CKeyCode::Insert => KeyCode::Insert,
            CKeyCode::F(u) => KeyCode::F(u),
            CKeyCode::Char(char) => KeyCode::Char(char),
            CKeyCode::Null => KeyCode::Null,
            CKeyCode::Esc => KeyCode::Esc,
            _ => anyhow::bail!("Illegal"),
        };

        let mut mods: u8 = 0;

        if value.modifiers.contains(CKeyModifiers::SHIFT) {
            mods |= Modifiers::SHIFT.bits();
        }
        if value.modifiers.contains(CKeyModifiers::CONTROL) {
            mods |= Modifiers::CONTROL.bits();
        }
        if value.modifiers.contains(CKeyModifiers::ALT) {
            mods |= Modifiers::ALT.bits();
        }
        if value.modifiers.contains(CKeyModifiers::SUPER) {
            mods |= Modifiers::SUPER.bits();
        }
        if value.modifiers.contains(CKeyModifiers::HYPER) {
            mods |= Modifiers::HYPER.bits();
        }
        if value.modifiers.contains(CKeyModifiers::META) {
            mods |= Modifiers::META.bits();
        }
        if value.modifiers.contains(CKeyModifiers::NONE) {
            mods |= Modifiers::NONE.bits();
        }

        Ok(Key {
            code,
            mods: Modifiers::from_bits_truncate(mods),
        })
    }
}
