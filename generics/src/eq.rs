#[derive(Default, Debug)]
pub struct Equation<IterMethod> {
    current: u32,
    _method: IterMethod,
}

#[derive(Default, Debug)]
pub struct Linear;

#[derive(Default, Debug)]
pub struct Quadratic;

impl Iterator for Equation<Linear> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;
        Some(self.current)
    }
}

impl Iterator for Equation<Quadratic> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += 2;
        if self.current >= u16::MAX as u32 {
            None
        } else {
            Some(self.current * self.current)
        }
    }
}

mod tests {
    use crate::eq::{Equation, Linear};

    #[test]
    fn test_linear() {
        let mut eq = Equation::<Linear>::default();
        assert_eq!(eq.next(), Some(1));
        assert_eq!(eq.next(), Some(2));
        assert_eq!(eq.next(), Some(3));
        assert_eq!(eq.next(), Some(4));
        assert_eq!(eq.next(), Some(5));
        assert_eq!(eq.next(), Some(6));
    }
}
