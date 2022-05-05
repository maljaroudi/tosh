pub use logos::Logos;
#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[token("|")]
    Pipe,
    #[token("$")]
    Dollar,
    // Or regular expressions.
    #[token("(")]
    ParensOpen,
    #[token(")")]
    ParensClose,
    #[regex("[a-zA-Z.]+")]
    Text,
    #[regex("[dlLsStT+@rwx-][dlLsStT+@rwx-][dlLsStT+@rwx-][dlLsStT+@rwx-][dlLsStT+@rwx-][dlLsStT+@rwx-][dlLsStT+@rwx-][dlLsStT+@rwx-][dlLsStT+@rwx-][dlLsStT+@rwx-]")]
    Permissions,
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
    fn lex_paren_test() {
        let mut lex = Token::lexer("Hello $(HELLO)");
        assert_eq!(lex.next(), Some(Token::Text));

        assert_eq!(lex.next(), Some(Token::Dollar));
        assert_eq!(lex.next(), Some(Token::ParensOpen));
        assert_eq!(lex.next(), Some(Token::Text));
        assert_eq!(lex.next(), Some(Token::ParensClose));
    }

    #[test]
    fn lex_pipe_test() {
        let mut lex = Token::lexer("echo yes | apt install test");
        assert_eq!(lex.next(), Some(Token::Text));
        assert_eq!(lex.next(), Some(Token::Text));
        assert_eq!(lex.next(), Some(Token::Pipe));
    }
    #[test]
    fn lex_output_test_permissions() {
        let mut lex = Token::lexer("drwxr-xr-x");
        assert_eq!(lex.next(), Some(Token::Permissions));
    }
    #[test]
    fn lex_filename_test() {
        let mut lex = Token::lexer("hello.txt");
        assert_eq!(lex.next(), Some(Token::Text));
    }

    #[test]
    fn lex_ls_test() {
        let prelex = "drwxr-xr-x 2 moheeb moheeb_users    4 Mar 17 15:38 .\ndrwxr-xr-x 3 moheeb moheeb_users    4 Mar 17 14:49 ..\n-rw-r--r-- 1 moheeb moheeb_users  201 Mar 17 14:49 jsonify.rs\n-rw-r--r-- 1 moheeb moheeb_users 1383 Mar 17 15:25 lexer.rs\n-rw-r--r-- 1 moheeb moheeb_users  907 Mar 17 13:55 lib.rs\n-rw-r--r-- 1 moheeb moheeb_users 3041 Mar  9 09:15 trie.rs";
        for lines in prelex.lines() {
            let mut lex = Token::lexer(lines);
            assert_eq!(lex.next(), Some(Token::Permissions));
        }
    }
}
