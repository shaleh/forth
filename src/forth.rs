use std::{convert::TryFrom, iter};

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Token {
    Number(f64),
    Operator(ForthOperator),
    Builtin(ForthBuiltin),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ForthOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

fn pop1(stack: &mut Vec<f64>) -> Result<f64, ForthError> {
    match stack.pop() {
        Some(num) => Ok(num),
        _ => Err(ForthError::StackUnderflow),
    }
}

fn pop2(stack: &mut Vec<f64>) -> Result<(f64, f64), ForthError> {
    match (stack.pop(), stack.pop()) {
        (Some(v1), Some(v2)) => Ok((v1, v2)),
        _ => Err(ForthError::StackUnderflow),
    }
}

impl ForthOperator {
    pub fn eval(&self, stack: &mut Vec<f64>) -> Result<Option<f64>, ForthError> {
        let result = match self {
            Self::Add => {
                let (op1, op2) = pop2(stack)?;
                op2 + op1
            }
            Self::Subtract => {
                let (op1, op2) = pop2(stack)?;
                op2 - op1
            }
            Self::Multiply => {
                let (op1, op2) = pop2(stack)?;
                op2 * op1
            }
            Self::Divide => {
                let (op1, op2) = pop2(stack)?;
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
    pub fn eval(&self, stack: &mut Vec<f64>) -> Result<Option<f64>, ForthError> {
        match self {
            Self::Bye => {
                return Err(ForthError::UserQuit);
            }
            Self::Drop => {
                pop1(stack)?;
            }
            Self::Dup => {
                let value = pop1(stack)?;
                stack.push(value);
            }
            Self::Emit => {
                let value = pop1(stack)?;
                print!("{}", value as u8 as char);
            }
            Self::Over => {
                let (num1, num2) = pop2(stack)?;
                stack.push(num2);
                stack.push(num1);
                stack.push(num2);
            }
            Self::Show => {
                show_stack(stack);
            }
            Self::Spaces => {
                let num = pop1(stack)?;
                println!(
                    "{}",
                    iter::repeat(" ")
                        .take(num as usize)
                        .intersperse("")
                        .collect::<String>()
                );
            }
            Self::Swap => {
                let (value1, value2) = pop2(stack)?;
                stack.push(value1);
                stack.push(value2);
            }
            Self::Take => {
                let value = pop1(stack)?;
                println!("{}", value);
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

fn show_stack(stack: &Vec<f64>) {
    println!("{:?}", stack);
}

#[derive(Debug)]
pub struct Forth {
    keep_running: bool,
    stack: Vec<f64>,
}

impl Forth {
    pub fn new() -> Self {
        Self {
            keep_running: true,
            stack: Vec::new(),
        }
    }

    pub fn eval(&mut self, input: &str) -> Result<Option<()>, ForthError> {
        let line = input.trim().to_string();
        if line.is_empty() {
            Ok(Some(()))
        } else {
            let lexemes = self.lex(&line)?;
            println!("{:?} Ok", lexemes);
            let tokens = self.tokenize(&lexemes)?;
            println!("{:?} Ok", tokens);
            let result = self.run(&tokens)?;
            println!("{:?} Ok", result);
            if self.keep_running {
                Ok(Some(()))
            } else {
                Ok(None)
            }
        }
    }

    fn run(&mut self, tokens: &[Token]) -> Result<Option<f64>, ForthError> {
        for token in tokens {
            let result = match token {
                Token::Number(num) => Some(*num),
                Token::Operator(operator) => operator.eval(&mut self.stack)?,
                Token::Builtin(builtin) => builtin.eval(&mut self.stack)?,
            };
            if let Some(num) = result {
                self.stack.push(num);
            }

            println!("Stack: {:?}", self.stack);
        }

        match self.stack.last() {
            Some(num) => Ok(Some(*num)),
            None => Ok(None),
        }
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
        for item in input {
            if let Ok(value) = item.value.parse() {
                tokens.push(Token::Number(value));
            } else if let Ok(operator) = ForthOperator::try_from(item.value.as_ref()) {
                tokens.push(Token::Operator(operator));
            } else if let Ok(builtin) = ForthBuiltin::try_from(item.value.as_ref()) {
                tokens.push(Token::Builtin(builtin));
            } else {
                return Err(ForthError::UnknownWord(item.value.clone()));
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
