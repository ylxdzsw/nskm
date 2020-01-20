use super::*;

pub(crate) unsafe fn hook(ev: &input_event, u: &UInput) {
    // simply forward other events
    if ev.type_ != EV_KEY {
        u.emit(ev);
        return
    }

    static mut caps_down: bool = false;
    static mut shortcut_triggered_after_caps_down: bool = false;

    match (ev.code, ev.value) {
        (KEY_CAPSLOCK, V_KEYDOWN) => {
            caps_down = true;
            return
        },
        (KEY_CAPSLOCK, V_KEYUP) => {
            if !shortcut_triggered_after_caps_down {
                // u.click(KEY_CAPSLOCK);
            }
            caps_down = false;
            shortcut_triggered_after_caps_down = false;
            return;
        },
        (KEY_CAPSLOCK, _) => return, // ignore repeating of this key, as we use it as a modifier

        _ => {}
    }

    macro_rules! caps_map_to { // TODO: a fourth argument that throttle (or disable) the repetition
        ($from: ident, $to: ident, $state: ident) => { // concat ident requires nightly so we pass $state in for now
            static mut $state: bool = false;
            match (ev.code, ev.value) {
                ($from, V_KEYDOWN) if caps_down => {
                    $state = true;
                    shortcut_triggered_after_caps_down = true;
                    u.emit(&input_event { code: $to, ..*ev });
                    return
                },
                ($from, V_KEYREP) if $state => {
                    u.emit(&input_event { code: $to, ..*ev });
                    return
                },
                ($from, V_KEYUP) if $state => {
                    $state = false;
                    u.emit(&input_event { code: $to, ..*ev });
                    return
                },
                _ => {}
            }
        };
        ($from: ident, $to: expr, $state: ident) => {
            static mut $state: bool = false;
            match (ev.code, ev.value) {
                ($from, V_KEYDOWN) if caps_down => {
                    $state = true;
                    shortcut_triggered_after_caps_down = true;
                    spawn_orphan($to.as_ptr() as _);
                    return
                },
                ($from, V_KEYREP) if $state => {
                    // spawn_orphan($to.as_ptr() as _);
                    return
                },
                ($from, V_KEYUP) if $state => {
                    $state = false;
                    return
                },
                _ => {}
            }
        };
    }

    caps_map_to!(KEY_A, KEY_LEFT, A_after_caps_down);
    caps_map_to!(KEY_D, KEY_RIGHT, D_after_caps_down);
    caps_map_to!(KEY_W, KEY_UP, W_after_caps_down);
    caps_map_to!(KEY_S, KEY_DOWN, S_after_caps_down);
    caps_map_to!(KEY_Q, KEY_HOME, Q_after_caps_down);
    caps_map_to!(KEY_E, KEY_END, E_after_caps_down);
    caps_map_to!(KEY_R, KEY_PAGEUP, R_after_caps_down);
    caps_map_to!(KEY_F, KEY_PAGEDOWN, F_after_caps_down);
    caps_map_to!(KEY_LEFT, KEY_PREVIOUSSONG, LEFT_after_caps_down);
    caps_map_to!(KEY_RIGHT, KEY_NEXTSONG, RIGHT_after_caps_down);
    caps_map_to!(KEY_UP, "su ylxdzsw -c 'XDG_RUNTIME_DIR=/run/user/1000 pactl set-sink-volume @DEFAULT_SINK@ +1%'\0", UP_after_caps_down);
    caps_map_to!(KEY_DOWN, "su ylxdzsw -c 'XDG_RUNTIME_DIR=/run/user/1000 pactl set-sink-volume @DEFAULT_SINK@ -1%'\0", DOWN_after_caps_down);

    macro_rules! disable {
        ($key:ident) => {
            if ev.code == $key {
                return
            }
        };
    }

    disable!(KEY_RIGHTSHIFT);

    u.emit(ev);
}
