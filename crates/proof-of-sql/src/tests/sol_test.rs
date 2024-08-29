use alloy_primitives::{Bytes, U256};
use alloy_sol_types::{sol, SolValue};
use forge_script::ScriptArgs;

#[tokio::test(flavor = "multi_thread")]
async fn we_can_run_solidity_script_from_rust() {
    ScriptArgs {
        path: "./src/tests/TestScript.t.sol".to_string(),
        sig: "rustTestWeCanThrowErrorDependingOnParameter".to_string(),
        args: vec![U256::from(1234).to_string()],
        ..Default::default()
    }
    .run_script()
    .await
    .unwrap();

    assert!(ScriptArgs {
        path: "./src/tests/TestScript.t.sol".to_string(),
        sig: "rustTestWeCanThrowErrorDependingOnParameter".to_string(),
        args: vec![U256::from(0).to_string()],
        ..Default::default()
    }
    .run_script()
    .await
    .is_err());
}
#[tokio::test(flavor = "multi_thread")]
async fn we_can_pass_custom_struct_into_solidity_from_rust() {
    sol!("./src/tests/TestScript.t.sol");
    let arg = TestScript::CustomStruct {
        value: U256::from(1234),
    };
    ScriptArgs {
        path: "./src/tests/TestScript.t.sol".to_string(),
        sig: "rustTestWeCanDecodeCustomStruct".to_string(),
        args: vec![Bytes::from(arg.abi_encode()).to_string()],
        ..Default::default()
    }
    .run_script()
    .await
    .unwrap();
}
