use std::collections::VecDeque;

pub trait Stream<T: Clone> {
    fn next(&mut self) -> Option<T>;
}

pub struct CharsStream {
    chars: Vec<char>,
    index: usize,
}

impl CharsStream {
    pub fn new(s: &str) -> Self {
        Self {
            chars: s.to_string().chars().collect(),
            index: 0,
        }
    }
}

impl Stream<char> for CharsStream {
    fn next(&mut self) -> Option<char> {
        if self.index < self.chars.len() {
            let c = self.chars[self.index];
            self.index += 1;
            Some(c)
        } else {
            None
        }
    }
}

pub struct BufferedStream<T> {
    buffer: VecDeque<T>,
    stream: Box<dyn Stream<T>>,
}

impl<T: Clone> BufferedStream<T> {
    pub fn new(stream: Box<dyn Stream<T>>) -> Self {
        Self {
            buffer: VecDeque::new(),
            stream,
        }
    }

    pub fn next(&mut self) -> Option<T> {
        self.fill_buffer(1);
        self.buffer.pop_front()
    }

    pub fn peek(&mut self) -> Option<T> {
        self.fill_buffer(1);
        self.buffer.front().cloned()
    }

    pub fn peek_many(&mut self, n: i32) -> Vec<T> {
        self.fill_buffer(n);
        self.buffer.iter().take(n as usize).cloned().collect()
    }

    fn fill_buffer(&mut self, max_size: i32) {
        while self.buffer.len() < max_size as usize {
            if let Some(token) = self.stream.next() {
                self.buffer.push_back(token);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chars_stream() {
        let mut stream = CharsStream::new("hello");
        assert_eq!(stream.next(), Some('h'));
        assert_eq!(stream.next(), Some('e'));
        assert_eq!(stream.next(), Some('l'));
        assert_eq!(stream.next(), Some('l'));
        assert_eq!(stream.next(), Some('o'));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_buffered_stream() {
        let mut stream = BufferedStream::new(Box::new(CharsStream::new("hello")));
        assert_eq!(stream.next(), Some('h'));
        assert_eq!(stream.next(), Some('e'));
        assert_eq!(stream.next(), Some('l'));
        assert_eq!(stream.next(), Some('l'));
        assert_eq!(stream.next(), Some('o'));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_buffered_stream_peek() {
        let mut stream = BufferedStream::new(Box::new(CharsStream::new("hello")));
        assert_eq!(stream.peek(), Some('h'));
        assert_eq!(stream.next(), Some('h')); // Ensure peek doesn't advance the stream
    }

    #[test]
    fn test_buffered_stream_peek_many() {
        let mut stream = BufferedStream::new(Box::new(CharsStream::new("hello")));
        assert_eq!(stream.peek_many(3), vec!['h', 'e', 'l']);
        assert_eq!(stream.next(), Some('h')); // Ensure peek_many doesn't advance the stream
    }

    #[test]
    fn test_buffered_stream_peek_many_more() {
        let mut stream = BufferedStream::new(Box::new(CharsStream::new("hello")));
        assert_eq!(stream.peek_many(10), vec!['h', 'e', 'l', 'l', 'o']);
        assert_eq!(stream.next(), Some('h')); // Ensure peek_many doesn't advance the stream
    }
}
