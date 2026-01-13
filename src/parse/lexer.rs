//use crate::parse::BadSyntaxException;
use std::collections::HashSet;

/// A lexical analyzer for SQL
pub struct Lexer {
    s: String,
    pos: usize,
    keywords: HashSet<String>,
}

impl Lexer {
    pub fn new(s: &str) -> Self {
        let mut keywords = HashSet::new();
        for kw in &[
            "select", "from", "where", "and", "insert", "into", "values", "delete", "update",
            "set", "create", "table", "int", "varchar", "view", "as", "index", "on",
        ] {
            keywords.insert(kw.to_string());
        }
        let mut lexer = Lexer {
            s: s.to_string(),
            pos: 0,
            keywords,
        };
        lexer.skip_whitespace();
        lexer
    }

    pub fn eat_id(&mut self) -> String {
        self.skip_whitespace();
        if !self.match_id() {
            panic!("Syntax error: expected identifier");
        }
        let start = self.pos;
        while self.pos < self.s.len() {
            let c = self.s.chars().nth(self.pos).unwrap();
            if self.is_id_char(c) {
                self.pos += 1;
            } else {
                break;
            }
        }
        let id = self.s[start..self.pos].to_lowercase();
        self.skip_whitespace();
        id
    }

    pub fn eat_int_constant(&mut self) -> i32 {
        self.skip_whitespace();
        if !self.match_int_constant() {
            panic!("Syntax error: expected integer constant");
        }
        let start = self.pos;
        while self.pos < self.s.len() {
            let c = self.s.chars().nth(self.pos).unwrap();
            if c.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str = &self.s[start..self.pos];
        let num = num_str
            .parse()
            .unwrap_or_else(|_| panic!("Invalid integer: {}", num_str));
        self.skip_whitespace();
        num
    }

    pub fn eat_string_constant(&mut self) -> String {
        self.skip_whitespace();
        if !self.match_string_constant() {
            panic!("Syntax error: expected string constant");
        }
        self.pos += 1; // skip opening quote
        let start = self.pos;
        while self.pos < self.s.len() {
            let c = self.s.chars().nth(self.pos).unwrap();
            if c == '\'' {
                break;
            }
            self.pos += 1;
        }
        let result = self.s[start..self.pos].to_string();
        self.pos += 1; // skip closing quote
        self.skip_whitespace();
        result
    }

    pub fn eat_keyword(&mut self, w: &str) {
        self.skip_whitespace();
        if !self.match_keyword(w) {
            panic!("Syntax error: expected keyword {}", w);
        }
        while self.pos < self.s.len() {
            let c = self.s.chars().nth(self.pos).unwrap();
            if c.is_ascii_alphanumeric() || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.skip_whitespace();
    }

    pub fn eat_delim(&mut self, d: char) {
        self.skip_whitespace();
        if self.pos >= self.s.len() || self.s.chars().nth(self.pos).unwrap() != d {
            panic!("Syntax error: expected delimiter {}", d);
        }
        self.pos += 1;
        self.skip_whitespace();
    }

    pub fn match_id(&self) -> bool {
        if self.pos >= self.s.len() {
            return false;
        }
        let c = self.s.chars().nth(self.pos).unwrap();
        c.is_ascii_alphabetic() && !self.is_keyword_at_pos()
    }

    pub fn match_int_constant(&self) -> bool {
        if self.pos >= self.s.len() {
            return false;
        }
        self.s.chars().nth(self.pos).unwrap().is_ascii_digit()
    }

    pub fn match_string_constant(&self) -> bool {
        if self.pos >= self.s.len() {
            return false;
        }
        self.s.chars().nth(self.pos).unwrap() == '\''
    }

    pub fn match_keyword(&self, w: &str) -> bool {
        if self.pos >= self.s.len() {
            return false;
        }
        let start = self.pos;
        let mut pos = self.pos;
        while pos < self.s.len() {
            let c = self.s.chars().nth(pos).unwrap();
            if c.is_ascii_alphanumeric() || c == '_' {
                pos += 1;
            } else {
                break;
            }
        }
        if pos > start {
            let word = self.s[start..pos].to_lowercase();
            word == w.to_lowercase() && self.keywords.contains(&word)
        } else {
            false
        }
    }

    pub fn match_delim(&self, d: char) -> bool {
        if self.pos >= self.s.len() {
            return false;
        }
        self.s.chars().nth(self.pos).unwrap() == d
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.s.len() {
            let c = self.s.chars().nth(self.pos).unwrap();
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn is_id_char(&self, c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn is_keyword_at_pos(&self) -> bool {
        let start = self.pos;
        let mut pos = self.pos;
        while pos < self.s.len() {
            let c = self.s.chars().nth(pos).unwrap();
            if c.is_ascii_alphanumeric() || c == '_' {
                pos += 1;
            } else {
                break;
            }
        }
        if pos > start {
            let word = self.s[start..pos].to_lowercase();
            self.keywords.contains(&word)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_test() {
        let s = "select name, address from t1 where id = 123 and 18 < age";
        let mut lexer = Lexer::new(s);
        lexer.eat_keyword("select");
        assert_eq!("name", lexer.eat_id());
        lexer.eat_delim(',');
        assert_eq!("address", lexer.eat_id());
        lexer.eat_keyword("from");
        assert_eq!("t1", lexer.eat_id());
        lexer.eat_keyword("where");
        assert_eq!("id", lexer.eat_id());
        lexer.eat_delim('=');
        assert_eq!(123, lexer.eat_int_constant());
        lexer.eat_keyword("and");
        assert_eq!(18, lexer.eat_int_constant());
        lexer.eat_delim('<');
        assert_eq!("age", lexer.eat_id());
    }
}
