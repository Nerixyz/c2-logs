pub struct DoubleBuffer {
    last: Vec<u8>,
    next: Vec<u8>,
}

impl DoubleBuffer {
    pub fn new() -> Self {
        Self {
            last: vec![],
            next: vec![],
        }
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.last, &mut self.next);
    }

    pub fn next(&self) -> &[u8] {
        &self.next
    }

    pub fn last(&self) -> &[u8] {
        &self.last
    }

    pub fn next_mut(&mut self) -> &mut Vec<u8> {
        &mut self.next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut buf = DoubleBuffer::new();
        buf.next_mut().resize(1, 0);
        assert_eq!(buf.next().len(), 1);
        assert_eq!(buf.last().len(), 0);
        buf.swap();
        assert_eq!(buf.next().len(), 0);
        assert_eq!(buf.last().len(), 1);
    }
}
