use std::sync::Arc;

use ckb_types::{bytes::Bytes, packed, prelude::*};

use common_config_parser::types::ConfigRocksDB;
use protocol::types::H256;

use crate::system_contract::image_cell::{
    cell_key, get_cell, get_header, header_key, image_cell_abi, CellInfo, RocksTrieDB,
};
use crate::MPTTrie;

use super::*;

const ROCKDB_PATH: &str = "src/tests/system_script/image_cell_data";
// const ROCKDB_PATH: &str = "../../devtools/chain/data/rocksdb/image_cell_data";

// copy from terminal: core/executor/src/system_contract/image_cell/mod.rs:L75
const ROOT: &str = "a877d142221e08cd27d2293e56446dd7b2108b5813e39271dbc107736ae01c19";

#[test]
fn inspect_mpt() {
    let root = H256(decode_hex(ROOT));
    println!("root: {:?}", root);

    let trie_db = RocksTrieDB::new(
        ROCKDB_PATH,
        ConfigRocksDB::default(),
        100,
    ).expect("[image cell] new rocksdb error");
    let mpt = MPTTrie::from_root(root, Arc::new(trie_db)).unwrap();

    let block_number = 0x5fe340;
    println!("block_number: {:?}", block_number);

    let block_hash = decode_hex("2d743830a44d39f0011dd4e0fcf2fcc5dcd02df95d2725e0bc2a32b458af4bc6");
    let header_key = header_key(&block_hash, block_number);
    let header = get_header(&mpt, &header_key).unwrap().unwrap();
    println!("\n---------------header---------------\n\n{}", header);
    check_header(&header, &right_header());

    let tx_hash = decode_hex("3d9075de60200689507f8c389be6101b1d4496ba9ef0a6b272ba37fd24f3a24b");
    let cell = get_cell(&mpt, &cell_key(&tx_hash, 0x0)).unwrap().unwrap();
    println!("\n---------------cell------------------\n");
    println!("data: {:?}\n", cell.cell_data);
    println!("output: {}\n",  packed::CellOutput::new_unchecked(cell.cell_output.clone()));
    println!("created_number: {:?}\n", cell.created_number);
    println!("consumed_number: {:?}\n", cell.consumed_number);
    check_cell(&cell, &right_outputs()[0], block_number, None);
}

pub fn decode_hex(s: &str) -> [u8; 32] {
    let mut buf = [0u8; 32];
    hex::decode_to_slice(s, &mut buf).unwrap();
    buf
}

fn check_header(get_header: &packed::Header, dst_header: &image_cell_abi::Header) {
    let nonce: packed::Uint128 = dst_header.nonce.pack();
    assert_eq!(get_header.nonce().raw_data(), nonce.raw_data());

    let get_header = get_header.raw();

    assert_eq!(get_header.dao(), dst_header.dao.pack());
    assert_eq!(get_header.extra_hash(), dst_header.block_hash.pack());
    assert_eq!(get_header.parent_hash(), dst_header.parent_hash.pack());
    assert_eq!(get_header.proposals_hash(), dst_header.proposals_hash.pack());
    assert_eq!(
        get_header.transactions_root(),
        dst_header.transactions_root.pack()
    );

    let version: packed::Uint32 = dst_header.version.pack();
    assert_eq!(get_header.version().raw_data(), version.raw_data());

    let compact_target: packed::Uint32 = dst_header.compact_target.pack();
    assert_eq!(
        get_header.compact_target().raw_data(),
        compact_target.raw_data()
    );

    let timestamp: packed::Uint64 = dst_header.timestamp.pack();
    assert_eq!(get_header.timestamp().raw_data(), timestamp.raw_data());

    let number: packed::Uint64 = dst_header.number.pack();
    assert_eq!(get_header.number().raw_data(), number.raw_data());

    let epoch: packed::Uint64 = dst_header.epoch.pack();
    assert_eq!(get_header.epoch().raw_data(), epoch.raw_data());
}

fn check_cell(
    get_cell: &CellInfo,
    dst_cell: &image_cell_abi::CellInfo,
    created_number: u64,
    consumed_number: Option<u64>,
) {
    let data: packed::Bytes = dst_cell.data.pack();
    assert_eq!(get_cell.cell_data, data.raw_data());

    check_cell_output(&get_cell.cell_output, &dst_cell.output);

    assert_eq!(get_cell.created_number, created_number);
    if get_cell.consumed_number.is_some() {
        assert_eq!(get_cell.consumed_number.unwrap(), consumed_number.unwrap());
    }
}

fn check_cell_output(get_output: &Bytes, output: &image_cell_abi::CellOutput) {
    let get_output: packed::CellOutput = packed::CellOutput::from_slice(get_output).unwrap();

    let capacity: packed::Uint64 = output.capacity.pack();
    assert_eq!(get_output.capacity().raw_data(), capacity.raw_data());

    check_script(&get_output.lock(), &output.lock);

    if !output.type_.is_empty() {
        check_script(&get_output.type_().to_opt().unwrap(), &output.type_[0]);
    } else {
        assert!(get_output.type_().to_opt().is_none());
    }
}

fn check_script(get_script: &packed::Script, script: &image_cell_abi::Script) {
    assert_eq!(get_script.code_hash(), script.code_hash.pack());

    let hash_type: packed::Byte = packed::Byte::new(script.hash_type);
    assert_eq!(get_script.hash_type(), hash_type);

    let args: packed::Bytes = script.args.pack();
    assert_eq!(get_script.args().raw_data(), args.raw_data());
}

fn right_header() -> image_cell_abi::Header {
    image_cell_abi::Header {
        version:           0x0,
        compact_target:    0x1d076781,
        timestamp:         0x1827b3557ba,
        number:            0x5fe340,
        epoch:             0x70802650012e7,
        parent_hash:       decode_hex("66ebbd53c7e88a513dd62504d2c4eaabfe05c6a9f93ea31916b4e9a556476401"),
        transactions_root: decode_hex("7c4d3800b4d6392fdde3b0a6798485736b6ff959b23987a6dcdd80c53804c43d"),
        proposals_hash:    decode_hex("d9209d38b175e5a5bb764d05caca62391d0a64f4b7e9d87043b8b32fde33ae53"),
        uncles_hash:       decode_hex("0000000000000000000000000000000000000000000000000000000000000000"),
        dao:               decode_hex("f87fd42b407ca13f7b46cf89d84e26001e74935da194900300350c247e563b08"),
        nonce:             0x1856d79da4cf4c3ad227bf8973ea0680,
        block_hash:        decode_hex("2d743830a44d39f0011dd4e0fcf2fcc5dcd02df95d2725e0bc2a32b458af4bc6"),
    }
}

fn right_outputs() -> Vec<image_cell_abi::CellInfo> {
    vec![image_cell_abi::CellInfo {
        out_point: image_cell_abi::OutPoint {
            tx_hash: decode_hex("3d9075de60200689507f8c389be6101b1d4496ba9ef0a6b272ba37fd24f3a24b"),
            index:   0x0,
        },
        output:    image_cell_abi::CellOutput {
            capacity: 0x277cf2a00,
            lock:     image_cell_abi::Script {
                args:      ethers::core::types::Bytes::from_str("0x5989ae415bb667931a99896e5fbbfad9ba53a223").unwrap(),
                code_hash: decode_hex("9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
                hash_type: 1,
            },
            type_:    vec![image_cell_abi::Script {
                args:      ethers::core::types::Bytes::from_str("0x").unwrap(),
                code_hash: decode_hex("82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e"),
                hash_type: 1,
            }],
        },
        data:      ethers::core::types::Bytes::from_str("0x0000000000000000").unwrap(),
    }]
}
