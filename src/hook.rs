use super::*;

pub(crate) unsafe fn hook(ev: &input_event, u: &UInput) {
    // simply forward non-keyboard events
    if ev.type_ != EV_KEY {
        u.emit(ev);
        return
    }

    static mut caps_down: bool = false;
    if ev.code == KEY_CAPSLOCK {
        match ev.value {
            V_KEYDOWN => caps_down = true,
            V_KEYUP => caps_down = false,
            _ => {}
        };
        return // ignore CapsLock key 'cause I never use it
    }

    macro_rules! caps_map_to { // TODO: a fourth argument that throttle (or disable) the repetition
        ($from: ident, $to: ident) => {{
            static mut triggered_after_caps: bool = false;
            match (ev.code, ev.value) {
                ($from, V_KEYDOWN) if caps_down => {
                    triggered_after_caps = true;
                    u.emit(&input_event { code: $to, ..*ev });
                    return
                },
                ($from, V_KEYREP) if triggered_after_caps => {
                    u.emit(&input_event { code: $to, ..*ev });
                    return
                },
                ($from, V_KEYUP) if triggered_after_caps => {
                    triggered_after_caps = false;
                    u.emit(&input_event { code: $to, ..*ev });
                    return
                },
                _ => {}
            }
        }};
        ($from: ident, $to: expr) => {{
            static mut triggered_after_caps: bool = false;
            match (ev.code, ev.value) {
                ($from, V_KEYDOWN) if caps_down => {
                    triggered_after_caps = true;
                    spawn_orphan($to.as_ptr() as _);
                    return
                },
                ($from, V_KEYREP) if triggered_after_caps => {
                    // spawn_orphan($to.as_ptr() as _);
                    return
                },
                ($from, V_KEYUP) if triggered_after_caps => {
                    triggered_after_caps = false;
                    return
                },
                _ => {}
            }
        }}
    }

    caps_map_to!(KEY_A, KEY_LEFT);
    caps_map_to!(KEY_D, KEY_RIGHT);
    caps_map_to!(KEY_W, KEY_UP);
    caps_map_to!(KEY_S, KEY_DOWN);
    caps_map_to!(KEY_Q, KEY_HOME);
    caps_map_to!(KEY_E, KEY_END);
    caps_map_to!(KEY_R, KEY_PAGEUP);
    caps_map_to!(KEY_F, KEY_PAGEDOWN);
    caps_map_to!(KEY_LEFT, KEY_PREVIOUSSONG);
    caps_map_to!(KEY_RIGHT, KEY_NEXTSONG);
    caps_map_to!(KEY_UP, "su ylxdzsw -c 'XDG_RUNTIME_DIR=/run/user/1000 pactl set-sink-volume @DEFAULT_SINK@ +1%'\0");
    caps_map_to!(KEY_DOWN, "su ylxdzsw -c 'XDG_RUNTIME_DIR=/run/user/1000 pactl set-sink-volume @DEFAULT_SINK@ -1%'\0");

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
