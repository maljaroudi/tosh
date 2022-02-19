use logos::Logos;
#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[token("|")]
    Pipe,

    // Or regular expressions.
    #[regex(r"\$*\([^)]*\)")]
    Parens,
    #[regex("[a-zA-Z]+")]
    Text,
    #[error]
    // We can also use this variant to define whitespace,
    // or any other matches we wish to skip.
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use super::Token;

    #[test]
    fn lex_pipe_test() {
        let mut lex = Token::lexer("$(ridiculouslyfastLexers) Hello");
        assert_eq!(lex.next(), Some(Token::Parens));
        assert_eq!("$(ridiculouslyfastLexers)", lex.slice());
        assert_eq!(lex.next(), Some(Token::Text));
    }
}
