use std::sync::Arc;

use ckb_types::{bytes::Bytes, packed, prelude::*};
use ckb_types::core::TransactionView;
use common_config_parser::types::ConfigRocksDB;
use core_rpc_client::RpcClient;

use protocol::tokio;
use protocol::traits::CkbClient;
use protocol::types::{H256, OutPoint};

use crate::system_contract::image_cell::{
    image_cell_abi, init, update_mpt_root, CellInfo, CellKey, ImageCellContract,
};

use super::*;

const ROCKSDB_PATH: &str = "src/tests/system_script/image_cell_data";
// const ROCKDB_PATH: &str = "../../devtools/chain/data/rocksdb/image_cell_data";

// copy from terminal: core/executor/src/system_contract/image_cell/mod.rs:L75
const ROOT: &str = "da6471312971240f520a3f159871deac4d88e1e20f260a4ebe19feaa143bf46d";

lazy_static::lazy_static! {
    pub static ref RPC: RpcClient = init_rpc_client();
}

#[tokio::test(flavor = "multi_thread")]
async fn inspect_mpt() {
    init_rpc_client();

    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
    update_mpt_root(&mut backend,  H256(decode_hex(ROOT)));

    let executor = ImageCellContract::default();
    init(
        ROCKSDB_PATH,
        ConfigRocksDB::default(),
        Arc::new(backend.clone()),
    );

    let block_number = 6318236;
    println!("block_number: {:?}", block_number);

    let cell_key = CellKey::new(
        decode_hex("ff25295d3cc036969f8498f53af4df3a3be3841a4730287881ebe29f8f92234b"), 
        0x0
    );

    let cell = executor.get_cell(&cell_key).unwrap().unwrap();
    println!("\n---------------cell------------------\n");
    println!("data: {:?}\n", cell.cell_data);
    println!("output: {}\n",  packed::CellOutput::new_unchecked(cell.cell_output.clone()));
    println!("created_number: {:?}\n", cell.created_number);
    println!("consumed_number: {:?}\n", cell.consumed_number);

    let expect_cell = get_cell_by_out_point(OutPoint {
        tx_hash: H256(decode_hex("ff25295d3cc036969f8498f53af4df3a3be3841a4730287881ebe29f8f92234b")),
        index:   0x0,
    })
    .await;
    check_cell(&cell, &expect_cell, block_number, None);
}

pub fn decode_hex(s: &str) -> [u8; 32] {
    let mut buf = [0u8; 32];
    hex::decode_to_slice(s, &mut buf).unwrap();
    buf
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

async fn get_cell_by_out_point(out_point: OutPoint) -> image_cell_abi::CellInfo {
    let (cell, data) = get_ckb_tx(out_point.tx_hash.0)
        .await
        .output_with_data(out_point.index as usize)
        .unwrap();

    let lock_script = image_cell_abi::Script {
        code_hash: cell.lock().code_hash().unpack().0,
        hash_type: cell.lock().hash_type().as_slice()[0],
        args:      {
            let tmp: Vec<u8> = cell.lock().args().unpack();
            tmp.into()
        },
    };
    let mut type_script = vec![];
    if let Some(s) = cell.type_().to_opt() {
        type_script.push(image_cell_abi::Script {
            code_hash: s.code_hash().unpack().0,
            hash_type: s.hash_type().as_slice()[0],
            args:      {
                let tmp: Vec<u8> = s.args().unpack();
                tmp.into()
            },
        })
    }

    let cell_output = image_cell_abi::CellOutput {
        capacity: cell.capacity().unpack(),
        lock:     lock_script,
        type_:    type_script,
    };

    image_cell_abi::CellInfo {
        out_point: out_point.into(),
        output:    cell_output,
        data:      data.into(),
    }
}

async fn get_ckb_tx<T: Into<ckb_types::H256>>(hash: T) -> TransactionView {
    let tx: packed::Transaction = RPC
        .get_txs_by_hashes(Default::default(), vec![hash.into()])
        .await
        .unwrap()
        .get(0)
        .cloned()
        .unwrap()
        .unwrap()
        .inner
        .into();
    tx.into_view()
}

fn init_rpc_client() -> RpcClient {
    RpcClient::new(
        "https://testnet.ckb.dev/",
        "http://127.0.0.1:8116",
        "http://127.0.0.1:8118",
    )
}
