use anyhow::bail;

#[derive(PartialEq)]
enum State {
    Bare,
    InSequence(String),
}

pub enum Token {
    Char(char),
    Seq(String),
}

#[derive(Debug, thiserror::Error)]
pub enum TokenizerError {
    #[error("Invalid unclosed sequence")]
    InvalidUnclosedSequence,

    #[error("Invalid nested <")]
    InvalidNestedLt,
}

pub fn tokenize_chords(chord: &str) -> Result<Vec<Token>, TokenizerError> {
    let mut result: Vec<Token> = vec![];
    let mut state = State::Bare;

    for c in chord.chars() {
        match (&mut state, c) {
            (State::Bare, '<') => state = State::InSequence(String::new()),
            (State::Bare, c) => result.push(Token::Char(c)),
            (State::InSequence(buf), '>') => {
                result.push(Token::Seq(std::mem::take(buf)));
                state = State::Bare;
            }
            (State::InSequence(_), '<') => return Err(TokenizerError::InvalidNestedLt),
            (State::InSequence(buf), c) => buf.push(c),
        }
    }

    if state != State::Bare {
        return Err(TokenizerError::InvalidUnclosedSequence);
    }

    Ok(result)
}
