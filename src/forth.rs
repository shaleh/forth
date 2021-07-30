use std::convert::TryFrom;

#[derive(Debug)]
pub struct Forth {}

#[derive(Clone, Debug)]
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

#[derive(Clone, Copy, Debug)]
enum Token {
    Number(f64),
    Operator(ForthOperator),
}

#[derive(Clone, Copy, Debug)]
enum ForthOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl TryFrom<&str> for ForthOperator {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Subtract),
            "*" => Ok(Self::Multiply),
            "/" => Ok(Self::Divide),
            v => Err(format!("Invalid operator: {}", v)),
        }
    }
}

impl Forth {
    pub fn new() -> Self {
        Self {}
    }

    fn debug_state(&self) {}

    pub fn eval(&self, input: &str) -> Result<Option<()>, String> {
        let line = input.trim().to_string();
        if line.is_empty() {
            Ok(Some(()))
        } else if line.to_lowercase() == "bye" || line.to_lowercase() == "quit" {
            self.debug_state();
            Ok(None)
        } else {
            let lexemes = self.lex(&line)?;
            println!("{:?} Ok", lexemes);
            let tokens = self.tokenize(&lexemes);
            println!("{:?} Ok", tokens);
            self.debug_state();
            Ok(Some(()))
        }
    }

    fn lex(&self, input: &str) -> Result<Vec<Lexeme>, String> {
        let mut lexemes = Vec::new();
        for item in input.split(' ') {
            lexemes.push(Lexeme::new(item));
        }

        Ok(lexemes)
    }

    fn tokenize(&self, input: &[Lexeme]) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        for item in input {
            if let Ok(value) = item.value.parse() {
                tokens.push(Token::Number(value));
            } else if let Ok(operator) = ForthOperator::try_from(item.value.as_ref()) {
                tokens.push(Token::Operator(operator));
            } else {
                return Err(format!("not a number: {}", item.value));
            }
        }

        Ok(tokens)
    }
}
