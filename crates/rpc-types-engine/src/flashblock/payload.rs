//! Flashblock payload types.

use super::{
    OpFlashblockExecutionPayloadBaseRef, OpFlashblockExecutionPayloadBaseV1,
    OpFlashblockExecutionPayloadDeltaRef, OpFlashblockExecutionPayloadDeltaV1,
    OpFlashblockMetadataRef, OpFlashblockMetadataV1,
};
use alloy_rpc_types_engine::PayloadId;

/// Flashblock payload version 1.
///
/// Represents a Flashblock, a real-time block-like structure emitted by the Base L2 chain.
/// A Flashblock provides a snapshot of a block's effects before finalization,
/// allowing faster insight into state transitions, balance changes, and logs.
///
/// See: [Base Flashblocks Documentation](https://docs.base.org/chain/flashblocks)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpFlashblockPayloadV1 {
    /// The unique payload ID as assigned by the execution engine for this block.
    pub payload_id: PayloadId,
    /// A sequential index that identifies the order of this Flashblock.
    pub index: u64,
    /// Immutable block properties shared across all flashblocks in the sequence.
    /// This is `None` for all flashblocks except the first in a sequence.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub base: Option<OpFlashblockExecutionPayloadBaseV1>,
    /// Accumulating and changing block properties for this flashblock.
    pub diff: OpFlashblockExecutionPayloadDeltaV1,
    /// Additional metadata about the flashblock such as receipts and balance changes.
    pub metadata: OpFlashblockMetadataV1,
}

impl From<OpFlashblockPayloadV1> for OpFlashblockPayload {
    fn from(payload: OpFlashblockPayloadV1) -> Self {
        Self::V1(payload)
    }
}

/// Flashblock payload envelope.
///
/// This enum allows for future versioning of flashblock payloads.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum OpFlashblockPayload {
    /// Version 1 flashblock payload.
    V1(OpFlashblockPayloadV1),
}

impl Default for OpFlashblockPayload {
    fn default() -> Self {
        Self::V1(Default::default())
    }
}

impl OpFlashblockPayload {
    /// Returns the payload ID.
    pub const fn payload_id(&self) -> &PayloadId {
        match self {
            Self::V1(payload) => &payload.payload_id,
        }
    }

    /// Returns the index.
    pub const fn index(&self) -> u64 {
        match self {
            Self::V1(payload) => payload.index,
        }
    }

    /// Returns a reference to the base execution payload, if present.
    ///
    /// The returned reference type implements [`Deref`](core::ops::Deref) for direct field access.
    pub fn base(&self) -> Option<OpFlashblockExecutionPayloadBaseRef<'_>> {
        match self {
            Self::V1(payload) => payload.base.as_ref().map(OpFlashblockExecutionPayloadBaseRef::V1),
        }
    }

    /// Returns a reference to the diff execution payload.
    ///
    /// The returned reference type implements [`Deref`](core::ops::Deref) for direct field access.
    pub const fn diff(&self) -> OpFlashblockExecutionPayloadDeltaRef<'_> {
        match self {
            Self::V1(payload) => OpFlashblockExecutionPayloadDeltaRef::V1(&payload.diff),
        }
    }

    /// Returns a reference to the metadata.
    ///
    /// The returned reference type implements [`Deref`](core::ops::Deref) for direct field access.
    pub const fn metadata(&self) -> OpFlashblockMetadataRef<'_> {
        match self {
            Self::V1(payload) => OpFlashblockMetadataRef::V1(&payload.metadata),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flashblock::{
        OpFlashblockExecutionPayloadBaseV1, OpFlashblockExecutionPayloadDeltaV1,
        OpFlashblockMetadataV1,
    };
    use alloc::collections::BTreeMap;
    use alloy_primitives::{B256, Bloom, Bytes, U256, address};

    fn sample_payload() -> OpFlashblockPayloadV1 {
        let base = OpFlashblockExecutionPayloadBaseV1 {
            parent_beacon_block_root: B256::ZERO,
            parent_hash: B256::ZERO,
            fee_recipient: address!("0000000000000000000000000000000000000001"),
            prev_randao: B256::ZERO,
            block_number: 100,
            gas_limit: 30_000_000,
            timestamp: 1234567890,
            extra_data: Bytes::default(),
            base_fee_per_gas: U256::from(1000000000u64),
        };

        let diff = OpFlashblockExecutionPayloadDeltaV1 {
            state_root: B256::ZERO,
            receipts_root: B256::ZERO,
            logs_bloom: Bloom::ZERO,
            gas_used: 21000,
            block_hash: B256::ZERO,
            transactions: vec![],
            withdrawals: vec![],
            withdrawals_root: B256::ZERO,
        };

        let metadata = OpFlashblockMetadataV1 {
            block_number: 100,
            new_account_balances: BTreeMap::new(),
            receipts: BTreeMap::new(),
        };

        OpFlashblockPayloadV1 {
            payload_id: PayloadId::new([1u8; 8]),
            index: 0,
            base: Some(base),
            diff,
            metadata,
        }
    }

    #[test]
    fn test_payload_accessors() {
        let payload = sample_payload();

        // Direct field access via public fields
        assert_eq!(payload.metadata.block_number, 100);
        assert_eq!(payload.base.as_ref().map(|b| b.parent_hash), Some(B256::ZERO));
        assert!(!payload.metadata.receipts.contains_key(&B256::ZERO));
    }

    #[test]
    fn test_payload_without_base() {
        let mut payload = sample_payload();
        payload.base = None;

        // Direct field access via public fields
        assert_eq!(payload.metadata.block_number, 100);
        assert_eq!(payload.base.as_ref().map(|b| b.parent_hash), None);
    }

    #[test]
    fn test_payload_enum() {
        let payload_v1 = sample_payload();
        let payload = OpFlashblockPayload::V1(payload_v1.clone());

        // Using enum accessor methods that return Ref types with Deref
        assert_eq!(payload.metadata().block_number, payload_v1.metadata.block_number);
        assert_eq!(
            payload.base().map(|b| b.parent_hash),
            payload_v1.base.as_ref().map(|b| b.parent_hash)
        );
        assert_eq!(payload.payload_id(), &payload_v1.payload_id);
        assert_eq!(payload.index(), payload_v1.index);
    }

    #[test]
    fn test_payload_serde_roundtrip() {
        let payload = sample_payload();

        let json = serde_json::to_string(&payload).unwrap();
        let decoded: OpFlashblockPayloadV1 = serde_json::from_str(&json).unwrap();
        assert_eq!(payload, decoded);
    }

    #[test]
    fn test_payload_snake_case_serialization() {
        let payload = sample_payload();

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("payload_id"));
        assert!(json.contains("\"index\""));
        assert!(json.contains("\"base\""));
        assert!(json.contains("\"diff\""));
        assert!(json.contains("\"metadata\""));
    }

    #[test]
    fn test_payload_enum_untagged() {
        let payload_v1 = sample_payload();
        let payload = OpFlashblockPayload::V1(payload_v1);

        let json = serde_json::to_string(&payload).unwrap();
        // Untagged means no "V1" key wrapper
        assert!(!json.contains("\"V1\""));

        let decoded: OpFlashblockPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload, decoded);
    }
}
