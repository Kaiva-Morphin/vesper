use num_bigint::BigUint;
use num_traits::Zero;

/// Algorithm originally written in C by Mark Adler (https://stackoverflow.com/users/1180620/mark-adler)
/// src: https://stackoverflow.com/a/12162059/29967053

fn combination_encode_between(
    pack: &mut BigUint,
    base: &mut BigUint,
    set: &[u32],
    low: usize,
    high: usize,
) {
    let mid = (low + high) / 2;
    if mid == low {
        assert!(set[low] < set[high]);
        return;
    }
    *pack += &(base.clone() * BigUint::from(set[mid] - set[low] - 1));
    *base *= BigUint::from(set[high] - set[low] - 1);

    combination_encode_between(pack, base, set, low, mid);
    combination_encode_between(pack, base, set, mid, high);
    return;
}

fn combination_encode(set: &[u32], max: u32) -> BigUint {
    let mut pack = BigUint::zero();
    if set.is_empty() {
        return pack;
    }

    pack = BigUint::from(set[0]);
    if set.len() < 2 {
        assert!(set[0] <= max);
        return pack;
    }
    assert!(set[set.len() - 1] <= max);

    let mut base = BigUint::from(max - set.len() as u32 + 2);
    pack += &base * BigUint::from(set[set.len() - 1] - set[0] - 1); 
    base *= BigUint::from(max - set[0]);

    combination_encode_between(&mut pack, &mut base, set, 0, set.len() - 1);
    pack
}

fn combination_decode_between(unpack: &mut BigUint, set: &mut [u32], low: usize, high: usize) {
    let mid = (low + high) / 2;
    if mid == low {
        return;
    }

    let div = set[high] - set[low] - 1;
    let rem = *(&*unpack % div).to_u32_digits().get(0).unwrap_or(&0);
    *unpack /= div;
    set[mid] = set[low] + 1 + rem;

    combination_decode_between(unpack, set, low, mid);
    combination_decode_between(unpack, set, mid, high);
}

fn combination_decode(pack: &BigUint, num: usize, max: u32) -> Vec<u32> {
    if num == 0 {
        return Vec::new();
    }

    let mut unpack = pack.clone();
    let mut set = vec![0; num];

    if num == 1 {
        set[0] = *unpack.to_u32_digits().get(0).unwrap_or(&0);
        return set;
    }

    let div = max - num as u32 + 2;
    set[0] = *(&unpack % div).to_u32_digits().get(0).unwrap_or(&0);
    unpack /= div;

    let rem = *(&unpack % (max - set[0])).to_u32_digits().get(0).unwrap_or(&0);
    unpack /= max - set[0];
    set[num - 1] = set[0] + 1 + rem;

    combination_decode_between(&mut unpack, &mut set, 0, num - 1);
    set
}

///! SET MUST BE WITHOUT DUPLICATES
pub fn encode_set(set: &mut Vec<u32>) ->Vec<u8> {
    let len = set.len();
    if len == 0 {return vec![]}
    set.sort();
    let max = *set.iter().last().unwrap();
    let encoded = combination_encode(set, max);
    let mut bytes = encoded.to_bytes_le();
    let mut additional_data = (len as u32).to_le_bytes().to_vec(); 
    additional_data.extend_from_slice(&max.to_le_bytes());
    bytes.extend_from_slice(&additional_data);
    bytes
}

pub fn decode_set(bytes: Vec<u8>) -> Vec<u32> {
    if bytes.len() < 8 {return vec![]}
    let len = u32::from_le_bytes(bytes[bytes.len()-8..bytes.len()-4].try_into().unwrap());
    let max = u32::from_le_bytes(bytes[bytes.len()-4..].try_into().unwrap());
    let encoded_data = &bytes[..bytes.len()-8];
    combination_decode(&BigUint::from_bytes_le(encoded_data),len as usize, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode_decode_single_element() {
        let mut set = vec![3];
        let encoded = encode_set(&mut set);
        let decoded = decode_set(encoded);

        assert_eq!(set, decoded);
    }

    #[test]
    fn test_encode_decode_single_large_element() {
        let mut set = vec![u32::MAX];
        let encoded = encode_set(&mut set);
        let decoded = decode_set(encoded);

        assert_eq!(set, decoded);
    }

    #[test]
    fn test_encode_decode_multiple_elements() {
        let mut set = vec![1, 3, 5];
        let encoded = encode_set(&mut set);
        let decoded = decode_set(encoded);

        assert_eq!(set, decoded);
    }

    #[test]
    fn test_encode_decode_large_set() {
        let mut set = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let encoded = encode_set(&mut set);
        let decoded = decode_set(encoded);

        assert_eq!(set, decoded);
    }

    #[test]
    fn test_encode_decode_empty_set() {
        let mut set: Vec<u32> = vec![];
        let encoded = encode_set(&mut set);
        let decoded = decode_set(encoded);

        assert_eq!(set, decoded);
    }

    #[test]
    fn test_encode_decode_edge_case() {
        let mut set = vec![0, 10, 100];
        let encoded = encode_set(&mut set);
        let decoded = decode_set(encoded);
        
        assert_eq!(set, decoded);
    }

    #[test]
    fn test_encode_decode_with_large_numbers() {
        let mut set = vec![u32::MAX - 2, u32::MAX - 1, u32::MAX];
        let encoded = encode_set(&mut set);
        let decoded = decode_set(encoded);

        assert_eq!(set, decoded);
    }

    #[test]
    fn test_encode_decode_non_sequential_set() {
        let mut set = vec![5, 10, 15, 20];
        let encoded = encode_set(&mut set);
        let decoded = decode_set(encoded);

        assert_eq!(set, decoded);
    }

    #[test]
    fn test_encode_decode_large_set_of_size_10000() {
        let mut set: Vec<u32> = (1..=10000).collect();
        let encoded = encode_set(&mut set);
        let decoded = decode_set(encoded);

        assert_eq!(set, decoded);
    }
}
