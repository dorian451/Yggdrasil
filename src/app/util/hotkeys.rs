use leptos::{ev::keydown, prelude::*};
use leptos_use::{use_document, use_event_listener};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Debug,
};
use tracing::{debug, info};
use web_sys::KeyboardEvent;

/// Describes a particular key and modifier combination to watch for
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct KeyCondition {
    keys: String,
    control: bool,
    alt: bool,
    shift: bool,
    meta: bool,
}

impl TryFrom<&str> for KeyCondition {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut key = KeyCondition::default();
        value
            .split("+")
            .try_for_each(|x| match x.to_lowercase().as_str() {
                "ctrl" => {
                    key.control = true;
                    Ok(())
                }
                "alt" => {
                    key.alt = true;
                    Ok(())
                }
                "shift" => {
                    key.shift = true;
                    Ok(())
                }
                "meta" => {
                    key.meta = true;
                    Ok(())
                }
                _ => {
                    if key.keys.is_empty() {
                        key.keys = x.to_string();
                        Ok(())
                    } else {
                        Err("key already set")
                    }
                }
            })?;

        Ok(key)
    }
}

impl From<&KeyboardEvent> for KeyCondition {
    fn from(value: &KeyboardEvent) -> Self {
        Self {
            keys: value.key(),
            control: value.ctrl_key(),
            alt: value.alt_key(),
            shift: value.shift_key(),
            meta: value.meta_key(),
        }
    }
}

/// State-keeping struct to store hotkeys
#[derive(Default)]
pub struct HotkeysState {
    hotkeys:
        HashMap</*Vec<*/ KeyCondition /*>*/, HashMap<usize, Callback<KeyboardEvent>>>,
}

impl HotkeysState {
    /// Adds a hotkey
    fn add_hotkey<F: Fn(KeyboardEvent) + Send + Sync + 'static>(
        &mut self,
        id: usize,
        keys: KeyCondition,
        cb: F,
    ) -> Result<(), Box<dyn Error>> {
        self.hotkeys
            .entry(keys)
            .or_default()
            .insert(id, Callback::new(cb));
        Ok(())
    }

    /// Removes a hotkey
    fn remove_hotkey(&mut self, id: usize, keys: &KeyCondition) -> Result<(), Box<dyn Error>> {
        if let Some(entry) = self.hotkeys.get_mut(keys) {
            entry.remove(&id);
        }

        Ok(())
    }
}

/// Use this function at the app entrypoint to enable hotkey support for this component and all of its children.
pub fn provide_hotkeys_context() {
    provide_context(RwSignal::new(HotkeysState::default()));
    let hotkeys_state = use_context::<RwSignal<HotkeysState>>().unwrap();
    #[allow(unused_must_use)]
    let cleanup = use_event_listener(use_document(), keydown, move |ev| {
        let ev_cond: KeyCondition = (&ev).into();
        // info!("key pressed: {:?}", ev_cond);
        hotkeys_state.with_untracked(move |x| {
            if let Some(thing) = x.hotkeys.get(&ev_cond) {
                if thing.len() > 0 {
                    ev.prevent_default();
                    for (_, action) in thing.iter() {
                        action.run(ev.clone());
                    }
                }
            }
        });
    });

    on_cleanup(cleanup);
}

/// Runs a callback when a certain [KeyCondition] is met.
/// Make sure [provide_hotkeys_context] was ran first.
/// No-op on ssr.
pub fn use_hotkey<F: Fn(KeyboardEvent) + Send + Sync + 'static, T: TryInto<KeyCondition>>(
    keys: T,
    cb: F,
) -> Result<(), String>
where
    <T as std::convert::TryInto<KeyCondition>>::Error: Debug + 'static,
{
    #[cfg(not(feature = "ssr"))]
    {
        let id = Owner::current().map(|v| v.debug_id());
        let hotkeys_state = use_context::<RwSignal<HotkeysState>>();

        if let (Some(hotkeys_state), Some(id)) = (hotkeys_state, id) {
            let keys = keys.try_into().map_err(|v| format!("{:?}", v))?;
            let keys2 = keys.clone();

            hotkeys_state.update(|v| v.add_hotkey(id, keys, cb).unwrap());
            debug!("hotkey {:?} added for id {}", keys2, id);

            on_cleanup(move || {
                debug!("hotkey {:?} removed for id {}", keys2, id);
                hotkeys_state.update(move |v| {
                    v.remove_hotkey(id, &keys2).unwrap();
                });
            });
        }
    }
    Ok(())
}

// pub fn use_hotkeys<
//     F: Fn(KeyboardEvent) + 'static,
//     T: TryInto<KeyCondition>,
//     I: IntoIterator<Item = (T, Callback<KeyboardEvent>)>,
// >(
//     keys: T,
//     cb: F,
// ) -> Result<(), String>
// where
//     <T as std::convert::TryInto<KeyCondition>>::Error: Debug + 'static,
// {
//     let hotkeys_state = use_context::<RwSignal<HotkeysState>>();
//     if let Some(hotkeys_state) = hotkeys_state {
//         let keys = keys.try_into().map_err(|v| format!("{:?}", v))?;
//         let keys2 = keys.clone();
//         hotkeys_state.update(|v| v.add_hotkey(vec![keys], cb).unwrap());
//         on_cleanup(move || {
//             hotkeys_state.update(move |v| {
//                 v.remove_hotkey(&vec![keys2]).unwrap();
//             });
//         });
//     }
//     Ok(())
// }
