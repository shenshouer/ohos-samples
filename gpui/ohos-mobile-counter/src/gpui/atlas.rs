//! A small, owned monochrome atlas. Each tile stores its own alpha mask.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::gpui_core::{
    AtlasKey, AtlasTextureId, AtlasTextureKind, AtlasTile, Bounds, DevicePixels, PlatformAtlas,
    Result, Size, TileId, point, size,
};

#[derive(Clone)]
pub(crate) struct Mask {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) alpha: Vec<u8>,
}

#[derive(Default)]
struct Storage {
    by_key: HashMap<AtlasKey, AtlasTile>,
    masks: HashMap<u32, Mask>,
    next_id: u32,
}

#[derive(Default)]
pub(crate) struct MonoAtlas(Mutex<Storage>);

impl MonoAtlas {
    pub(crate) fn mask(&self, tile: AtlasTile) -> Result<Mask> {
        if tile.texture_id.kind != AtlasTextureKind::Monochrome || tile.texture_id.index != 0 {
            return Err(crate::gpui_core::private::anyhow::anyhow!(
                "unsupported atlas texture"
            ));
        }
        self.0
            .lock()
            .map_err(|_| crate::gpui_core::private::anyhow::anyhow!("atlas lock poisoned"))?
            .masks
            .get(&tile.tile_id.0)
            .cloned()
            .ok_or_else(|| crate::gpui_core::private::anyhow::anyhow!("atlas tile missing"))
    }
}

impl PlatformAtlas for MonoAtlas {
    fn get_or_insert_with<'a>(
        &self,
        key: AtlasKey,
        build: &mut dyn FnMut() -> Result<Option<(Size<DevicePixels>, Cow<'a, [u8]>)>>,
    ) -> Result<Option<AtlasTile>> {
        if key.texture_kind() != AtlasTextureKind::Monochrome {
            return Err(crate::gpui_core::private::anyhow::anyhow!(
                "only monochrome atlas tiles are supported in M3-2"
            ));
        }
        // GPUI's atlas contract calls the builder while holding its lock.
        let mut storage = self
            .0
            .lock()
            .map_err(|_| crate::gpui_core::private::anyhow::anyhow!("atlas lock poisoned"))?;
        if let Some(tile) = storage.by_key.get(&key) {
            return Ok(Some(*tile));
        }
        let Some((dimensions, bytes)) = build()? else {
            return Ok(None);
        };
        let width = u32::try_from(dimensions.width.0)?;
        let height = u32::try_from(dimensions.height.0)?;
        let expected = usize::try_from(width)?
            .checked_mul(usize::try_from(height)?)
            .ok_or_else(|| crate::gpui_core::private::anyhow::anyhow!("atlas tile too large"))?;
        if width == 0 || height == 0 || expected > 4096 * 4096 || bytes.len() != expected {
            return Err(crate::gpui_core::private::anyhow::anyhow!(
                "invalid monochrome atlas mask dimensions or byte count"
            ));
        }
        let id = storage.next_id;
        storage.next_id = id.checked_add(1).ok_or_else(|| {
            crate::gpui_core::private::anyhow::anyhow!("atlas tile IDs exhausted")
        })?;
        let tile = AtlasTile {
            texture_id: AtlasTextureId {
                index: 0,
                kind: AtlasTextureKind::Monochrome,
            },
            tile_id: TileId(id),
            padding: 0,
            bounds: Bounds::new(
                point(DevicePixels(0), DevicePixels(0)),
                size(dimensions.width, dimensions.height),
            ),
        };
        storage.masks.insert(
            id,
            Mask {
                width,
                height,
                alpha: bytes.into_owned(),
            },
        );
        storage.by_key.insert(key, tile);
        Ok(Some(tile))
    }

    fn remove(&self, key: &AtlasKey) {
        if let Ok(mut storage) = self.0.lock()
            && let Some(tile) = storage.by_key.remove(key)
        {
            storage.masks.remove(&tile.tile_id.0);
        }
    }
}
