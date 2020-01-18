use super::*;

pub(crate) unsafe fn hook(ev: &input_event, u: &UInput) {
    // simply forward other events
    if ev.type_ != EV_KEY {
        u.emit(ev);
        return
    }

    static mut caps_down: bool = false;
    static mut shortcut_triggered_after_caps_down: bool = false;

    static mut A_after_caps_down: bool = false;
    static mut D_after_caps_down: bool = false;

    match (ev.code, ev.value) {
        (KEY_CAPSLOCK, V_KEYDOWN) => caps_down = true,
        (KEY_CAPSLOCK, V_KEYUP) => {
            if !shortcut_triggered_after_caps_down {
                u.click(KEY_CAPSLOCK);
            }
            caps_down = false;
            shortcut_triggered_after_caps_down = false;
        },
        (KEY_CAPSLOCK, _) => {}, // ignore repeating of this key, as we use it as a modifier

        (KEY_A, V_KEYDOWN) if caps_down => {
            A_after_caps_down = true;
            shortcut_triggered_after_caps_down = true;
            u.emit(&input_event { code: KEY_LEFT, ..*ev });
        },
        (KEY_A, V_KEYREP) if A_after_caps_down => {
            u.emit(&input_event { code: KEY_LEFT, ..*ev });
        },
        (KEY_A, V_KEYUP) if A_after_caps_down => {
            A_after_caps_down = false;
            u.emit(&input_event { code: KEY_LEFT, ..*ev });
        },

        (KEY_D, V_KEYDOWN) if caps_down => {
            D_after_caps_down = true;
            shortcut_triggered_after_caps_down = true;
            u.emit(&input_event { code: KEY_RIGHT, ..*ev });
        },
        (KEY_D, V_KEYREP) if D_after_caps_down => {
            u.emit(&input_event { code: KEY_RIGHT, ..*ev });
        },
        (KEY_D, V_KEYUP) if D_after_caps_down => {
            D_after_caps_down = false;
            u.emit(&input_event { code: KEY_RIGHT, ..*ev });
        },

        (KEY_RIGHT, V_KEYDOWN) if caps_down => {
            shortcut_triggered_after_caps_down = true;
            u.click(KEY_NEXTSONG);
        },
        (KEY_LEFT, V_KEYDOWN) if caps_down => {
            shortcut_triggered_after_caps_down = true;
            u.click(KEY_PREVIOUSSONG);
        }

        _ => { u.emit(ev); }
    }
}
