use std::convert::TryInto;

pub trait IntoVecU8 {
    fn into_vec_u8(self) -> Vec<u8>;
}

pub trait FromVecU8 {
    fn from_vec_u8(vec: &[u8]) -> Self;
}

impl IntoVecU8 for i32 {
    fn into_vec_u8(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl FromVecU8 for i32 {
    fn from_vec_u8(vec: &[u8]) -> Self {
        let bytes: [u8; 4] = vec.try_into().expect("Expected a Vec<u8> with length 4");
        Self::from_le_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i32_to_vec_u8() {
        let num: i32 = 12_345_678;
        let expected: Vec<u8> = vec![78, 97, 188, 0];
        let result: Vec<u8> = num.into_vec_u8();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_vec_u8_to_i32() {
        let vec = [78, 97, 188, 0];
        let expected: i32 = 12_345_678;
        let result: i32 = i32::from_vec_u8(&vec);
        assert_eq!(result, expected);
    }
}
