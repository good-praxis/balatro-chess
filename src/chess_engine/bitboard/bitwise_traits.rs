use super::Bitboard;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

// Bitboard implementations
impl BitAnd<Bitboard> for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Self(*self & *rhs)
    }
}

impl BitAndAssign<Bitboard> for Bitboard {
    fn bitand_assign(&mut self, rhs: Bitboard) {
        self.0 &= *rhs;
    }
}

impl BitOr<Bitboard> for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Self(*self | *rhs)
    }
}

impl BitOrAssign<Bitboard> for Bitboard {
    fn bitor_assign(&mut self, rhs: Bitboard) {
        self.0 |= *rhs;
    }
}

impl BitXor<Bitboard> for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Bitboard) -> Self::Output {
        Self(*self ^ *rhs)
    }
}

impl BitXorAssign<Bitboard> for Bitboard {
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        self.0 ^= *rhs;
    }
}

// u128 implementations
impl BitAnd<u128> for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: u128) -> Self::Output {
        Self(*self & rhs)
    }
}

impl BitAndAssign<u128> for Bitboard {
    fn bitand_assign(&mut self, rhs: u128) {
        self.0 &= rhs;
    }
}

impl BitOr<u128> for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: u128) -> Self::Output {
        Self(*self | rhs)
    }
}

impl BitOrAssign<u128> for Bitboard {
    fn bitor_assign(&mut self, rhs: u128) {
        self.0 |= rhs;
    }
}

impl BitXor<u128> for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: u128) -> Self::Output {
        Self(*self ^ rhs)
    }
}

impl BitXorAssign<u128> for Bitboard {
    fn bitxor_assign(&mut self, rhs: u128) {
        self.0 ^= rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_and() {
        let b1 = Bitboard(0b0011);
        let b2 = Bitboard(0b0101);
        let res = b1 & b2;
        assert_eq!(res, Bitboard(0b0001));
    }

    #[test]
    fn bit_and_assign() {
        let mut b1 = Bitboard(0b0011);
        let b2 = Bitboard(0b0101);
        b1 &= b2;
        assert_eq!(b1, Bitboard(0b0001));
    }

    #[test]
    fn bit_or() {
        let b1 = Bitboard(0b0011);
        let b2 = Bitboard(0b0101);
        let res = b1 | b2;
        assert_eq!(res, Bitboard(0b0111));
    }

    #[test]
    fn bit_or_assign() {
        let mut b1 = Bitboard(0b0011);
        let b2 = Bitboard(0b0101);
        b1 |= b2;
        assert_eq!(b1, Bitboard(0b0111));
    }

    #[test]
    fn bit_xor() {
        let b1 = Bitboard(0b0011);
        let b2 = Bitboard(0b0101);
        let res = b1 ^ b2;
        assert_eq!(res, Bitboard(0b0110));
    }

    #[test]
    fn bit_xor_assign() {
        let mut b1 = Bitboard(0b0011);
        let b2 = Bitboard(0b0101);
        b1 ^= b2;
        assert_eq!(b1, Bitboard(0b0110));
    }

    #[test]
    fn bit_and_u128() {
        let b1 = Bitboard(0b0011);
        let b2 = 0b0101;
        let res = b1 & b2;
        assert_eq!(res, Bitboard(0b0001));
    }

    #[test]
    fn bit_and_assign_u128() {
        let mut b1 = Bitboard(0b0011);
        let b2 = 0b0101;
        b1 &= b2;
        assert_eq!(b1, Bitboard(0b0001));
    }

    #[test]
    fn bit_or_u128() {
        let b1 = Bitboard(0b0011);
        let b2 = 0b0101;
        let res = b1 | b2;
        assert_eq!(res, Bitboard(0b0111));
    }

    #[test]
    fn bit_or_assign_u128() {
        let mut b1 = Bitboard(0b0011);
        let b2 = 0b0101;
        b1 |= b2;
        assert_eq!(b1, Bitboard(0b0111));
    }

    #[test]
    fn bit_xor_u128() {
        let b1 = Bitboard(0b0011);
        let b2 = 0b0101;
        let res = b1 ^ b2;
        assert_eq!(res, Bitboard(0b0110));
    }

    #[test]
    fn bit_xor_assign_u128() {
        let mut b1 = Bitboard(0b0011);
        let b2 = 0b0101;
        b1 ^= b2;
        assert_eq!(b1, Bitboard(0b0110));
    }
}
