use protocol::types::{Bloom, Hasher, Log, H160, H256, U256};

use crate::FeeAllocate;

const FUNC_SELECTOR_LEN: usize = 4;
const U256_BE_BYTES_LEN: usize = 32;
const REVERT_MSG_LEN_OFFSET: usize = FUNC_SELECTOR_LEN + U256_BE_BYTES_LEN;
const REVERT_EFFECT_MSG_OFFSET: usize = REVERT_MSG_LEN_OFFSET + U256_BE_BYTES_LEN;
const BLOOM_BYTE_LENGTH: usize = 256;
const EXEC_REVERT: &str = "execution reverted: ";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeeInlet {
    pub address: H160,
    pub amount:  U256,
}

#[derive(Default, Clone, Debug)]
pub struct DefaultFeeAllocator;

impl FeeAllocate for DefaultFeeAllocator {
    fn allocate(
        &self,
        block_number: U256,
        fee_collect: U256,
        _proposer: H160,
        validators: &[protocol::types::ValidatorExtend],
    ) -> Vec<FeeInlet> {
        if fee_collect.is_zero() || block_number.is_zero() {
            return Vec::new();
        }

        let weight_sum = U256::from(validators.iter().map(|v| v.vote_weight).sum::<u32>());

        validators
            .iter()
            .map(|v| FeeInlet {
                address: v.address,
                amount:  (fee_collect / weight_sum) * v.vote_weight,
            })
            .collect()
    }
}

pub fn code_address(sender: &H160, nonce: &U256) -> H256 {
    let mut stream = rlp::RlpStream::new_list(2);
    stream.append(sender);
    stream.append(nonce);
    Hasher::digest(&stream.out())
}

pub fn decode_revert_msg(input: &[u8]) -> String {
    if input.is_empty() {
        return EXEC_REVERT.to_string();
    }

    let decode_reason = |i: &[u8]| -> String {
        let reason = String::from_iter(i.iter().map(|c| *c as char));
        EXEC_REVERT.to_string() + &reason
    };

    if input.len() < REVERT_EFFECT_MSG_OFFSET {
        return decode_reason(input);
    }

    let end_offset = REVERT_EFFECT_MSG_OFFSET
        + U256::from_big_endian(&input[REVERT_MSG_LEN_OFFSET..REVERT_EFFECT_MSG_OFFSET]).as_usize();

    if input.len() < end_offset {
        return decode_reason(input);
    }

    decode_reason(&input[REVERT_EFFECT_MSG_OFFSET..end_offset])
}

pub fn logs_bloom<'a, I>(logs: I) -> Bloom
where
    I: Iterator<Item = &'a Log>,
{
    let mut bloom = Bloom::zero();

    for log in logs {
        m3_2048(&mut bloom, log.address.as_bytes());
        for topic in log.topics.iter() {
            m3_2048(&mut bloom, topic.as_bytes());
        }
    }
    bloom
}

fn m3_2048(bloom: &mut Bloom, x: &[u8]) {
    let hash = Hasher::digest(x).0;
    for i in [0, 2, 4] {
        let bit = (hash[i + 1] as usize + ((hash[i] as usize) << 8)) & 0x7FF;
        bloom.0[BLOOM_BYTE_LENGTH - 1 - bit / 8] |= 1 << (bit % 8);
    }
}

#[cfg(test)]
mod tests {
    use protocol::codec::{hex_decode, hex_encode};

    use super::*;

    #[test]
    fn test_code_address() {
        let sender = H160::from_slice(
            hex_decode("8ab0cf264df99d83525e9e11c7e4db01558ae1b1")
                .unwrap()
                .as_ref(),
        );
        let nonce: U256 = 0u64.into();
        let addr: H160 = code_address(&sender, &nonce).into();
        assert_eq!(
            hex_encode(addr.0).as_str(),
            "a13763691970d9373d4fab7cc323d7ba06fa9986"
        );

        let sender = H160::from_slice(
            hex_decode("6ac7ea33f8831ea9dcc53393aaa88b25a785dbf0")
                .unwrap()
                .as_ref(),
        );
        let addr: H160 = code_address(&sender, &nonce).into();
        assert_eq!(
            hex_encode(addr.0).as_str(),
            "cd234a471b72ba2f1ccf0a70fcaba648a5eecd8d"
        )
    }
}
