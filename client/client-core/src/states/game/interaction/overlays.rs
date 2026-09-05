use crate::states::game::OverlayKind;
use crate::states::game::interaction::overlay::{Overlay, OverlayPayload};

#[derive(Default)]
pub struct Overlays {
    overlays: Vec<Overlay>,
}

impl Overlays {
    pub fn new() -> Self {
        Self {
            overlays: Vec::new(),
        }
    }

    pub fn get<T: OverlayPayload>(&self) -> Option<&T> {
        self.overlays.iter().rev().find_map(T::extract)
    }

    pub fn get_mut<T: OverlayPayload>(&mut self) -> Option<&mut T> {
        self.overlays.iter_mut().rev().find_map(T::extract_mut)
    }

    pub fn open(&mut self, overlay: Overlay) {
        let discriminant = overlay.discriminant();
        self.overlays.retain(|o| o.discriminant() != discriminant);
        self.overlays.push(overlay);
    }

    pub fn toggle(&mut self, overlay: Overlay) {
        let discriminant = overlay.discriminant();

        if self
            .overlays
            .iter()
            .any(|o| o.discriminant() == discriminant)
        {
            self.overlays.retain(|o| o.discriminant() != discriminant);
        } else {
            self.open(overlay);
        }
    }

    pub fn close<T: OverlayPayload>(&mut self) {
        self.overlays.retain(|o| T::extract(o).is_none());
    }

    pub fn close_top(&mut self) {
        self.overlays.pop();
    }

    pub fn close_all(&mut self) {
        self.overlays.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Overlay> {
        self.overlays.iter()
    }

    pub fn top_kind(&self) -> Option<OverlayKind> {
        self.overlays.last().map(Overlay::kind)
    }

    pub fn is_open<T: OverlayPayload>(&self) -> bool {
        self.get::<T>().is_some()
    }
}
