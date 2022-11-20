

pub trait Matcher {
    fn peek(&self) -> Option<u8>;
    fn advance(&mut self) -> Option<u8>;

    /// Tries to match the next byte in the input stream to the given function.
    /// If the byte matches, returns Some, otherwise returns None. Or, if the
    /// stream ends, may also return None.
    fn match_fn<F>(&mut self, f: F) -> Option<u8>
        where
            F: Fn(Option<u8>) -> bool
    {
        let matches = f(self.peek());
        if matches {
            self.advance()
        }
        else {
            None
        }
    }

    fn match_to_vec<F>(&mut self, f: F) -> Option<Vec<u8>>
        where
            F: Fn(Option<u8>) -> bool
    {
        self.match_fn(f).map(|byte| vec![byte])
    }

    fn match_onto_vec<F>(&mut self, vector: &mut Vec<u8>, f: F)
        where
            F: Fn(Option<u8>) -> bool
    {
        while let Some(byte) = self.match_fn(&f) {
            vector.push(byte);
        }
    }

    fn match_one(&mut self, character: u8) -> bool {
        if self.peek().map(|c| c == character).unwrap_or(false) {
            self.advance();
            return true;
        }
        false
    }

    fn match_not(&mut self, character: u8) -> Option<u8> {
        // Assume that EOF also does not match. We basically never want to match EOF.
        if self.peek().map(|c| c != character).unwrap_or(false) {
            return self.advance()
        }
        None
    }
}