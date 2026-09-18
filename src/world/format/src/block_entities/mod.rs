mod sign;

pub use sign::{SignBlockEntity, SignText};

use crate::errors::WorldError;
use serde::{Deserialize, Serialize};
use temper_nbt::blob::NbtBlob;
use temper_nbt::{NBTSerializable, NBTSerializeOptions};

/// A block entity stored in a chunk. `protocol_id` comes from the blockstate
/// via `temper_data`'s generated `block_entity_type_for_state` at placement
/// time, so it stays correct across version bumps without this crate
/// depending on the block data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockEntityData {
    pub kind: BlockEntityKind,
    pub protocol_id: u16,
    pub blob: Vec<u8>,
}

/// A block entity type stored in a chunk. The variant determines how the
/// accompanying blob deserializes; the protocol ID for the wire comes from
/// the blockstate via `temper_data`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockEntityKind {
    Sign,
}

impl BlockEntityKind {
    /// Deserializes a stored blob and re-serializes it as network NBT for the wire.
    ///
    /// Blobs are JSON rather than bitcode like the rest of the chunk: `TextComponent`
    /// uses `#[serde(flatten)]`, which serializes as a map with no known length, and
    /// bitcode requires one. The blob is opaque to `Chunk` either way.
    pub fn to_network_nbt(self, blob: &[u8]) -> Result<NbtBlob, WorldError> {
        let mut buf = Vec::new();
        match self {
            Self::Sign => {
                let sign: SignBlockEntity = serde_json::from_slice(blob)
                    .map_err(|e| WorldError::BlockEntityDeserializeError(e.to_string()))?;
                NBTSerializable::serialize(&sign, &mut buf, &NBTSerializeOptions::Network);
            }
        }
        Ok(NbtBlob(buf))
    }
}
