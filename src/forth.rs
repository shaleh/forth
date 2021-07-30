use std::convert::TryFrom;

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
}

#[derive(Debug)]
pub struct Forth {}

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
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ForthOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
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

impl Forth {
    pub fn new() -> Self {
        Self {}
    }

    fn debug_state(&self) {}

    pub fn eval(&self, input: &str) -> Result<Option<()>, ForthError> {
        let line = input.trim().to_string();
        if line.is_empty() {
            Ok(Some(()))
        } else if line.to_lowercase() == "bye" || line.to_lowercase() == "quit" {
            self.debug_state();
            Ok(None)
        } else {
            let lexemes = self.lex(&line)?;
            println!("{:?} Ok", lexemes);
            let tokens = self.tokenize(&lexemes)?;
            println!("{:?} Ok", tokens);
            let result = self.run(&tokens)?;
            println!("{:?} Ok", result);
            self.debug_state();
            Ok(Some(()))
        }
    }

    fn run(&self, tokens: &[Token]) -> Result<Option<f64>, ForthError> {
        let mut stack = Vec::new();
        for token in tokens {
            let result = match token {
                Token::Number(num) => Some(*num),
                Token::Operator(operator) => operator.eval(&mut stack)?,
            };
            if let Some(num) = result {
                stack.push(num);
            }

            println!("Stack: {:?}", &stack);
        }

        match stack.last() {
            Some(num) => Ok(Some(*num)),
            None => Err(ForthError::StackUnderflow),
        }
    }

    fn lex(&self, input: &str) -> Result<Vec<Lexeme>, ForthError> {
        let mut lexemes = Vec::new();
        for item in input.split(' ') {
            lexemes.push(Lexeme::new(item));
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
