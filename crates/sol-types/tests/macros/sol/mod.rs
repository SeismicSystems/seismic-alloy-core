use alloy_primitives::{b256, bytes, hex, keccak256, Address, B256, I256, U256};
use alloy_sol_types::{sol, SolCall, SolError, SolEvent, SolStruct, SolType};
use serde::Serialize;
use serde_json::Value;

#[cfg(feature = "json")]
mod abi;
#[cfg(feature = "json")]
mod json;

mod eip712;

#[test]
fn e2e() {
    sol! {
        struct MyStruct {
            uint256 a;
            bytes32 b;
            address[] c;
        }
    }

    sol! {
        struct MyStruct2 {
            MyStruct a;
            bytes32 b;
            address[] c;
        }
    }

    sol! {
        struct MyStruct3 {
            saddress b;
            suint256 c;
            sint248 d;
        }
    }

    type MyTuple = sol! {
        (MyStruct, bytes32)
    };

    type LateBinding<A> = sol! {
        (A[], address)
    };

    type NestedArray = sol! {
        bool[2][]
    };

    sol! {
        type MyValueType is uint256;
    }

    <sol!(bool)>::abi_encode(&true);

    let a = MyStruct { a: U256::from(1), b: [0; 32].into(), c: Vec::new() };

    MyTuple::abi_encode(&(a.clone(), [0; 32]));
    MyStruct::abi_encode(&a);

    LateBinding::<MyStruct>::abi_encode(&(vec![a.clone(), a.clone()], Address::default()));

    MyStruct2::abi_encode(&MyStruct2 { a, b: [0; 32].into(), c: vec![] });

    NestedArray::abi_encode(&vec![[true, false], [true, false], [true, false]]);

    let mvt = MyValueType::from(U256::from(1));
    assert_eq!(
        mvt.abi_encode(),
        alloy_sol_types::sol_data::Uint::<256>::abi_encode(&U256::from(1))
    );
}

#[test]
fn function() {
    sol! {
        #[derive(Debug, PartialEq)]
        struct CustomStruct {
            address a;
            uint64 b;
        }

        #[derive(Debug, PartialEq)]
        function someFunction(
            uint256 basic,
            string memory string_,
            bytes calldata longBytes,
            address[] memory array,
            bool[2] memory fixedArray,
            CustomStruct struct_,
            CustomStruct[] structArray,
        ) returns (bool x);
    }

    let sig =
        "someFunction(uint256,string,bytes,address[],bool[2],(address,uint64),(address,uint64)[])";
    assert_eq!(someFunctionCall::SIGNATURE, sig);
    assert_eq!(someFunctionCall::SELECTOR, keccak256(sig)[..4]);

    let call = someFunctionCall {
        basic: U256::from(1),
        string_: "Hello World".to_owned(),
        longBytes: bytes![0; 36],
        array: vec![Address::ZERO, Address::ZERO, Address::ZERO],
        fixedArray: [true, false],
        struct_: CustomStruct { a: Address::ZERO, b: 2 },
        structArray: vec![
            CustomStruct { a: Address::ZERO, b: 3 },
            CustomStruct { a: Address::ZERO, b: 4 },
            CustomStruct { a: Address::ZERO, b: 5 },
            CustomStruct { a: Address::ZERO, b: 6 },
        ],
    };
    let encoded = call.abi_encode();
    assert_eq!(someFunctionCall::abi_decode(&encoded).unwrap(), call);

    assert_eq!(
        call.abi_encoded_size(),
        32 + (64 + 32) + (64 + 32 + 32) + (64 + 3 * 32) + 2 * 32 + (32 + 32) + (64 + 4 * (32 + 32))
    );
    assert_eq!(encoded.len(), 4 + call.abi_encoded_size());
}

#[test]
fn function_returns() {
    sol! {
        #[derive(Debug, PartialEq)]
        function test() returns (uint256[]);
    }
    assert_eq!(
        testCall::abi_decode_returns(&hex!(
            "0000000000000000000000000000000000000000000000000000000000000020
                 0000000000000000000000000000000000000000000000000000000000000000"
        ),),
        Ok(vec![])
    );
    assert_eq!(
        testCall::abi_decode_returns(&hex!(
            "0000000000000000000000000000000000000000000000000000000000000020
                 0000000000000000000000000000000000000000000000000000000000000001
                 0000000000000000000000000000000000000000000000000000000000000002"
        )),
        Ok(vec![U256::from(2)])
    );
    assert_eq!(
        testCall::abi_decode_returns(&hex!(
            "0000000000000000000000000000000000000000000000000000000000000020
                 0000000000000000000000000000000000000000000000000000000000000002
                 0000000000000000000000000000000000000000000000000000000000000042
                 0000000000000000000000000000000000000000000000000000000000000069"
        ),),
        Ok(vec![U256::from(0x42), U256::from(0x69)])
    );
}

#[test]
fn ret_param_single_test() {
    use alloy_sol_types::SolValue;
    sol! {
        function balanceOf(address owner) returns (uint256);

        function balanceOfUnnamedArray(address owner) returns (uint256[2]);

        #[derive(Debug, PartialEq, Eq)]
        struct MyBalance {
            uint256 bal;
        }
        function balanceOfStructUnnamed(address owner) returns (MyBalance);

        function balanceOfNamed(address owner) returns (uint256 bal);
    }
    let data = vec![42].abi_encode_sequence();
    let res = balanceOfCall::abi_decode_returns(&data).unwrap();

    assert_eq!(res, U256::from(42));

    let res = balanceOfStructUnnamedCall::abi_decode_returns(&data).unwrap();

    assert_eq!(res, MyBalance { bal: U256::from(42) });

    let data = vec![24, 42].abi_encode_sequence();

    let res = balanceOfUnnamedArrayCall::abi_decode_returns(&data).unwrap();

    assert_eq!(res, [U256::from(24), U256::from(42)]);

    let data = vec![42].abi_encode_sequence();
    let res = balanceOfNamedCall::abi_decode_returns(&data).unwrap();

    assert_eq!(res, U256::from(42));

    assert_eq!(balanceOfCall::abi_encode_returns(&res), data);
}

#[test]
fn ret_tuple_param() {
    use alloy_sol_types::SolValue;
    sol! {
        function balanceOfTuple(address owner) returns (uint256, uint256);

        function balanceOfTupleNamed(address owner) returns (uint256 bal, uint256);

        function balanceOfDoubleTuple(address owner) returns ((uint256, uint256), uint256);
    }

    let data = vec![24, 42].abi_encode_sequence();
    let balanceOfTupleReturn { _0, _1 } = balanceOfTupleCall::abi_decode_returns(&data).unwrap();

    assert_eq!(_0, U256::from(24));
    assert_eq!(_1, U256::from(42));

    let balanceOfTupleNamedReturn { bal, _1 } =
        balanceOfTupleNamedCall::abi_decode_returns(&data).unwrap();

    assert_eq!(bal, U256::from(24));
    assert_eq!(_1, U256::from(42));

    let data = vec![24, 42, 69].abi_encode_sequence();
    let balanceOfDoubleTupleReturn { _0: (u1, u2), _1: u3 } =
        balanceOfDoubleTupleCall::abi_decode_returns(&data).unwrap();

    assert_eq!(u1, U256::from(24));
    assert_eq!(u2, U256::from(42));
    assert_eq!(u3, U256::from(69));
}

#[test]
fn error() {
    sol! {
        error SomeError(int a, bool b);
    }

    let sig = "SomeError(int256,bool)";
    assert_eq!(SomeError::SIGNATURE, sig);
    assert_eq!(SomeError::SELECTOR, keccak256(sig)[..4]);

    let e = SomeError { a: I256::ZERO, b: false };
    assert_eq!(e.abi_encoded_size(), 64);
}

// Handle empty call encoding/decoding correctly
// https://github.com/alloy-rs/core/issues/158
#[test]
fn empty_call() {
    sol! {
        interface WETH {
            function deposit() external payable;
        }
    }
    use WETH::depositCall;

    assert_eq!(depositCall {}.abi_encode(), depositCall::SELECTOR);
    assert_eq!(depositCall {}.abi_encoded_size(), 0);
    let mut out = vec![];
    depositCall {}.abi_encode_raw(&mut out);
    assert!(out.is_empty());

    let depositCall {} = depositCall::abi_decode(&depositCall::SELECTOR).unwrap();
    let depositCall {} = depositCall::abi_decode_raw(&[]).unwrap();

    assert!(depositCall::abi_encode_returns(&WETH::depositReturn {}).is_empty());
}

#[test]
fn function_names() {
    sol! {
        #[sol(extra_methods)]
        contract LeadingUnderscores {
            function f();
            function _f();
            function __f();
        }
    }
    use LeadingUnderscores::*;

    let call = LeadingUnderscoresCalls::f(fCall {});
    assert!(call.is_f());
    assert!(!call.is__f());
    assert!(!call.is___f());
}

#[test]
fn getters() {
    // modified from https://docs.soliditylang.org/en/latest/contracts.html#getter-functions
    sol! {
        struct Data {
            uint a;
            bytes3 b;
            uint[3] c;
            uint[] d;
            bytes e;
        }
        mapping(uint => mapping(bool => Data[])) public data1;
        mapping(uint => mapping(bool => Data)) public data2;

        mapping(bool => mapping(address => uint256[])[])[][] public nestedMapArray;
    }

    assert_eq!(data1Call::SIGNATURE, "data1(uint256,bool,uint256)");
    let _ = data1Return { _0: U256::ZERO, _1: [0, 0, 0].into(), _2: bytes![] };

    assert_eq!(data2Call::SIGNATURE, "data2(uint256,bool)");
    let _ = data2Return { _0: U256::ZERO, _1: [0, 0, 0].into(), _2: bytes![] };

    assert_eq!(
        nestedMapArrayCall::SIGNATURE,
        "nestedMapArray(uint256,uint256,bool,uint256,address,uint256)"
    );
    let _ = nestedMapArrayReturn { _0: U256::ZERO };
}

#[test]
fn getter_names() {
    sol! {
        contract Getters {
            string public value;
            string[] public array;
            mapping(bytes32 => string) public map;
            mapping(bytes32 k => string v) public mapWithNames;

            mapping(bytes32 k1 => mapping(uint256 k2 => string v2) v1) public nestedMapWithNames;
        }
    }

    let _ = Getters::valueCall {};
    let _ = Getters::valueReturn { value: String::new() };

    let _ = Getters::arrayCall(U256::ZERO);
    let _ = Getters::arrayReturn { _0: String::new() };

    let _ = Getters::mapCall(B256::ZERO);
    let _ = Getters::mapReturn { _0: String::new() };

    let _ = Getters::mapWithNamesCall { k: B256::ZERO };
    let _ = Getters::mapWithNamesReturn { v: String::new() };

    let _ = Getters::nestedMapWithNamesCall { k1: B256::ZERO, k2: U256::ZERO };
    let _ = Getters::nestedMapWithNamesReturn { v2: String::new() };
}

#[test]
fn abigen_sol_multicall() {
    sol!("../syn-solidity/tests/contracts/Multicall.sol");

    sol! {
        // SPDX-License-Identifier: MIT
        pragma solidity >=0.8.12 <0.9.0;

        interface IMulticall3_2 {
            struct Call {
                address target;
                bytes callData;
            }

            struct Call3 {
                address target;
                bool allowFailure;
                bytes callData;
            }

            struct Call3Value {
                address target;
                bool allowFailure;
                uint256 value;
                bytes callData;
            }

            struct Result {
                bool success;
                bytes returnData;
            }

            function aggregate(Call[] calldata calls) external payable returns (uint256 blockNumber, bytes[] memory returnData);

            function aggregate3(Call3[] calldata calls) external payable returns (Result[] memory returnData);

            function aggregate3Value(Call3Value[] calldata calls) external payable returns (Result[] memory returnData);

            function blockAndAggregate(
                Call[] calldata calls
            ) external payable returns (uint256 blockNumber, bytes32 blockHash, Result[] memory returnData);

            function getBasefee() external view returns (uint256 basefee);

            function getBlockHash(uint256 blockNumber) external view returns (bytes32 blockHash);

            function getBlockNumber() external view returns (uint256 blockNumber);

            function getChainId() external view returns (uint256 chainid);

            function getCurrentBlockCoinbase() external view returns (address coinbase);

            function getCurrentBlockDifficulty() external view returns (uint256 difficulty);

            function getCurrentBlockGasLimit() external view returns (uint256 gaslimit);

            function getCurrentBlockTimestamp() external view returns (uint256 timestamp);

            function getEthBalance(address addr) external view returns (uint256 balance);

            function getLastBlockHash() external view returns (bytes32 blockHash);

            function tryAggregate(
                bool requireSuccess,
                Call[] calldata calls
            ) external payable returns (Result[] memory returnData);

            function tryBlockAndAggregate(
                bool requireSuccess,
                Call[] calldata calls
            ) external payable returns (uint256 blockNumber, bytes32 blockHash, Result[] memory returnData);
        }
    }

    use IMulticall3 as M1;
    use IMulticall3_2 as M2;

    macro_rules! assert_signatures {
        ($($t:ident),+ $(,)?) => {$(
            assert_eq!(
                M1::$t::SIGNATURE,
                M2::$t::SIGNATURE,
                concat!("signature mismatch for ", stringify!($t))
            );
            assert_eq!(
                M1::$t::SELECTOR,
                M2::$t::SELECTOR,
                concat!("selector mismatch for ", stringify!($t))
            );
        )+};
    }

    assert_signatures!(
        aggregate3Call,
        aggregate3ValueCall,
        blockAndAggregateCall,
        getBasefeeCall,
        getBlockHashCall,
        getBlockNumberCall,
        getChainIdCall,
        getCurrentBlockCoinbaseCall,
        getCurrentBlockDifficultyCall,
        getCurrentBlockGasLimitCall,
        getCurrentBlockTimestampCall,
        getEthBalanceCall,
        getLastBlockHashCall,
        tryAggregateCall,
        tryBlockAndAggregateCall,
    );
}

#[test]
fn struct_field_attrs() {
    sol! {
        #[derive(Default, Serialize)]
        struct MyStruct {
            #[serde(skip)]
            uint256 a;
            bytes32 b;
            address[] c;
        }
    }

    assert_eq!(
        serde_json::from_str::<Value>(
            serde_json::to_string(&MyStruct::default()).unwrap().as_str()
        )
        .unwrap()["a"],
        Value::Null
    );
}

#[test]
fn enum_variant_attrs() {
    sol! {
        #[derive(Debug, Default, PartialEq, Eq, Serialize)]
        enum MyEnum {
            A,
            #[default]
            B,
            #[serde(skip)]
            C,
        }
    }

    assert_eq!(MyEnum::default(), MyEnum::B);
    assert!(serde_json::to_string(&MyEnum::C).is_err());
}

#[test]
fn nested_items() {
    // This has to be in a module (not a function) because of Rust import rules
    mod nested {
        alloy_sol_types::sol! {
            #[derive(Debug, PartialEq)]
            struct FilAddress {
                bytes data;
            }

            #[derive(Debug, PartialEq)]
            struct BigInt {
                bytes val;
                bool neg;
            }

            #[derive(Debug, PartialEq)]
            interface InterfaceTest {
                function f1(FilAddress memory fAddress, uint256 value) public payable;

                function f2(BigInt memory b) public returns (BigInt memory);
            }
        }
    }
    use nested::{InterfaceTest::*, *};

    let _ = FilAddress { data: bytes![] };
    let _ = BigInt { val: bytes![], neg: false };
    assert_eq!(f1Call::SIGNATURE, "f1((bytes),uint256)");
    assert_eq!(f2Call::SIGNATURE, "f2((bytes,bool))");
}

// Allow enums as fields of structs
// https://github.com/alloy-rs/core/issues/319
#[test]
fn enum_field_of_struct() {
    sol! {
        enum MyEnum {
            FIRST,
            SECOND
        }

        struct MyStruct {
            MyEnum myOption;
            uint value;
        }
    }

    let _ = MyStruct { myOption: MyEnum::FIRST, value: U256::ZERO };
}

#[test]
fn same_names_different_namespaces() {
    sol! {
        library RouterErrors {
            error ReturnAmountIsNotEnough();
            error InvalidMsgValue();
            error ERC20TransferFailed();
        }

        library Errors {
            error InvalidMsgValue();
            error ETHTransferFailed();
        }
    }

    assert_ne!(
        std::any::TypeId::of::<RouterErrors::InvalidMsgValue>(),
        std::any::TypeId::of::<Errors::InvalidMsgValue>(),
    );
}

#[test]
fn rust_keywords() {
    sol! {
        contract dyn {
            struct const {
                bool unsafe;
                bytes32 box;
            }

            function mod(address impl, address some) returns (bool is, bool fn);
        }
    }
    use r#dyn::*;

    let _ = r#const { r#unsafe: true, r#box: Default::default() };
    let m = modCall { r#impl: Address::ZERO, some: Address::ZERO };
    let _ = dynCalls::r#mod(m);
    let _ = modReturn { is: true, r#fn: false };
    assert_eq!(r#const::NAME, "const");
    assert_eq!(modCall::SIGNATURE, "mod(address,address)");
}

#[test]
fn most_rust_keywords() {
    // $(kw r#kw)*
    macro_rules! kws {
        ($($kw:tt $raw:tt)*) => { paste::paste! {
            $({
                sol! {
                    struct $kw {
                        uint $kw;
                    }

                    function $kw(bytes1 $kw) returns (uint $kw);
                }

                mod error {
                    use super::*;

                    sol! {
                        error $kw(bytes2 $kw);
                    }
                }

                mod event {
                    use super::*;

                    sol! {
                        event $kw(bytes3 $kw);
                    }
                }

                // Special cased, signatures will be different.
                let kw = match stringify!($kw) {
                    "self" => "this",
                    "Self" => "This",
                    kw => kw,
                };
                assert_eq!($raw::NAME, kw);
                assert_eq!(<[<$raw Call>]>::SIGNATURE, format!("{kw}(bytes1)"));
                let _ = [<$raw Call>] { $raw: [0u8; 1].into() };
                assert_eq!(error::$raw::SIGNATURE, format!("{kw}(bytes2)"));
                let _ = error::$raw { $raw: [0u8; 2].into() };
                assert_eq!(event::$raw::SIGNATURE, format!("{kw}(bytes3)"));
                let _ = event::$raw { $raw: [0u8; 3].into() };
            })*
        } };
    }

    kws! {
        // Special cased: https://github.com/alloy-rs/core/issues/902
        self this
        Self This

        const r#const
        extern r#extern
        fn r#fn
        impl r#impl
        loop r#loop
        mod r#mod
        move r#move
        mut r#mut
        pub r#pub
        ref r#ref
        trait r#trait
        unsafe r#unsafe
        use r#use
        where r#where
        async r#async
        await r#await
        dyn r#dyn
        become r#become
        box r#box
        priv r#priv
        unsized r#unsized
        yield r#yield
    }
}

#[test]
fn raw_identifiers() {
    sol! {
        struct r#mod {
            int r#type;
        }
        function r#try();
    }
    let _ = r#mod { r#type: Default::default() };
    let _ = tryCall {};
    assert_eq!(r#mod::NAME, "mod");
    assert_eq!(tryCall::SIGNATURE, "try()");
}

// Translate contract types to `address`
// https://github.com/alloy-rs/core/issues/347
#[test]
fn contract_type() {
    sol! {
        interface IERC20 {}
        function func(IERC20 addr);
        struct Struct {
            IERC20 addr;
        }
    }
}

// Correctly identify whether a type is dynamic
// https://github.com/alloy-rs/core/issues/352
#[test]
fn word_dynarray_event() {
    sol! {
        event Dynamic1(string[] indexed);
        event Dynamic2(string[] indexed, bytes[] indexed);

        event Word1(address[] indexed);
        event Word2(address[] indexed, bytes32[] indexed);
        event Word3(address[] indexed, bytes32[] indexed, uint256[] indexed);
    }
}

#[test]
fn string_indexed_event() {
    sol! {
        event S1(string);
        event S2(uint, string);
        event S3(string, uint);
        event S4(uint[], string);
        event S5(uint[], string, uint[]);
        event S6(uint[] indexed, string, uint[]);
        event S7(uint[] indexed, string, uint[] indexed);

        event SI1(string indexed);
        event SI2(uint, string indexed);
        event SI3(string indexed, uint);
        event SI4(uint[], string indexed);
        event SI5(uint[], string indexed, uint[]);
        event SI6(uint[] indexed, string indexed, uint[]);
        event SI7(uint[] indexed, string indexed, uint[] indexed);
    }
}

// TODO: make commented out code work
#[test]
fn paths_resolution_1() {
    sol! {
        // library OrderRFQLib {
            struct OrderRFQ {
                uint256 info;
                address makerAsset;
                address takerAsset;
                address maker;
                address allowedSender;
                uint256 makingAmount;
                uint256 takingAmount;
            }
        // }

        function fillOrderRFQ(
            /*OrderRFQLib.*/OrderRFQ memory order,
            bytes calldata signature,
            uint256 flagsAndAmount
        ) external payable returns(uint256, uint256, bytes32) {
            return fillOrderRFQTo(order, signature, flagsAndAmount, msg.sender);
        }
    }
}

// Correctly expand the `tokenize` function statements for events
// https://github.com/alloy-rs/core/issues/361
#[test]
fn event_tokenize_fields() {
    sol! {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
    let _ = PairCreated {
        token0: Address::ZERO,
        token1: Address::ZERO,
        pair: Address::ZERO,
        _3: U256::ZERO,
    };
}

// Allow multiple overrides of the same function
// https://github.com/alloy-rs/core/issues/398
#[test]
fn duplicate_attributes() {
    sol! {
        contract TaxableTeamToken is IERC20, Context, Ownable {
            constructor(
                string memory name,
                string memory symbol,
                uint8 decimals,
                uint256 supply,
                uint256 fees,
                address owner,
                address feeWallet
            ) public checkIsFeesValid(fees) checkIsFeesValid(fees2) checkIsAddressValid(owner) checkIsAddressValid(feeWallet) {
                require(decimals >=8 && decimals <= 18, "[Validation] Not valid decimals");
                require(supply > 0, "[Validation] initial supply should be greater than 0");
                require(owner != feeWallet, "[Validation] fee wallet and owner wallet cannot be same.");

                _name = name;
                _symbol = symbol;
                _decimals = decimals;
                _feesPercentage = fees;

                _tTotal = supply;
                _rTotal = (MAX - (MAX % _tTotal));

                _rOwned[owner] = _rTotal;

                emit Transfer(address(0), owner, _tTotal);

                emit TeamFinanceTokenMint(owner);
            }
        }
    }
}

#[test]
fn duplicate_events() {
    sol! {
    #[derive(derive_more::Display)]
    interface Console {
        #[display("{val}")]
        event log(string val);

        #[display("{}", "hex::encode_prefixed(val)")]
        event logs(bytes val);

        #[display("{val}")]
        event log_address(address val);

        #[display("{val}")]
        event log_bytes32(bytes32 val);

        #[display("{val}")]
        event log_int(int val);

        #[display("{val}")]
        event log_uint(uint val);

        #[display("{}", "hex::encode_prefixed(val)")]
        event log_bytes(bytes val);

        #[display("{val}")]
        event log_string(string val);

        #[display("{val:?}")]
        event log_array(uint256[] val);

        #[display("{val:?}")]
        event log_array(int256[] val);

        #[display("{val:?}")]
        event log_array(address[] val);

        #[display("{key}: {val}")]
        event log_named_address(string key, address val);

        #[display("{key}: {val}")]
        event log_named_bytes32(string key, bytes32 val);

        #[display("{key}: {val}")]
        event log_named_decimal_int(string key, int val, uint decimals);

        #[display("{key}: {val}")]
        event log_named_decimal_uint(string key, uint val, uint decimals);

        #[display("{key}: {val}")]
        event log_named_int(string key, int val);

        #[display("{key}: {val}")]
        event log_named_uint(string key, uint val);

        #[display("{key}: {val:?}")]
        event log_named_bytes(string key, bytes val);

        #[display("{key}: {val}")]
        event log_named_string(string key, string val);

        #[display("{key}: {val:?}")]
        event log_named_array(string key, uint256[] val);

        #[display("{key}: {val:?}")]
        event log_named_array(string key, int256[] val);

        #[display("{key}: {val:?}")]
        event log_named_array(string key, address[] val);
    }
    }
}

// https://github.com/alloy-rs/core/issues/433
#[test]
fn decoder_fixed_array_before_dynamic() {
    sol! {
        #[derive(Debug, PartialEq, Eq)]
        struct FullReport {
            bytes32[3] report_context;
            bytes      report_blob;
            bytes32[]  raw_rs;
            bytes32[]  raw_ss;
            bytes32    raw_vs;
        }
    }
    let full_report = FullReport {
        report_context: [
            b256!("0x0006015a2de20abc8c880eb052a09c069e4edf697529d12eeae88b7b6867fc81"),
            b256!("0x00000000000000000000000000000000000000000000000000000000080f7906"),
            b256!("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ],
        report_blob: hex!("0002191c50b7bdaf2cb8672453141946eea123f8baeaa8d2afa4194b6955e68300000000000000000000000000000000000000000000000000000000655ac7af00000000000000000000000000000000000000000000000000000000655ac7af000000000000000000000000000000000000000000000000000000000000138800000000000000000000000000000000000000000000000000000000000a1f6800000000000000000000000000000000000000000000000000000000655c192f000000000000000000000000000000000000000000000000d130d9ecefeaae30").into(),
        raw_rs: vec![
            b256!("0xd1e3d8b8c581a7ed9cfc41316f1bb8598d98237fc8278a01a9c6a323c4b5c331"),
            b256!("0x38ef50778560ec2bb08b23960e3d74f1ffe83b9240a39555c6eb817e3f68302c"),
        ],
        raw_ss: vec![
            b256!("0x7fb9c59cc499a4672f1481a526d01aa8c01380dcfa0ea855041254d3bcf45536"),
            b256!("0x2ce612a86846a7cbb640ddcd3abdecf56618c7b24cf96242643d5c355dee5f0e"),
        ],
        raw_vs: b256!("0x0001000000000000000000000000000000000000000000000000000000000000"),
    };

    let encoded = FullReport::abi_encode(&full_report);
    let expected = hex!("00000000000000000000000000000000000000000000000000000000000000200006015a2de20abc8c880eb052a09c069e4edf697529d12eeae88b7b6867fc8100000000000000000000000000000000000000000000000000000000080f7906000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000000240000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e00002191c50b7bdaf2cb8672453141946eea123f8baeaa8d2afa4194b6955e68300000000000000000000000000000000000000000000000000000000655ac7af00000000000000000000000000000000000000000000000000000000655ac7af000000000000000000000000000000000000000000000000000000000000138800000000000000000000000000000000000000000000000000000000000a1f6800000000000000000000000000000000000000000000000000000000655c192f000000000000000000000000000000000000000000000000d130d9ecefeaae300000000000000000000000000000000000000000000000000000000000000002d1e3d8b8c581a7ed9cfc41316f1bb8598d98237fc8278a01a9c6a323c4b5c33138ef50778560ec2bb08b23960e3d74f1ffe83b9240a39555c6eb817e3f68302c00000000000000000000000000000000000000000000000000000000000000027fb9c59cc499a4672f1481a526d01aa8c01380dcfa0ea855041254d3bcf455362ce612a86846a7cbb640ddcd3abdecf56618c7b24cf96242643d5c355dee5f0e");
    assert_eq!(hex::encode(&encoded), hex::encode(expected));

    let decoded = FullReport::abi_decode(&encoded).unwrap();
    assert_eq!(decoded, full_report);
}

#[test]
fn bytecode_attributes() {
    sol! {
        #[sol(bytecode = "1234", deployed_bytecode = "0x5678")]
        contract Dummy {}
    }

    assert_eq!(Dummy::BYTECODE[..], hex::decode("1234").unwrap());
    assert_eq!(Dummy::DEPLOYED_BYTECODE[..], hex::decode("5678").unwrap());
}

#[test]
fn function_overrides() {
    mod one {
        alloy_sol_types::sol! {
            function testFunction(bytes32 one);
        }
    }

    mod two {
        alloy_sol_types::sol! {
            function testFunction(bytes32 one);
            function testFunction(bytes32 one, bytes32 two);
        }
    }

    assert_eq!(one::testFunctionCall::SIGNATURE, "testFunction(bytes32)");
    assert_eq!(one::testFunctionCall::SIGNATURE, two::testFunction_0Call::SIGNATURE);

    assert_eq!(two::testFunction_1Call::SIGNATURE, "testFunction(bytes32,bytes32)");
}

#[test]
fn error_overrides() {
    mod one {
        alloy_sol_types::sol! {
            error TestError(bytes32 one);
        }
    }

    mod two {
        alloy_sol_types::sol! {
            error TestError(bytes32 one);
            error TestError(bytes32 one, bytes32 two);
        }
    }

    assert_eq!(one::TestError::SIGNATURE, "TestError(bytes32)");
    assert_eq!(one::TestError::SIGNATURE, two::TestError_0::SIGNATURE);

    assert_eq!(two::TestError_1::SIGNATURE, "TestError(bytes32,bytes32)");
}

// https://github.com/alloy-rs/core/issues/640
#[test]
fn event_overrides() {
    mod one {
        alloy_sol_types::sol! {
            event TestEvent(bytes32 indexed one);
        }
    }

    mod two {
        alloy_sol_types::sol! {
            event TestEvent(bytes32 indexed one);
            event TestEvent(bytes32 indexed one, bytes32 indexed two);
        }
    }

    assert_eq!(one::TestEvent::SIGNATURE, "TestEvent(bytes32)");
    assert_eq!(one::TestEvent::SIGNATURE, two::TestEvent_0::SIGNATURE);
    assert_eq!(one::TestEvent::SIGNATURE_HASH, keccak256("TestEvent(bytes32)"));
    assert_eq!(one::TestEvent::SIGNATURE_HASH, two::TestEvent_0::SIGNATURE_HASH);

    assert_eq!(two::TestEvent_1::SIGNATURE, "TestEvent(bytes32,bytes32)");
    assert_eq!(two::TestEvent_1::SIGNATURE_HASH, keccak256("TestEvent(bytes32,bytes32)"));
}

#[test]
fn contract_derive_default() {
    sol! {
        #[derive(Debug, Default)]
        contract MyContract {
            function f1(address);
            function f2(address b);
            event e1();
            event e2();
            error c();
        }
    }

    let MyContract::f1Call(_) = MyContract::f1Call::default();
    let MyContract::f2Call { b: _ } = MyContract::f2Call::default();
    #[allow(clippy::default_constructed_unit_structs)]
    let MyContract::e1 = MyContract::e1::default();
    #[allow(clippy::default_constructed_unit_structs)]
    let MyContract::e2 = MyContract::e2::default();
    #[allow(clippy::default_constructed_unit_structs)]
    let MyContract::c {} = MyContract::c::default();
}

#[test]
fn contract_namespaces() {
    mod inner {
        alloy_sol_types::sol! {
            library LibA {
                struct Struct {
                    uint64 field64;
                }
            }

            library LibB {
                struct Struct {
                    uint128 field128;
                }
            }

            contract Contract {
                LibA.Struct internal aValue;
                LibB.Struct internal bValue;

                constructor(
                    LibA.Struct memory aValue_,
                    LibB.Struct memory bValue_
                )
                {
                    aValue = aValue_;
                    bValue = bValue_;
                }

                function fn(
                    LibA.Struct memory aValue_,
                    LibB.Struct memory bValue_
                ) public
                {
                    aValue = aValue_;
                    bValue = bValue_;
                }
            }
        }
    }

    let _ = inner::Contract::fnCall {
        aValue_: inner::LibA::Struct { field64: 0 },
        bValue_: inner::LibB::Struct { field128: 0 },
    };
    assert_eq!(inner::Contract::fnCall::SIGNATURE, "fn((uint64),(uint128))");
}

// https://github.com/alloy-rs/core/pull/694#issuecomment-2274263880
#[test]
fn regression_overloads() {
    sol! {
        contract Vm {
            struct Wallet {
                uint stuff;
            }

            /// Gets the nonce of an account.
            function getNonce(address account) external view returns (uint64 nonce);

            /// Get the nonce of a `Wallet`.
            function getNonce(Wallet calldata wallet) external returns (uint64 nonce);
        }
    }

    let _ = Vm::getNonce_0Call { account: Address::ZERO };
    let _ = Vm::getNonce_0Return { nonce: 0 };
    assert_eq!(Vm::getNonce_0Call::SIGNATURE, "getNonce(address)");

    let _ = Vm::getNonce_1Call { wallet: Vm::Wallet { stuff: U256::ZERO } };
    let _ = Vm::getNonce_1Return { nonce: 0 };
    assert_eq!(Vm::getNonce_1Call::SIGNATURE, "getNonce((uint256))");
}

#[test]
fn normal_paths() {
    sol! {
        interface I {
            struct S {
                uint x;
            }
        }
        function func(I.S memory stuff);
    }

    let _ = funcCall { stuff: I::S { x: U256::ZERO } };
}

#[test]
fn regression_nested_namespaced_structs() {
    mod inner {
        super::sol! {
            library LibA {
                struct Simple {
                    uint256 x;
                }

                struct Nested {
                    Simple simple;
                    LibB.Simple[] simpleB;
                }
            }

            library LibB {
                struct Simple {
                    uint256 x;
                    uint256 y;
                }

                struct Nested {
                    Simple simple;
                    LibA.Simple simpleA;
                    LibB.Simple simpleB;
                }
            }

            library LibC {
                struct Nested1 {
                    LibA.Nested nestedA;
                    LibB.Nested nestedB;
                }

                struct Nested2 {
                    LibA.Simple simpleA;
                    LibB.Simple simpleB;
                    LibA.Nested[] nestedA;
                    LibB.Nested nestedB;
                    Nested1[] nestedC1;
                    LibC.Nested1 nestedC2;
                }
            }

            contract C {
                function libASimple(LibA.Simple memory simple) public returns(LibA.Simple memory);
                function libBSimple(LibB.Simple memory simple) public returns(LibB.Simple memory);
                function libANested(LibA.Nested memory nested) public returns(LibA.Nested memory);
                function libBNested(LibB.Nested memory nested) public returns(LibB.Nested memory);
                function libCNested1(LibC.Nested1 memory nested) public returns(LibC.Nested1 memory);
                function libCNested2(LibC.Nested2 memory nested) public returns(LibC.Nested2 memory);
            }
        }
    }

    let a_simple = "(uint256)";
    let b_simple = "(uint256,uint256)";
    let a_nested = format!("({a_simple},{b_simple}[])");
    let b_nested = format!("({b_simple},{a_simple},{b_simple})");
    let c_nested1 = format!("({a_nested},{b_nested})");
    let c_nested2 =
        format!("({a_simple},{b_simple},{a_nested}[],{b_nested},{c_nested1}[],{c_nested1})");

    assert_eq!(inner::C::libASimpleCall::SIGNATURE, format!("libASimple({a_simple})"));
    assert_eq!(inner::C::libBSimpleCall::SIGNATURE, format!("libBSimple({b_simple})"));
    assert_eq!(inner::C::libANestedCall::SIGNATURE, format!("libANested({a_nested})"));
    assert_eq!(inner::C::libBNestedCall::SIGNATURE, format!("libBNested({b_nested})"));
    assert_eq!(inner::C::libCNested1Call::SIGNATURE, format!("libCNested1({c_nested1})"));
    assert_eq!(inner::C::libCNested2Call::SIGNATURE, format!("libCNested2({c_nested2})"));
}

// https://github.com/alloy-rs/core/issues/734
#[test]
fn event_indexed_udvt() {
    use alloy_primitives::aliases::*;

    sol! {
        type Currency is address;
        type PoolId is bytes32;

        event Initialize(
            PoolId indexed id,
            Currency indexed currency0,
            Currency indexed currency1,
            uint24 fee,
            int24 tickSpacing,
            address hooks,
            uint160 sqrtPriceX96,
            int24 tick
        );
    }

    assert_eq!(
        Initialize::SIGNATURE,
        "Initialize(bytes32,address,address,uint24,int24,address,uint160,int24)",
    );
    assert_eq!(
        Initialize::SIGNATURE_HASH,
        b256!("0xdd466e674ea557f56295e2d0218a125ea4b4f0f6f3307b95f85e6110838d6438"),
    );

    let _ = Initialize {
        id: B256::ZERO,
        currency0: Address::ZERO,
        currency1: Address::ZERO,
        fee: U24::ZERO,
        tickSpacing: I24::ZERO,
        hooks: Address::ZERO,
        sqrtPriceX96: U160::ZERO,
        tick: I24::ZERO,
    };
}

#[test]
fn event_indexed_elementary_arrays() {
    sol! {
        event AddrArray(address[1] indexed x);
        event AddrDynArray(address[] indexed x);

        type MyAddress is address;
        event AddrUdvtArray(MyAddress[1] indexed y);
        event AddrUdvtDynArray(MyAddress[] indexed y);
    }

    assert_eq!(AddrArray::SIGNATURE, "AddrArray(address[1])");
    let _ = AddrArray { x: B256::ZERO };
    assert_eq!(AddrDynArray::SIGNATURE, "AddrDynArray(address[])");
    let _ = AddrDynArray { x: B256::ZERO };

    assert_eq!(AddrUdvtArray::SIGNATURE, "AddrUdvtArray(address[1])");
    let _ = AddrUdvtArray { y: B256::ZERO };
    assert_eq!(AddrUdvtDynArray::SIGNATURE, "AddrUdvtDynArray(address[])");
    let _ = AddrUdvtDynArray { y: B256::ZERO };
}

// https://github.com/alloy-rs/core/issues/589
#[test]
#[allow(clippy::assertions_on_constants)]
fn event_check_signature() {
    sol! {
        #[derive(Debug)]
        event MyEvent();
        event MyEventAnonymous() anonymous;
    }

    let no_topics: [B256; 0] = [];

    assert!(!MyEvent::ANONYMOUS);
    let e = MyEvent::decode_raw_log(no_topics, &[]).unwrap_err();
    assert_eq!(e.to_string(), "topic list length mismatch");
    let e = MyEvent::decode_raw_log([B256::ZERO], &[]).unwrap_err();
    assert!(e.to_string().contains("invalid signature hash"), "{e:?}");
    let MyEvent {} = MyEvent::decode_raw_log([MyEvent::SIGNATURE_HASH], &[]).unwrap();

    assert!(MyEventAnonymous::ANONYMOUS);
    let MyEventAnonymous {} = MyEventAnonymous::decode_raw_log(no_topics, &[]).unwrap();
}

// https://github.com/alloy-rs/core/issues/811
#[test]
fn mapping_getters() {
    sol! {
        #![sol(all_derives)]

        contract TestIbc {
            /// ConnectionId -> Connection
            mapping(uint32 => Connection) public connections;
            /// ChannelId -> Channel
            mapping(uint32 => Channel) public channels;

            enum ConnectionState {
                Unspecified,
                Init,
                TryOpen,
                Open
            }

            struct Connection {
                ConnectionState state;
                uint32 client_id;
                uint32 counterparty_client_id;
                uint32 counterparty_connection_id;
            }

            enum ChannelState {
                Unspecified,
                Init,
                TryOpen,
                Open,
                Closed
            }

            struct Channel {
                ChannelState state;
                uint32 connection_id;
                uint32 counterparty_channel_id;
                bytes counterparty_port_id;
                string version;
            }
        }
    }

    assert_eq!(TestIbc::connectionsCall::SIGNATURE, "connections(uint32)");
    let _ = TestIbc::connectionsReturn { _0: 0u8, _1: 0u32, _2: 0u32, _3: 0u32 };

    assert_eq!(TestIbc::channelsCall::SIGNATURE, "channels(uint32)");
    let _ =
        TestIbc::channelsReturn { _0: 0u8, _1: 0u32, _2: 0u32, _3: bytes![], _4: String::new() };
}

// https://github.com/alloy-rs/core/issues/829
#[test]
fn bytes64() {
    sol! {
        struct bytes64 {
            bytes32 a;
            bytes32 b;
        }

        function f(bytes64 x) public returns(bytes64 y) {
            bytes64 z = x;
            y = z;
        }
    }

    let x = bytes64 { a: B256::ZERO, b: B256::ZERO };
    assert_eq!(bytes64::abi_encode_packed(&x), alloy_primitives::B512::ZERO.as_slice());
}

#[test]
fn array_sizes() {
    sol! {
        uint constant x = 1;
        uint constant y = x + 1;

        contract C {
            uint constant z = y * 2;

            struct S {
                uint[x] a;
                uint[y] b;
                uint[z] c;
                uint[z * 2] d;
            }

            function f(S memory s);
        }
    }

    assert_eq!(C::fCall::SIGNATURE, "f((uint256[1],uint256[2],uint256[4],uint256[8]))");
}

#[test]
fn extra_derives() {
    sol! {
        #![sol(extra_derives(std::fmt::Debug))]

        struct MyStruct {
            uint256 a;
        }
    }

    let s = MyStruct { a: U256::ZERO };
    let _ = format!("{s:#?}");
}
