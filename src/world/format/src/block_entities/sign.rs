use crate::{BlockEntityData, block_entities::BlockEntityKind};
use serde::{Deserialize, Serialize};
use temper_macros::NBTSerialize;
use temper_text::TextComponent;

use crate::errors::WorldError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, NBTSerialize)]
pub struct SignText {
    pub messages: Vec<TextComponent>,
    pub color: String,
    pub has_glowing_text: bool,
}

impl Default for SignText {
    fn default() -> Self {
        Self {
            messages: vec![TextComponent::default(); 4],
            color: "black".to_string(),
            has_glowing_text: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, NBTSerialize)]
pub struct SignBlockEntity {
    pub is_waxed: bool,
    pub front_text: SignText,
    pub back_text: SignText,
}

impl SignBlockEntity {
    pub fn to_blob(&self) -> Result<Vec<u8>, WorldError> {
        serde_json::to_vec(self).map_err(|e| WorldError::BlockEntitySerializeError(e.to_string()))
    }
}

impl BlockEntityData {
    /// Reads the stored blob as a sign, if this entry is one.
    pub fn as_sign(&self) -> Result<Option<SignBlockEntity>, WorldError> {
        if self.kind != BlockEntityKind::Sign {
            return Ok(None);
        }
        serde_json::from_slice(&self.blob)
            .map(Some)
            .map_err(|e| WorldError::BlockEntityDeserializeError(e.to_string()))
    }

    /// Replaces the stored blob with a sign's data.
    pub fn set_sign(&mut self, sign: &SignBlockEntity) -> Result<(), WorldError> {
        self.blob = sign.to_blob()?;
        Ok(())
    }
}
