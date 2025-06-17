pub struct Tokenizer<'a> {
    content: &'a [char],
}

impl<'a> Tokenizer<'a> {
    /// Contruct a new Tokenizer
    pub fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    /// Chops the first `n` chars and returns them.
    /// # Panics
    /// This method panics if `n` is out of bound
    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[..n];
        self.content = &self.content[n..];
        token
    }

    fn chop_while(&mut self, predicate: impl Fn(char) -> bool) -> &'a [char] {
        let mut i = 0;
        while i < self.content.len() && predicate(self.content[i]) {
            i += 1;
        }
        self.chop(i)
    }

    fn next_token(&mut self) -> Option<String> {
        self.chop_while(char::is_whitespace);

        if self.content.is_empty() {
            return None;
        }

        let token = if self.content[0].is_alphabetic() {
            self.chop_while(char::is_alphanumeric)
        } else if self.content[0].is_numeric() {
            self.chop_while(char::is_numeric)
        } else {
            self.chop(1)
        };

        Some(token.iter().collect())
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
