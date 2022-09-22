//! Implements an authenticated wrapper around the MpcScalar type for malicious security

use curve25519_dalek::scalar::Scalar;

use crate::{network::MpcNetwork, mpc_scalar::MpcScalar, beaver::SharedValueSource, Visibility, SharedNetwork, BeaverSource, macros};


/// An authenticated scalar, wrapper around an MPC-capable Scalar that supports methods
/// to authenticate an opened result against a shared global MAC.
/// See SPDZ (https://eprint.iacr.org/2012/642.pdf) for a detailed explanation.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AuthenticatedScalar<N: MpcNetwork + Send, S: SharedValueSource<Scalar>> {
    /// The underlying MpcScalar that this structure authenticates 
    value: MpcScalar<N, S>,
    /// The local party's share of the value's MAC. If the value is `x`, then
    /// parties hold an additive share of \delta * x; where \delta is the
    /// shared MAC key
    mac_share: Option<MpcScalar<N, S>>,
    /// The local party's share of the global MAC key `\delta`. No party knows
    /// the cleartext key, only an additive share of the key.
    key_share: MpcScalar<N, S>,
    /// The visibility of the value within the network
    visibility: Visibility,
}

#[allow(unused_doc_comments)]
impl<N: MpcNetwork + Send, S: SharedValueSource<Scalar>> AuthenticatedScalar<N, S> {
    /// Create a new AuthenticatedScalar from a public u64 constant
    macros::impl_authenticated!(
        MpcScalar<N, S>, from_public_u64, from_private_u64, from_u64_with_visibility, u64
    );

    /// Create a new AuthenticatedScalar from a public Scalar constant
    macros::impl_authenticated!(
        MpcScalar<N, S>, from_public_scalar, from_private_scalar, from_scalar_with_visibility, Scalar
    );

    macros::impl_authenticated!(MpcScalar<N, S>, zero);
    macros::impl_authenticated!(MpcScalar<N, S>, one);
    macros::impl_authenticated!(MpcScalar<N, S>, default);

    macros::impl_authenticated!(
        MpcScalar<N, S>, 
        from_public_bytes_mod_order, 
        from_private_bytes_mod_order, 
        from_bytes_mod_order_with_visibility,
        [u8; 32]
    );

    macros::impl_authenticated!(
        MpcScalar<N, S>,
        from_bytes_mod_order_wide,
        from_public_bytes_mod_order_wide,
        from_bytes_mod_order_wide_with_visibility,
        &[u8; 64]
    );

    pub fn from_public_canonical_bytes_with_visibility(
        bytes: [u8; 32], visibility: Visibility, key_share: MpcScalar<N, S>, network: SharedNetwork<N>, beaver_source: BeaverSource<S>
    ) -> Option<Self> {
        let value = MpcScalar::<N, S>::from_canonical_bytes_with_visibility(bytes, Visibility::Public, network, beaver_source)?;

        Some(
            Self {
                value,
                visibility,
                mac_share: None,
                key_share,
            }
        )
    }

    macros::impl_authenticated!(
        MpcScalar<N, S>,
        from_public_bits,
        from_private_bits,
        from_bits_with_visibility,
        [u8; 32]
    );

    macros::impl_delegated!(to_bytes, self, [u8; 32]);
    macros::impl_delegated!(as_bytes, self, &[u8; 32]);
    macros::impl_delegated!(is_canonical, self, bool);
}
