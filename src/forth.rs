use std::{collections::HashMap, convert::TryFrom, iter};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ForthError {
    #[error("Division by zero!")]
    DivisionByZero,
    #[error("Empty stack!")]
    StackUnderflow,
    #[error("Unknown word: {0}")]
    UnknownWord(String),
    #[error("Invalid word: {0}")]
    InvalidWord(String),
    #[error("Unterminated input")]
    Unterminated,
    #[error("Bye")]
    UserQuit,
}

#[derive(Clone, Debug, PartialEq)]
struct Lexeme {
    value: String,
}

impl Lexeme {
    fn new(value: &str) -> Self {
        Lexeme {
            value: value.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Operator(ForthOperator),
    Builtin(ForthBuiltin),
    Word(String),
    Definition(Vec<Token>),
    UserDefined(Vec<Token>),
}

impl Token {
    pub fn eval(&self, state: &mut State) -> Result<Option<f64>, ForthError> {
        let result = match self {
            Token::Number(num) => Some(*num),
            Token::Operator(operator) => operator.eval(state)?,
            Token::Builtin(builtin) => builtin.eval(state)?,
            Token::Word(word) => match state.lookup(word) {
                Some(Token::Number(value)) => Some(value),
                Some(Token::Definition(user_defined_tokens)) => {
                    let mut result = None;
                    for token in user_defined_tokens {
                        result = token.eval(state)?;
                    }
                    result
                }
                Some(stored_token) => {
                    return Err(ForthError::InvalidWord(format!("{:?}", stored_token)));
                }
                None => {
                    return Err(ForthError::UnknownWord(word.clone()));
                }
            },
            Token::UserDefined(user_defined_tokens) => match user_defined_tokens.as_slice() {
                [Token::Word(name), rest @ ..] => {
                    state.define_word(name.clone(), Token::Definition(rest.to_vec()));
                    None
                }
                _ => {
                    return Err(ForthError::InvalidWord(format!(
                        "{:?}",
                        user_defined_tokens
                    )));
                }
            },
            Token::Definition(user_defined_tokens) => {
                return Err(ForthError::InvalidWord(format!(
                    "{:?}",
                    user_defined_tokens
                )));
            }
        };
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ForthOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl ForthOperator {
    pub fn eval(&self, state: &mut State) -> Result<Option<f64>, ForthError> {
        let result = match self {
            Self::Add => {
                let (op1, op2) = state.pop2()?;
                op2 + op1
            }
            Self::Subtract => {
                let (op1, op2) = state.pop2()?;
                op2 - op1
            }
            Self::Multiply => {
                let (op1, op2) = state.pop2()?;
                op2 * op1
            }
            Self::Divide => {
                let (op1, op2) = state.pop2()?;
                if op1 == 0 {
                    return Err(ForthError::DivisionByZero);
                }
                op2 / op1
            }
        };
        Ok(Some(result))
    }
}

impl TryFrom<&str> for ForthOperator {
    type Error = ForthError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Subtract),
            "*" => Ok(Self::Multiply),
            "/" => Ok(Self::Divide),
            v => Err(ForthError::UnknownWord(v.to_string())),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ForthBuiltin {
    Bye,
    Drop,
    Dup,
    Emit,
    Over,
    Show,
    Spaces,
    Swap,
    Take,
}

impl ForthBuiltin {
    pub fn eval(&self, state: &mut State) -> Result<Option<f64>, ForthError> {
        match self {
            Self::Bye => {
                return Err(ForthError::UserQuit);
            }
            Self::Drop => {
                state.pop()?;
            }
            Self::Dup => {
                let value = state.top()?;
                state.push(value);
            }
            Self::Emit => {
                let value = state.pop()?;
                print!("{}", value as u8 as char);
            }
            Self::Over => {
                let (num1, num2) = state.pop2()?;
                state.push(num2);
                state.push(num1);
                state.push(num2);
            }
            Self::Show => {
                state.show_stack();
            }
            Self::Spaces => {
                let num = state.pop()?;
                print!(
                    "{}",
                    iter::repeat(" ")
                        .take(num as usize)
                        .intersperse("")
                        .collect::<String>()
                );
            }
            Self::Swap => {
                let (value1, value2) = state.pop2()?;
                state.push(value1);
                state.push(value2);
            }
            Self::Take => {
                let value = state.pop()?;
                print!("{}", value);
            }
        }

        Ok(None)
    }
}

impl TryFrom<&str> for ForthBuiltin {
    type Error = ForthError;

    fn try_from(input: &str) -> Result<ForthBuiltin, Self::Error> {
        let builtin = match input {
            "bye" | "quit" => ForthBuiltin::Bye,
            "dup" => ForthBuiltin::Dup,
            "drop" => ForthBuiltin::Drop,
            "emit" => ForthBuiltin::Emit,
            "over" => ForthBuiltin::Over,
            ".s" => ForthBuiltin::Show,
            "spaces" => ForthBuiltin::Spaces,
            "swap" => ForthBuiltin::Swap,
            "." | "take" => ForthBuiltin::Take,
            //"toggle-debug" => ForthBuiltin::ToggleDebug,
            _ => {
                return Err(ForthError::UnknownWord(input.into()));
            }
        };
        Ok(builtin)
    }
}

#[derive(Debug)]
pub struct State {
    dictionary: HashMap<String, Token>,
    stack: Vec<f64>,
}

impl State {
    fn new() -> Self {
        Self {
            dictionary: HashMap::new(),
            stack: Vec::new(),
        }
    }

    fn define_word(&mut self, word: String, value: Token) {
        self.dictionary.insert(word, value);
    }

    fn lookup(&self, word: &str) -> Option<Token> {
        self.dictionary.get(word).map(|token| token.clone())
    }

    fn top(&self) -> Result<f64, ForthError> {
        match self.stack.last() {
            Some(value) => Ok(*value),
            None => Err(ForthError::StackUnderflow),
        }
    }

    fn push(&mut self, value: f64) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<f64, ForthError> {
        match self.stack.pop() {
            Some(num) => Ok(num),
            _ => Err(ForthError::StackUnderflow),
        }
    }

    fn pop2(&mut self) -> Result<(f64, f64), ForthError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(v1), Some(v2)) => Ok((v1, v2)),
            _ => Err(ForthError::StackUnderflow),
        }
    }

    fn show_stack(&self) {
        println!("{:?}", self.stack);
    }
}

#[derive(Debug)]
pub struct Forth {
    keep_running: bool,
    state: State,
}

impl Forth {
    pub fn new() -> Self {
        Self {
            keep_running: true,
            state: State::new(),
        }
    }

    pub fn eval(&mut self, input: &str) -> Result<Option<()>, ForthError> {
        let line = input.trim().to_string();
        if line.is_empty() {
            Ok(Some(()))
        } else {
            let lexemes = self.lex(&line)?;
            //println!("{:?} Ok", lexemes);
            let tokens = self.tokenize(&lexemes)?;
            //println!("{:?} Ok", tokens);
            let _result = self.run(&tokens)?;
            //println!("{:?} Ok", _result);
            if self.keep_running {
                Ok(Some(()))
            } else {
                Ok(None)
            }
        }
    }

    fn run(&mut self, tokens: &[Token]) -> Result<Option<f64>, ForthError> {
        let mut result = None;

        for token in tokens {
            result = token.eval(&mut self.state)?;
            if let Some(num) = result {
                self.state.push(num);
            }

            //self.state.show_stack();
        }

        Ok(result)
    }

    fn lex(&self, input: &str) -> Result<Vec<Lexeme>, ForthError> {
        let mut lexemes = Vec::new();
        for item in input.split(' ') {
            lexemes.push(Lexeme::new(&item.to_lowercase()));
        }

        Ok(lexemes)
    }

    fn tokenize(&self, input: &[Lexeme]) -> Result<Vec<Token>, ForthError> {
        let mut tokens = Vec::new();

        let mut user_defined = Vec::new();
        let mut in_user_defined = false;

        for item in input {
            if in_user_defined == false && !user_defined.is_empty() {
                return Err(ForthError::InvalidWord(item.value.clone()));
            }
            let token = if let Ok(value) = item.value.parse() {
                Token::Number(value)
            } else if let Ok(operator) = ForthOperator::try_from(item.value.as_ref()) {
                Token::Operator(operator)
            } else if let Ok(builtin) = ForthBuiltin::try_from(item.value.as_ref()) {
                Token::Builtin(builtin)
            } else if item.value == ":" {
                //println!("Start custom");
                in_user_defined = true;
                continue;
            } else if item.value == ";" {
                in_user_defined = false;
                //println!("Custom {:?}", user_defined);
                Token::UserDefined(user_defined.clone())
            } else {
                Token::Word(item.value.clone())
            };
            if in_user_defined {
                //println!("Pushing {:?}", token);
                user_defined.push(token);
            } else {
                tokens.push(token);
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cannot_parse_letter() {
        let mut forth = Forth::new();
        let result = forth.eval("1 a 3 4 5");
        assert_eq!(Err(ForthError::UnknownWord("a".to_string())), result);
    }

    #[test]
    fn parses_numbers() {
        let mut forth = Forth::new();
        let result = forth.eval("1 2.3 0.3 4 5");
        assert_eq!(Ok(Some(())), result);
    }

    #[test]
    fn parses_math_expressions() {
        let mut forth = Forth::new();
        let lexemes = forth.lex("1 2.3 + 0.3 * 4 / 5 -").unwrap();
        let result = forth.tokenize(&lexemes);
        assert_eq!(
            Ok(vec![
                Token::Number(1.0),
                Token::Number(2.3),
                Token::Operator(ForthOperator::Add),
                Token::Number(0.3),
                Token::Operator(ForthOperator::Multiply),
                Token::Number(4.0),
                Token::Operator(ForthOperator::Divide),
                Token::Number(5.0),
                Token::Operator(ForthOperator::Subtract)
            ]),
            result
        );
    }

    #[test]
    fn simple_addition_works() {
        let mut forth = Forth::new();
        let lexemes = forth.lex("5 6 +").unwrap();
        let tokens = forth.tokenize(&lexemes).unwrap();
        let result = forth.run(&tokens).unwrap();
        assert_eq!(Some(11.0), result);
    }
}
