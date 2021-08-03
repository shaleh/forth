use std::io::{self, Write};
use std::{collections::HashMap, convert::TryFrom, iter};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ForthError {
    #[error("Division by zero!")]
    DivisionByZero,
    #[error("Stack underflow!")]
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
enum Token {
    Number(f64),
    Builtin(ForthBuiltin),
    Word(String),
    Definition(Vec<Token>),
    UserDefined(Vec<Token>),
}

impl Token {
    pub fn eval(&self, state: &mut State) -> Result<Option<f64>, ForthError> {
        let result = match self {
            Token::Number(num) => Some(*num),
            Token::Builtin(builtin) => builtin.eval(state)?,
            Token::Word(word) => self.eval_word(state, word)?,
            Token::UserDefined(user_defined_tokens) => {
                self.eval_user_defined(state, user_defined_tokens)?
            }
            Token::Definition(user_defined_tokens) => {
                self.eval_definition(state, user_defined_tokens)?
            }
        };
        Ok(result)
    }

    fn eval_word(&self, state: &mut State, word: &str) -> Result<Option<f64>, ForthError> {
        match state.lookup(word) {
            Some(Token::Number(value)) => Ok(Some(value)),
            Some(Token::Definition(user_defined_tokens)) => {
                self.eval_definition(state, user_defined_tokens.as_slice())
            }
            Some(stored_token) => Err(ForthError::InvalidWord(format!("{:?}", stored_token))),
            None => {
                let parsed = self.parse_word(word.as_ref())?;
                parsed.eval(state)
            }
        }
    }

    fn parse_word(&self, word: &str) -> Result<Token, ForthError> {
        if let Ok(builtin) = ForthBuiltin::try_from(word.to_lowercase().as_ref()) {
            Ok(Token::Builtin(builtin))
        } else {
            Err(ForthError::UnknownWord(word.to_string()))
        }
    }

    fn lookup_definition(&self, state: &State, token: Token) -> Result<Token, ForthError> {
        let definition = match token {
            Token::Word(word) => match state.lookup(&word) {
                Some(value) => value,
                None => self.parse_word(word.as_ref())?,
            },
            _ => token,
        };
        Ok(definition)
    }

    fn eval_definition(
        &self,
        state: &mut State,
        tokens: &[Token],
    ) -> Result<Option<f64>, ForthError> {
        for token in tokens {
            if let Some(value) = token.eval(state)? {
                state.push(value);
            }
        }
        Ok(None)
    }

    fn eval_user_defined(
        &self,
        state: &mut State,
        tokens: &[Token],
    ) -> Result<Option<f64>, ForthError> {
        match tokens {
            [Token::Word(name), rest @ ..] => match rest
                .iter()
                .map(|token| self.lookup_definition(state, token.clone()))
                .collect()
            {
                Ok(collected_tokens) => {
                    state.define_word(name.clone(), Token::Definition(collected_tokens));
                    Ok(None)
                }
                Err(err) => Err(err),
            },
            _ => Err(ForthError::InvalidWord(format!("{:?}", tokens))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ForthBuiltin {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Bye,
    CR,
    Display,
    Drop,
    Dup,
    TwoDrop,
    TwoDup,
    Emit,
    Mod,
    SlashMod,
    Over,
    TwoOver,
    Rot,
    Show,
    Space,
    Spaces,
    Swap,
    TwoSwap,
}

impl ForthBuiltin {
    pub fn eval(&self, state: &mut State) -> Result<Option<f64>, ForthError> {
        match self {
            // (n1 n2 -- sum)
            Self::Add => {
                let (n2, n1) = state.pop2()?;
                state.push(n1 + n2);
            }
            Self::Subtract => {
                // (n1 n2 -- difference)
                let (n2, n1) = state.pop2()?;
                state.push(n1 - n2);
            }
            Self::Multiply => {
                // (n1 n2 -- result)
                let (n2, n1) = state.pop2()?;
                state.push(n1 * n2);
            }
            Self::Divide => {
                // (n1 n2 -- result)
                let (n2, n1) = state.pop2()?;
                if n2 == 0.0 {
                    return Err(ForthError::DivisionByZero);
                }
                state.push(n1 / n2);
            }
            Self::Mod => {
                // (n1 n2 -- rem)
                let (n2, n1) = state.pop2()?;
                if n2 == 0.0 {
                    return Err(ForthError::DivisionByZero);
                }
                state.push(n1 % n2);
            }
            Self::SlashMod => {
                // (n1 n2 -- rem quot)
                let (n2, n1) = state.pop2()?;
                if n2 == 0.0 {
                    return Err(ForthError::DivisionByZero);
                }
                state.push(n1 % n2);
                state.push(n1 / n2);
            }
            Self::Bye => {
                return Err(ForthError::UserQuit);
            }
            Self::CR => {
                println!();
            }
            Self::Display => {
                // (n1 -- )
                let value = state.pop()?;
                print!("{}", value);
            }
            Self::Drop => {
                // (n1 n2 -- n1)
                state.pop()?;
            }
            Self::TwoDrop => {
                // (d1 d2 -- d1)
                // IOW (n1 n2 n3 n4 -- n1 n2)
                state.pop2()?;
            }
            Self::Dup => {
                // (n -- n n)
                let n = state.top()?;
                state.push(n);
            }
            Self::TwoDup => {
                // (d -- d d)
                // IOW (n1 n2 -- n1 n2 n1 n2)
                let (n2, n1) = state.pop2()?;
                state.push(n1);
                state.push(n2);
                state.push(n1);
                state.push(n2);
            }
            Self::Emit => {
                // (n1 -- )
                let value = state.pop()?;
                print!("{}", value as u8 as char);
            }
            Self::Over => {
                // (n1 n2 -- n1 n2 n1)
                let (num2, num1) = state.pop2()?;
                state.push(num1);
                state.push(num2);
                state.push(num1);
            }
            Self::TwoOver => {
                // (d1 d2 -- d1 d2 d1)
                // IOW (n1 n2 n3 n4 -- n1 n2 n3 n4 n1 n2)
                let (num4, num3) = state.pop2()?;
                let (num2, num1) = state.pop2()?;
                state.push(num1);
                state.push(num2);
                state.push(num3);
                state.push(num4);
                state.push(num1);
                state.push(num2);
            }
            Self::Rot => {
                // (n1 n2 n3 -- n2 n3 n1)
                let (num3, num2) = state.pop2()?;
                let num1 = state.pop()?;
                state.push(num2);
                state.push(num3);
                state.push(num1);
            }
            Self::Show => {
                state.show_stack();
            }
            Self::Space => {
                print!(" ");
            }
            Self::Spaces => {
                // (n1 -- )
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
                // (n1 n2 -- n2 n1)
                let (n2, n1) = state.pop2()?;
                state.push(n2);
                state.push(n1);
            }
            Self::TwoSwap => {
                // (d1 d2 -- d2 d1)
                // IOW (n1 n2 n3 n4 -- n3 n4 n1 n2)
                let (n4, n3) = state.pop2()?;
                let (n2, n1) = state.pop2()?;
                state.push(n3);
                state.push(n4);
                state.push(n1);
                state.push(n2);
            }
        }

        Ok(None)
    }
}

impl TryFrom<&str> for ForthBuiltin {
    type Error = ForthError;

    fn try_from(input: &str) -> Result<ForthBuiltin, Self::Error> {
        let builtin = match input {
            "." => ForthBuiltin::Display,
            "+" => ForthBuiltin::Add,
            "-" => ForthBuiltin::Subtract,
            "*" => ForthBuiltin::Multiply,
            "/" => ForthBuiltin::Divide,
            "bye" | "quit" => ForthBuiltin::Bye,
            "cr" => ForthBuiltin::CR,
            "dup" => ForthBuiltin::Dup,
            "2dup" => ForthBuiltin::TwoDup,
            "drop" => ForthBuiltin::Drop,
            "2drop" => ForthBuiltin::TwoDrop,
            "emit" => ForthBuiltin::Emit,
            "/mod" => ForthBuiltin::SlashMod,
            "mod" => ForthBuiltin::Mod,
            "over" => ForthBuiltin::Over,
            "2over" => ForthBuiltin::TwoOver,
            "rot" => ForthBuiltin::Rot,
            ".s" => ForthBuiltin::Show,
            "space" => ForthBuiltin::Space,
            "spaces" => ForthBuiltin::Spaces,
            "swap" => ForthBuiltin::Swap,
            "2swap" => ForthBuiltin::TwoSwap,
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
        self.dictionary.insert(word.to_lowercase(), value);
    }

    fn lookup(&self, word: &str) -> Option<Token> {
        self.dictionary.get(&word.to_lowercase()).cloned()
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
            None => Err(ForthError::StackUnderflow),
        }
    }

    fn pop2(&mut self) -> Result<(f64, f64), ForthError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(v1), Some(v2)) => Ok((v1, v2)),
            _ => Err(ForthError::StackUnderflow),
        }
    }

    fn show_stack(&self) {
        print!("<{}> ", self.stack.len());
        for item in &self.stack {
            print!("{} ", item);
        }
        io::stdout().flush().unwrap();
    }
}

#[derive(Debug)]
pub struct Forth {
    state: State,
}

impl Forth {
    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    pub fn prompt(&self) -> String {
        "> ".to_string()
    }

    #[cfg(test)]
    pub fn stack(&self) -> &[f64] {
        &self.state.stack
    }

    pub fn eval(&mut self, input: &str) -> Result<Option<f64>, ForthError> {
        let line = input.trim().to_string();
        if line.is_empty() {
            Ok(None)
        } else {
            let lexemes = self.lex(&line)?;
            let tokens = self.tokenize(&lexemes)?;
            let result = self.run(&tokens)?;

            Ok(result)
        }
    }

    fn run(&mut self, tokens: &[Token]) -> Result<Option<f64>, ForthError> {
        let mut result = None;

        for token in tokens {
            result = token.eval(&mut self.state)?;
            if let Some(num) = result {
                self.state.push(num);
            }
        }

        Ok(result)
    }

    fn lex(&self, input: &str) -> Result<Vec<String>, ForthError> {
        Ok(input.split(' ').map(|s| s.to_string()).collect())
    }

    fn tokenize(&self, input: &[String]) -> Result<Vec<Token>, ForthError> {
        let mut tokens = Vec::new();

        let mut user_defined = Vec::new();
        let mut in_user_defined = false;

        for item in input {
            if item == ":" {
                in_user_defined = true;
                continue;
            }

            let token = if let Ok(value) = item.parse() {
                Token::Number(value)
            } else if item == ";" {
                in_user_defined = false;
                Token::UserDefined(user_defined.clone())
            } else {
                Token::Word(item.clone())
            };
            if in_user_defined {
                user_defined.push(token);
            } else {
                tokens.push(token);
            }
            if !in_user_defined && !user_defined.is_empty() {
                user_defined.clear();
            }
        }

        if in_user_defined {
            Err(ForthError::Unterminated)
        } else {
            Ok(tokens)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cannot_parse_letter() {
        let mut forth = Forth::new();
        assert_eq!(
            forth.eval("1 a 3 4 5"),
            Err(ForthError::UnknownWord("a".to_string()))
        );
    }

    #[test]
    fn parses_numbers() {
        let mut forth = Forth::new();
        assert_eq!(forth.eval("1 2.3 0.3 4 5"), Ok(Some(5.0)));
    }

    #[test]
    fn parses_math_expressions() {
        let forth = Forth::new();
        let lexemes = forth.lex("1 2.3 + 0.3 * 4 / 5 -").unwrap();
        let result = forth.tokenize(&lexemes);
        assert_eq!(
            Ok(vec![
                Token::Number(1.0),
                Token::Number(2.3),
                Token::Word("+".to_string()),
                Token::Number(0.3),
                Token::Word("*".to_string()),
                Token::Number(4.0),
                Token::Word("/".to_string()),
                Token::Number(5.0),
                Token::Word("-".to_string()),
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
        assert_eq!(None, result);
    }

    #[test]
    fn dup() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 dup"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 1.0],);
    }

    #[test]
    fn dup_top_value_only() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 dup"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 2.0]);
    }

    #[test]
    fn dup_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 DUP Dup dup"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn dup_error() {
        let mut f = Forth::new();
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("dup"));
    }

    #[test]
    fn two_dup() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 2dup"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 1.0, 2.0]);
    }

    #[test]
    fn two_dup_top_pair_only() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 2dup"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 3.0, 2.0, 3.0]);
    }

    #[test]
    fn two_dup_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 2DUP 2Dup 2dup"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0]);
    }

    #[test]
    fn two_dup_error() {
        let mut f = Forth::new();
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("2dup"));
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("1 2dup"));
    }

    #[test]
    fn rot() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 rot"), Ok(None));
        assert_eq!(f.stack(), vec![2.0, 3.0, 1.0]);
    }

    #[test]
    fn rot_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 ROT Rot rot"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn rot_error() {
        let mut f = Forth::new();
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("rot"));
    }

    #[test]
    fn drop() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 drop"), Ok(None));
        assert_eq!(Vec::<f64>::new(), f.stack());
    }

    #[test]
    fn drop_with_two() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 drop"), Ok(None));
        assert_eq!(f.stack(), vec![1.0]);
    }

    #[test]
    fn drop_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 4 DROP Drop drop"), Ok(None));
        assert_eq!(f.stack(), vec![1.0]);
    }

    #[test]
    fn drop_error() {
        let mut f = Forth::new();
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("drop"));
    }

    #[test]
    fn swap() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 swap"), Ok(None));
        assert_eq!(f.stack(), vec![2.0, 1.0]);
    }

    #[test]
    fn swap_with_three() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 swap"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 3.0, 2.0]);
    }

    #[test]
    fn swap_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 SWAP 3 Swap 4 swap"), Ok(None));
        assert_eq!(f.stack(), vec![2.0, 3.0, 4.0, 1.0]);
    }

    #[test]
    fn swap_error() {
        let mut f = Forth::new();
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("1 swap"));
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("swap"));
    }

    #[test]
    fn two_swap() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 4 2swap"), Ok(None));
        assert_eq!(f.stack(), vec![3.0, 4.0, 1.0, 2.0]);
    }

    #[test]
    fn two_swap_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 4 2SWAP 2Swap 2swap"), Ok(None));
        assert_eq!(f.stack(), vec![3.0, 4.0, 1.0, 2.0]);
    }

    #[test]
    fn two_swap_error() {
        let mut f = Forth::new();
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("1 2swap"));
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("2swap"));
    }

    #[test]
    fn over() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 over"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 1.0]);
    }

    #[test]
    fn over_with_three() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 over"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 3.0, 2.0]);
    }

    #[test]
    fn over_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 OVER Over over"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 1.0, 2.0, 1.0]);
    }

    #[test]
    fn over_error() {
        let mut f = Forth::new();
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("1 over"));
        assert_eq!(Err(ForthError::StackUnderflow), f.eval("over"));
    }

    #[test]
    fn two_over() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 4 2over"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 3.0, 4.0, 1.0, 2.0]);
    }

    #[test]
    fn two_over_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 3 4 2OVER 2Over 2over"), Ok(None));
        assert_eq!(
            f.stack(),
            vec![1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0, 1.0, 2.0]
        );
    }

    #[test]
    fn two_over_error() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2over"), Err(ForthError::StackUnderflow));
        assert_eq!(f.eval("1 2 2over"), Err(ForthError::StackUnderflow));
        assert_eq!(f.eval("1 2 3 2over"), Err(ForthError::StackUnderflow));
    }

    // User-defined words

    #[test]
    fn can_consist_of_built_in_words() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": dup-twice dup dup ;"), Ok(None));
        assert_eq!(f.eval("1 dup-twice"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 1.0, 1.0]);
    }

    #[test]
    fn execute_in_the_right_order() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": countup 1 2 3 ;"), Ok(None));
        assert_eq!(f.eval("countup"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn redefining_an_existing_word() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": foo dup ;"), Ok(None));
        assert_eq!(f.eval(": foo dup dup ;"), Ok(None));
        assert_eq!(f.eval("1 foo"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 1.0, 1.0]);
    }

    #[test]
    fn redefining_an_existing_built_in_word() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": swap dup ;"), Ok(None));
        assert_eq!(f.eval("1 swap"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 1.0]);
    }

    #[test]
    fn user_defined_words_are_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": foo dup ;"), Ok(None));
        assert_eq!(f.eval("1 FOO Foo foo"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn definitions_are_case_insensitive() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": SWAP DUP Dup dup ;"), Ok(None));
        assert_eq!(f.eval("1 swap"), Ok(None));
        assert_eq!(f.stack(), vec![1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn redefining_a_built_in_operator() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": + * ;"), Ok(None));
        assert_eq!(f.eval("3 4 +"), Ok(None));
        assert_eq!(f.stack(), vec![12.0]);
    }

    #[test]
    fn can_define_variable() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": foo 5 ;"), Ok(None));
        assert_eq!(f.eval("foo"), Ok(None));
        assert_eq!(f.stack(), vec![5.0]);
    }

    #[test]
    fn can_use_different_words_with_the_same_name() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": foo 5 ;"), Ok(None));
        assert_eq!(f.eval(": bar foo ;"), Ok(None));
        assert_eq!(f.eval(": foo 6 ;"), Ok(None));
        assert_eq!(f.eval("bar foo"), Ok(None));
        assert_eq!(f.stack(), vec![5.0, 6.0]);
    }

    #[test]
    fn can_define_word_that_uses_word_with_the_same_name() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": foo 10 ;"), Ok(None));
        assert_eq!(f.eval(": foo foo 1 + ;"), Ok(None));
        assert_eq!(f.eval("foo"), Ok(None));
        assert_eq!(f.stack(), vec![11.0]);
    }

    #[test]
    fn defining_a_number() {
        let mut f = Forth::new();
        let result = f.eval(": 1 2 ;");
        assert!(matches!(result, Err(ForthError::InvalidWord(_))));
    }

    #[test]
    fn malformed_word_definition() {
        let mut f = Forth::new();
        assert_eq!(Err(ForthError::Unterminated), f.eval(":"));
        assert_eq!(Err(ForthError::Unterminated), f.eval(": foo"));
        assert_eq!(Err(ForthError::Unterminated), f.eval(": foo 1"));
    }

    #[test]
    fn calling_non_existing_word() {
        let mut f = Forth::new();
        assert_eq!(
            Err(ForthError::UnknownWord("foo".to_string())),
            f.eval("1 foo")
        );
    }

    #[test]
    fn multiple_definitions() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": one 1 ; : two 2 ; one two +"), Ok(None));
        assert_eq!(f.stack(), vec![3.0]);
    }

    #[test]
    fn definitions_after_ops() {
        let mut f = Forth::new();
        assert_eq!(f.eval("1 2 + : addone 1 + ; addone"), Ok(None));
        assert_eq!(f.stack(), vec![4.0]);
    }

    #[test]
    fn redefine_an_existing_word_with_another_existing_word() {
        let mut f = Forth::new();
        assert_eq!(f.eval(": foo 5 ;"), Ok(None));
        assert_eq!(f.eval(": bar foo ;"), Ok(None));
        assert_eq!(f.eval(": foo 6 ;"), Ok(None));
        assert_eq!(f.eval(": bar foo ;"), Ok(None));
        assert_eq!(f.eval("bar foo"), Ok(None));
        assert_eq!(f.stack(), vec![6.0, 6.0]);
    }
}
