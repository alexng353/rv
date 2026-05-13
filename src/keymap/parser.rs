use crate::keymap::{
    Key,
    key::Modifiers,
    tokenizer::{Token, TokenizerError, tokenize_chords},
};
use anyhow::bail;

use super::key::KeyCode;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid Keycode: '{0}'")]
    InvalidKeycode(String),

    #[error("Invalid Modifier: '{0}'")]
    InvalidModifier(char),

    #[error("Invalid Key Sequenece")]
    InvalidSequence(#[from] TokenizerError),
}

fn parse_sequence(sequence: &str) -> Result<Key, ParseError> {
    let (mods, rest) = peel_modifiers(sequence)?;
    let code = keycode_from_vim(&rest).ok_or(ParseError::InvalidKeycode(rest))?;
    Ok(Key { code, mods })
}

pub fn parse_chords(chord: &str) -> Result<Vec<Key>, ParseError> {
    let tokens = tokenize_chords(chord)?;

    let mut result: Vec<Key> = vec![];
    for token in tokens {
        match token {
            Token::Char(c) => result.push(Key {
                code: KeyCode::Char(c),
                mods: Modifiers::NONE,
            }),
            Token::Seq(sequence) => result.push(parse_sequence(&sequence)?),
        }
    }

    Ok(result)
}

#[cfg(test)]
mod chord_tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn simple() -> Result<()> {
        let result = parse_chords("<C-w>l")?;
        assert_eq!(
            result,
            vec![
                Key {
                    code: KeyCode::Char('w'),
                    mods: Modifiers::CONTROL
                },
                Key {
                    code: KeyCode::Char('l'),
                    mods: Modifiers::NONE
                }
            ]
        );
        Ok(())
    }

    #[test]
    fn single_bare_char() -> Result<()> {
        let result = parse_chords("a")?;
        assert_eq!(
            result,
            vec![Key {
                code: KeyCode::Char('a'),
                mods: Modifiers::NONE
            }]
        );
        Ok(())
    }

    #[test]
    fn two_bare_chars() -> Result<()> {
        let result = parse_chords("dd")?;
        assert_eq!(
            result,
            vec![
                Key {
                    code: KeyCode::Char('d'),
                    mods: Modifiers::NONE
                },
                Key {
                    code: KeyCode::Char('d'),
                    mods: Modifiers::NONE
                }
            ]
        );
        Ok(())
    }

    #[test]
    fn escape() -> Result<()> {
        let result = parse_chords("<Esc>")?;
        assert_eq!(
            result,
            vec![Key {
                code: KeyCode::Esc,
                mods: Modifiers::NONE
            }]
        );
        Ok(())
    }

    #[test]
    fn enter_alias_cr() -> Result<()> {
        let result = parse_chords("<CR>")?;
        assert_eq!(
            result,
            vec![Key {
                code: KeyCode::Enter,
                mods: Modifiers::NONE
            }]
        );
        Ok(())
    }

    #[test]
    fn shift_tab() -> Result<()> {
        let result = parse_chords("<S-Tab>")?;
        assert_eq!(
            result,
            vec![Key {
                code: KeyCode::Tab,
                mods: Modifiers::SHIFT
            }]
        );
        Ok(())
    }

    #[test]
    fn function_key() -> Result<()> {
        let result = parse_chords("<F1>")?;
        assert_eq!(
            result,
            vec![Key {
                code: KeyCode::F(1),
                mods: Modifiers::NONE
            }]
        );
        Ok(())
    }

    #[test]
    fn arrow_key() -> Result<()> {
        let result = parse_chords("<Left>")?;
        assert_eq!(
            result,
            vec![Key {
                code: KeyCode::Left,
                mods: Modifiers::NONE
            }]
        );
        Ok(())
    }

    #[test]
    fn space_special() -> Result<()> {
        let result = parse_chords("<Space>")?;
        assert_eq!(
            result,
            vec![Key {
                code: KeyCode::Char(' '),
                mods: Modifiers::NONE
            }]
        );
        Ok(())
    }

    #[test]
    fn mixed_special_and_bare() -> Result<()> {
        let result = parse_chords("<Esc>jk")?;
        assert_eq!(
            result,
            vec![
                Key {
                    code: KeyCode::Esc,
                    mods: Modifiers::NONE
                },
                Key {
                    code: KeyCode::Char('j'),
                    mods: Modifiers::NONE
                },
                Key {
                    code: KeyCode::Char('k'),
                    mods: Modifiers::NONE
                }
            ]
        );
        Ok(())
    }

    #[test]
    fn invalid_unclosed_bracket() {
        assert!(parse_chords("<C-w").is_err());
    }

    #[test]
    fn invalid_empty_brackets() {
        assert!(parse_chords("<>").is_err());
    }

    #[test]
    fn invalid_modifier() {
        assert!(parse_chords("<X-w>").is_err());
    }
}

// https://vimhelp.org/intro.txt.html#key-notation
fn keycode_from_vim(chord: &str) -> Option<KeyCode> {
    if chord.is_empty() {
        return None;
    }
    let first = chord.chars().next().unwrap();
    if chord.chars().count() == 1 {
        return Some(KeyCode::Char(first));
    }

    if first == 'F' {
        // cannot be None because this is caught in the chord.len() == 1 case
        let rest = chord.strip_prefix('F').unwrap();
        if let Ok(parsed) = rest.parse() {
            return Some(KeyCode::F(parsed));
        }
    }

    match chord {
        "Up" => Some(KeyCode::Up),
        "Down" => Some(KeyCode::Down),
        "Left" => Some(KeyCode::Left),
        "Right" => Some(KeyCode::Right),
        "Return" => Some(KeyCode::Enter),
        "CR" => Some(KeyCode::Enter),
        "Enter" => Some(KeyCode::Enter),
        "BS" => Some(KeyCode::Backspace),
        "Home" => Some(KeyCode::Home),
        "kHome" => Some(KeyCode::Home),
        "End" => Some(KeyCode::End),
        "kEnd" => Some(KeyCode::End),
        "PageUp" => Some(KeyCode::PageUp),
        "kPageUp" => Some(KeyCode::PageUp),
        "PageDown" => Some(KeyCode::PageDown),
        "kPageDown" => Some(KeyCode::PageDown),
        "Bar" => Some(KeyCode::Char('|')),
        "Del" => Some(KeyCode::Delete),
        "lt" => Some(KeyCode::Char('<')),
        "Space" => Some(KeyCode::Char(' ')),
        "Esc" => Some(KeyCode::Esc),
        "Tab" => Some(KeyCode::Tab),
        _ => None,
    }
}

fn is_modifier_char(c: char) -> bool {
    ['C', 'S', 'M', 'A', 'D', 'T'].contains(&c)
}

fn peel_modifiers(sequence: &str) -> Result<(Modifiers, String), ParseError> {
    let mut modifiers = Modifiers::NONE;

    let segments: Vec<_> = sequence.split('-').collect();

    let mut i = 0;

    while segments[i].len() == 1
        && is_modifier_char(segments[i].chars().next().unwrap())
        && i + 1 < segments.len()
    {
        let c = segments[i].chars().next().unwrap();

        let modifier = match c {
            'C' => Modifiers::CONTROL,
            'S' => Modifiers::SHIFT,
            'M' => Modifiers::META,
            'A' => Modifiers::META,
            'D' => Modifiers::SUPER,
            'T' => Modifiers::META,
            _ => return Err(ParseError::InvalidModifier(c)),
        };

        modifiers |= modifier;

        i += 1;
    }

    Ok((modifiers, segments[i..].join("-")))
}

#[cfg(test)]
mod peel_modifiers_test {
    use crate::keymap::key::Modifiers;

    use super::peel_modifiers;
    use anyhow::Result;
    #[test]
    fn regular_word() -> Result<()> {
        let (a, b) = peel_modifiers("sigma")?;

        assert_eq!(a, Modifiers::NONE);
        assert_eq!(b, "sigma");

        Ok(())
    }

    #[test]
    fn simple_control_seq() -> Result<()> {
        let (a, b) = peel_modifiers("C-w")?;
        assert_eq!(a, Modifiers::CONTROL);
        assert_eq!(b, "w");
        Ok(())
    }

    #[test]
    fn complex_control_seq() -> Result<()> {
        let (a, b) = peel_modifiers("C-S-A-w")?;
        assert_eq!(a, Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::META);
        assert_eq!(b, "w");
        Ok(())
    }
}
