//! Defines tests for the fabric directly

use curve25519_dalek::scalar::Scalar;
use mpc_ristretto::{fabric::ResultValue, network::PartyId, PARTY0, PARTY1};
use tokio::runtime::Handle;

use crate::{helpers::assert_scalars_eq, DefaultResHandle, IntegrationTest, IntegrationTestArgs};

// -----------
// | Helpers |
// -----------

/// Send or receive a value from the given party
fn send_receive_value(
    value: Scalar,
    sender: PartyId,
    test_args: &IntegrationTestArgs,
) -> DefaultResHandle {
    if test_args.party_id == sender {
        test_args
            .get_fabric_mut()
            .send_value(ResultValue::Scalars(vec![value]))
    } else {
        test_args.get_fabric_mut().receive_value()
    }
}

// ---------
// | Tests |
// ---------

/// Tests that sharing a value over the fabric works correctly
fn test_fabric_share_value(test_args: &IntegrationTestArgs) -> Result<(), String> {
    // Each party shares their party ID with the counterparty
    let my_party_id = Scalar::from(test_args.party_id);

    // Party 0
    let party0_value = send_receive_value(my_party_id, PARTY0, test_args);
    let party0_res = Handle::current().block_on(party0_value).to_scalar();

    assert_scalars_eq(party0_res, Scalar::zero())?;

    // Party 1
    let party1_value = send_receive_value(my_party_id, PARTY1, test_args);
    let party1_res = Handle::current().block_on(party1_value).to_scalar();

    assert_scalars_eq(party1_res, Scalar::one())
}

inventory::submit!(IntegrationTest {
    name: "fabric::test_fabric_share_value",
    test_fn: test_fabric_share_value,
});
