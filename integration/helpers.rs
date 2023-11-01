//! Defines testing mocks and helpers for integration tests

use std::fmt::Debug;

use ark_mpc::{
    algebra::{
        AuthenticatedPointResult, AuthenticatedScalarResult, MpcPointResult, MpcScalarResult,
        Scalar,
    },
    beaver::SharedValueSource,
    network::{NetworkPayload, PartyId},
    {MpcFabric, ResultHandle, ResultValue},
};
use futures::{future::join_all, Future};
use itertools::Itertools;
use tokio::runtime::Handle;

// -----------
// | Helpers |
// -----------

use crate::{IntegrationTestArgs, TestCurve, TestCurvePoint, TestScalar};

/// Compares two scalars, returning a result that can be propagated up an
/// integration test stack in the case that the scalars are not equal
pub(crate) fn assert_scalars_eq(a: TestScalar, b: TestScalar) -> Result<(), String> {
    if a == b {
        Ok(())
    } else {
        Err(format!("{a:?} != {b:?}"))
    }
}

/// Assert a batch of scalars equal one another
pub(crate) fn assert_scalar_batches_eq(
    a: Vec<TestScalar>,
    b: Vec<TestScalar>,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!("Lengths differ: {a:?} != {b:?}"));
    }

    for (a, b) in a.into_iter().zip(b.into_iter()) {
        assert_scalars_eq(a, b)?;
    }

    Ok(())
}

/// Compares two points, returning a result that can be propagated up an
/// integration test stack in the case that the points are not equal
pub(crate) fn assert_points_eq(a: TestCurvePoint, b: TestCurvePoint) -> Result<(), String> {
    if a == b {
        Ok(())
    } else {
        Err(format!("{a:?} != {b:?}"))
    }
}

/// Compares two batches of points
pub(crate) fn assert_point_batches_eq(
    a: Vec<TestCurvePoint>,
    b: Vec<TestCurvePoint>,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!("Lengths differ: {a:?} != {b:?}"));
    }

    for (a, b) in a.into_iter().zip(b.into_iter()) {
        assert_points_eq(a, b)?;
    }

    Ok(())
}

/// Assert that an error occurred during MPC execution
pub(crate) fn assert_err<T, E>(res: Result<T, E>) -> Result<(), String> {
    if res.is_err() {
        Ok(())
    } else {
        Err("Expected error, got Ok".to_string())
    }
}

/// Await a result in the computation graph by blocking the current task
pub(crate) fn await_result<R, T: Future<Output = R>>(res: T) -> R {
    Handle::current().block_on(res)
}

/// Await a batch of results
pub(crate) fn await_result_batch<R, T: Future<Output = R> + Clone>(res: &[T]) -> Vec<R> {
    res.iter()
        .map(|res| await_result(res.clone()))
        .collect_vec()
}

/// Await a result that may error
pub(crate) fn await_result_with_error<R, E: Debug, T: Future<Output = Result<R, E>>>(
    res: T,
) -> Result<R, String> {
    Handle::current()
        .block_on(res)
        .map_err(|err| format!("Error awaiting result: {:?}", err))
}

/// Await a batch of results that may error
pub(crate) fn await_batch_result_with_error<R, E, T>(res: Vec<T>) -> Result<Vec<R>, String>
where
    E: Debug,
    T: Future<Output = Result<R, E>>,
{
    Handle::current()
        .block_on(join_all(res))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Error awaiting result: {:?}", err))
}

/// Send or receive a secret shared scalar from the given party
pub(crate) fn share_scalar(
    value: TestScalar,
    sender: PartyId,
    test_args: &IntegrationTestArgs,
) -> MpcScalarResult<TestCurve> {
    let authenticated_value = test_args.fabric.share_scalar(value, sender);
    authenticated_value.mpc_share()
}

/// Share a batch of scalars
pub(crate) fn share_scalar_batch(
    values: Vec<TestScalar>,
    sender: PartyId,
    test_args: &IntegrationTestArgs,
) -> Vec<MpcScalarResult<TestCurve>> {
    test_args
        .fabric
        .batch_share_scalar(values, sender)
        .iter()
        .map(|v| v.mpc_share())
        .collect_vec()
}

/// Send or receive a secret shared point from the given party
pub(crate) fn share_point(
    value: TestCurvePoint,
    sender: PartyId,
    test_args: &IntegrationTestArgs,
) -> MpcPointResult<TestCurve> {
    // Share the point then cast to an `MpcPointResult`
    let authenticated_point = share_authenticated_point(value, sender, test_args);
    authenticated_point.mpc_share()
}

/// Share a batch of points
pub(crate) fn share_point_batch(
    values: Vec<TestCurvePoint>,
    sender: PartyId,
    test_args: &IntegrationTestArgs,
) -> Vec<MpcPointResult<TestCurve>> {
    values
        .into_iter()
        .map(|point| share_point(point, sender, test_args))
        .collect_vec()
}

/// Send or receive a secret shared scalar from the given party and allocate it
/// as an authenticated value
pub(crate) fn share_authenticated_scalar(
    value: TestScalar,
    sender: PartyId,
    test_args: &IntegrationTestArgs,
) -> AuthenticatedScalarResult<TestCurve> {
    test_args.fabric.share_scalar(value, sender)
}

/// Send or receive a batch of secret shared scalars from the given party and
/// allocate them as authenticated values
pub(crate) fn share_authenticated_scalar_batch(
    values: Vec<TestScalar>,
    sender: PartyId,
    test_args: &IntegrationTestArgs,
) -> Vec<AuthenticatedScalarResult<TestCurve>> {
    test_args.fabric.batch_share_scalar(values, sender)
}

/// Send or receive a secret shared point from the given party and allocate it
/// as an authenticated value
pub(crate) fn share_authenticated_point(
    value: TestCurvePoint,
    sender: PartyId,
    test_args: &IntegrationTestArgs,
) -> AuthenticatedPointResult<TestCurve> {
    test_args.fabric.share_point(value, sender)
}

/// Send or receive a batch of secret shared points from the given party and
/// allocate them as authenticated values
pub(crate) fn share_authenticated_point_batch(
    values: Vec<TestCurvePoint>,
    sender: PartyId,
    test_args: &IntegrationTestArgs,
) -> Vec<AuthenticatedPointResult<TestCurve>> {
    test_args.fabric.batch_share_point(values, sender)
}

/// Share a value with the counterparty by sender ID, the sender sends and the
/// receiver receives
pub(crate) fn share_plaintext_value<
    T: From<ResultValue<TestCurve>> + Into<NetworkPayload<TestCurve>>,
>(
    value: ResultHandle<TestCurve, T>,
    sender: PartyId,
    fabric: &MpcFabric<TestCurve>,
) -> ResultHandle<TestCurve, T> {
    if fabric.party_id() == sender {
        fabric.send_value(value)
    } else {
        fabric.receive_value()
    }
}

/// Share a batch of values in the plaintext
pub(crate) fn share_plaintext_values_batch<
    T: From<ResultValue<TestCurve>> + Into<NetworkPayload<TestCurve>> + Clone,
>(
    values: &[ResultHandle<TestCurve, T>],
    sender: PartyId,
    fabric: &MpcFabric<TestCurve>,
) -> Vec<ResultHandle<TestCurve, T>> {
    values
        .iter()
        .map(|v| share_plaintext_value(v.clone(), sender, fabric))
        .collect_vec()
}

// ---------
// | Mocks |
// ---------

/// Returns beaver triples (0, 0, 0) for party 0 and (1, 1, 1) for party 1
#[derive(Clone, Debug)]
pub(crate) struct PartyIDBeaverSource {
    party_id: u64,
}

impl PartyIDBeaverSource {
    pub fn new(party_id: u64) -> Self {
        Self { party_id }
    }
}

/// The PartyIDBeaverSource returns beaver triplets split statically between the
/// parties. We assume a = 2, b = 3 ==> c = 6. [a] = (1, 1); [b] = (3, 0) [c] =
/// (2, 4)
impl SharedValueSource<TestCurve> for PartyIDBeaverSource {
    fn next_shared_bit(&mut self) -> TestScalar {
        // Simply output partyID, assume partyID \in {0, 1}
        assert!(self.party_id == 0 || self.party_id == 1);
        Scalar::from(self.party_id)
    }

    fn next_triplet(&mut self) -> (TestScalar, TestScalar, TestScalar) {
        if self.party_id == 0 {
            (Scalar::from(1u64), Scalar::from(3u64), Scalar::from(2u64))
        } else {
            (Scalar::from(1u64), Scalar::from(0u64), Scalar::from(4u64))
        }
    }

    fn next_shared_inverse_pair(&mut self) -> (TestScalar, TestScalar) {
        (Scalar::from(1u8), Scalar::from(1u8))
    }

    fn next_shared_value(&mut self) -> TestScalar {
        Scalar::from(self.party_id)
    }
}
