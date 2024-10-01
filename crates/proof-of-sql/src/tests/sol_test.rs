use crate::tests::{ForgeScript, ForgeScriptError};
use alloy_sol_types::{private::primitives::U256, sol};

#[test]
#[ignore = "Because forge needs to be installed, we are ignoring this test by default. They will still be run from within the ci."]
fn we_can_run_solidity_script_from_rust() {
    ForgeScript::new(
        "./sol_src/tests/TestScript.t.sol",
        "rustTestWeCanThrowErrorDependingOnParameter",
    )
    .arg(U256::from(1234))
    .execute()
    .unwrap();

    assert!(matches!(
        ForgeScript::new(
            "./sol_src/tests/TestScript.t.sol",
            "rustTestWeCanThrowErrorDependingOnParameter",
        )
        .arg(U256::from(0))
        .execute(),
        Err(ForgeScriptError::SolidityError { .. })
    ));
}
#[test]
#[ignore = "Because forge needs to be installed, we are ignoring this test by default. They will still be run from within the ci."]
fn we_can_pass_custom_struct_into_solidity_from_rust() {
    sol!("./sol_src/tests/TestScript.t.sol");
    let arg = TestScript::CustomStruct {
        value: U256::from(1234),
    };
    ForgeScript::new(
        "./sol_src/tests/TestScript.t.sol",
        "rustTestWeCanAcceptCustomStructAsEncodedBytes",
    )
    .arg(arg)
    .execute()
    .unwrap();
}
