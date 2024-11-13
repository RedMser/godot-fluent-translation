use godot::prelude::*;
use std::ops::{Deref, DerefMut};

use godot::builtin::Callable;

pub struct SyncSendCallable(Callable);

unsafe impl Sync for SyncSendCallable {}
unsafe impl Send for SyncSendCallable {}

impl Deref for SyncSendCallable {
    type Target = Callable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SyncSendCallable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Debug for SyncSendCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl GodotConvert for SyncSendCallable {
    type Via = Callable;
}

impl FromGodot for SyncSendCallable {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        Callable::try_from_godot(via).map(SyncSendCallable)
    }
}

impl ToGodot for SyncSendCallable {
    fn to_godot(&self) -> Self::Via {
        self.0.to_godot()
    }
    
    type ToVia<'v> = Callable
    where
        Self: 'v;
}
