use super::Bitboard;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!*self)
    }
}

// Bitboard implementations
impl BitAnd for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(*self & *rhs)
    }
}

impl BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= *rhs;
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(*self | *rhs)
    }
}

impl BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= *rhs;
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(*self ^ *rhs)
    }
}

impl BitXorAssign for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= *rhs;
    }
}

// u128 implementations
impl BitAnd<u128> for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: u128) -> Self::Output {
        Self(*self & rhs)
    }
}

impl BitAndAssign<u128> for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: u128) {
        self.0 &= rhs;
    }
}

impl BitOr<u128> for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitor(self, rhs: u128) -> Self::Output {
        Self(*self | rhs)
    }
}

impl BitOrAssign<u128> for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: u128) {
        self.0 |= rhs;
    }
}

impl BitXor<u128> for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitxor(self, rhs: u128) -> Self::Output {
        Self(*self ^ rhs)
    }
}

impl BitXorAssign<u128> for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u128) {
        self.0 ^= rhs;
    }
}

// Shifts
impl Shl<u32> for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn shl(self, rhs: u32) -> Self::Output {
        Self(self.wrapping_shl(rhs))
    }
}

impl ShlAssign<u32> for Bitboard {
    #[inline]
    fn shl_assign(&mut self, rhs: u32) {
        self.0 = self.wrapping_shl(rhs);
    }
}

impl Shr<u32> for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn shr(self, rhs: u32) -> Self::Output {
        Self(self.wrapping_shr(rhs))
    }
}

impl ShrAssign<u32> for Bitboard {
    #[inline]
    fn shr_assign(&mut self, rhs: u32) {
        self.0 = self.wrapping_shr(rhs);
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

    #[test]
    fn bit_shl() {
        let b1 = Bitboard(0b010);
        let res = b1 << 1;
        assert_eq!(res, Bitboard(0b100));
    }
    #[test]
    fn bit_shl_assign() {
        let mut b1 = Bitboard(0b010);
        b1 <<= 1;
        assert_eq!(b1, Bitboard(0b100));
    }
    #[test]
    fn bit_shr() {
        let b1 = Bitboard(0b010);
        let res = b1 >> 1;
        assert_eq!(res, Bitboard(0b001));
    }
    #[test]
    fn bit_shr_assign() {
        let mut b1 = Bitboard(0b010);
        b1 >>= 1;
        assert_eq!(b1, Bitboard(0b001));
    }
}
