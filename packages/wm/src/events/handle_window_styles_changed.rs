use tracing::info;
use wm_common::WindowState;
use wm_platform::NativeWindow;

use crate::{
  commands::window::update_window_state, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

/// Handles `EVENT_OBJECT_STYLECHANGE` for managed windows.
///
/// Delegates to [`try_promote_auto_floated_window`].
pub fn handle_window_styles_changed(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  try_promote_auto_floated_window(native_window, state, config)
}

/// Attempts to promote a window that was auto-floated because it lacked
/// `WS_THICKFRAME` at the moment it was first managed.
///
/// Some applications — most notably Visual Studio — only add the
/// resizable style flag after their main window has finished
/// initializing, which leaves them stuck floating until the user
/// manually toggles them or restarts the WM (#344).
///
/// This is intentionally cheap (a `HashSet` lookup) so it can be called
/// from any high-frequency event handler that might fire as a window
/// finishes initializing (style changes, location changes, focus, etc.).
///
/// If the window is in [`WmState::auto_floated_for_unresizable`] and
/// `is_resizable()` now returns true, the window transitions to the
/// user's configured default state.
pub fn try_promote_auto_floated_window(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let window_id = native_window.id();

  if !state.auto_floated_for_unresizable.contains(&window_id) {
    return Ok(());
  }

  if !state.auto_floated_for_unresizable.contains(&window_id) {
    return Ok(());
  }

  let Some(window) = state.window_from_native(native_window) else {
    // Window is no longer managed; clean up the marker.
    state.auto_floated_for_unresizable.remove(&window_id);
    return Ok(());
  };

  // If the user explicitly changed the state in the meantime, drop the
  // marker and don't override their choice.
  if !matches!(window.state(), WindowState::Floating(_)) {
    state.auto_floated_for_unresizable.remove(&window_id);
    return Ok(());
  }

  let is_resizable = native_window.is_resizable().unwrap_or(false);
  if !is_resizable {
    return Ok(());
  }

  state.auto_floated_for_unresizable.remove(&window_id);

  let target_state = WindowState::default_from_config(&config.value);
  info!(
    "Promoting auto-floated window to {:?}: {window}",
    target_state
  );

  update_window_state(window, target_state, state, config)?;

  Ok(())
}
