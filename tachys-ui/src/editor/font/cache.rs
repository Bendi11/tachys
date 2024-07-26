
use allsorts::{font_data::DynamicFontTableProvider, Font};
use slotmap::SlotMap;


struct CachedFont<'s> {
    font: Font<DynamicFontTableProvider<'s>>,
}

slotmap::new_key_type! { pub struct FontId; }

pub struct FontCache<'s> {
    loaded: SlotMap<FontId, CachedFont<'s>>,
}

impl<'s> FontCache<'s> {

}
