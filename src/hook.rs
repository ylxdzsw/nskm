use super::*;

pub(crate) unsafe fn hook(mut ev: input_event, u: &UInput) {
    // simply forward non-keyboard events
    if ev.type_ != EV_KEY {
        u.emit(&ev);
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

    static mut rctrl_down: bool = false;
    if ev.code == KEY_RIGHTCTRL {
        match ev.value {
            V_KEYDOWN => rctrl_down = true,
            V_KEYUP => rctrl_down = false,
            _ => {}
        };
        return // ignore it too
    }

    let command_mode = || caps_down || rctrl_down;

    macro_rules! caps_map_to { // TODO: a third argument that throttle (or disable) the repetition
        ($from: ident, $to: ident) => {{
            static mut triggered_in_command_mode: bool = false;
            match (ev.code, ev.value) {
                ($from, V_KEYDOWN) if command_mode() => {
                    triggered_in_command_mode = true;
                    ev.code = $to;
                },
                ($from, V_KEYREP) if triggered_in_command_mode => {
                    ev.code = $to;
                },
                ($from, V_KEYUP) if triggered_in_command_mode => {
                    triggered_in_command_mode = false;
                    ev.code = $to;
                },
                _ => {}
            }
        }};
        ($from: ident, $to: expr) => {{
            static mut triggered_in_command_mode: bool = false;
            match (ev.code, ev.value) {
                ($from, V_KEYDOWN) if command_mode() => {
                    triggered_in_command_mode = true;
                    spawn_orphan($to.as_ptr() as _);
                    return
                },
                ($from, V_KEYREP) if triggered_in_command_mode => {
                    // spawn_orphan($to.as_ptr() as _);
                    return
                },
                ($from, V_KEYUP) if triggered_in_command_mode => {
                    triggered_in_command_mode = false;
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

    caps_map_to!(KEY_J, KEY_LEFT);
    caps_map_to!(KEY_L, KEY_RIGHT);
    caps_map_to!(KEY_I, KEY_UP);
    caps_map_to!(KEY_K, KEY_DOWN);
    caps_map_to!(KEY_U, KEY_HOME);
    caps_map_to!(KEY_O, KEY_END);
    caps_map_to!(KEY_P, KEY_END);

    caps_map_to!(KEY_SPACE, KEY_ESC);

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

    macro_rules! swap {
        ($key1:ident, $key2:ident) => {
            match ev.code {
                $key1 => ev.code = $key2,
                $key2 => ev.code = $key1,
                _ => {}
            }
        }
    }

    swap!(KEY_LEFTCTRL, KEY_LEFTALT);

    u.emit(&ev);
}
