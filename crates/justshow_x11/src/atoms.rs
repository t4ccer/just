use crate::{utils::impl_resource_id, ResourceId};

impl_resource_id!(AtomId);

macro_rules! define_atom_id {
    ($($name:ident = $value:expr,)*) => {
        $(pub const $name: Self = Self(ResourceId { value: $value });)*
    };
}

impl AtomId {
    define_atom_id! {
        PRIMARY = 1,
        SECONDARY = 2,
        ARC = 3,
        ATOM = 4,
        BITMAP = 5,
        CARDINAL = 6,
        COLORMAP = 7,
        CURSOR = 8,
        CUT_BUFFER0 = 9,
        CUT_BUFFER1 = 10,
        CUT_BUFFER2 = 11,
        CUT_BUFFER3 = 12,
        CUT_BUFFER4 = 13,
        CUT_BUFFER5 = 14,
        CUT_BUFFER6 = 15,
        CUT_BUFFER7 = 16,
        DRAWABLE = 17,
        FONT = 18,
        INTEGER = 19,
        PIXMAP = 20,
        POINT = 21,
        RECTANGLE = 22,
        RESOURCE_MANAGER = 23,
        RGB_COLOR_MAP = 24,
        RGB_BEST_MAP = 25,
        RGB_BLUE_MAP = 26,
        RGB_DEFAULT_MAP = 27,
        RGB_GRAY_MAP = 28,
        RGB_GREEN_MAP = 29,
        RGB_RED_MAP = 30,
        STRING = 31,
        VISUALID = 32,
        WINDOW = 33,
        WM_COMMAND = 34,
        WM_HINTS = 35,
        WM_CLIENT_MACHINE = 36,
        WM_ICON_NAME = 37,
        WM_ICON_SIZE = 38,
        WM_NAME = 39,
        WM_NORMAL_HINTS = 40,
        WM_SIZE_HINTS = 41,
        WM_ZOOM_HINTS = 42,
        MIN_SPACE = 43,
        NORM_SPACE = 44,
        MAX_SPACE = 45,
        END_SPACE = 46,
        SUPERSCRIPT_X = 47,
        SUPERSCRIPT_Y = 48,
        SUBSCRIPT_X = 49,
        SUBSCRIPT_Y = 50,
        UNDERLINE_POSITION = 51,
        UNDERLINE_THICKNESS = 52,
        STRIKEOUT_ASCENT = 53,
        STRIKEOUT_DESCENT = 54,
        ITALIC_ANGLE = 55,
        X_HEIGHT = 56,
        QUAD_WIDTH = 57,
        WEIGHT = 58,
        POINT_SIZE = 59,
        RESOLUTION = 60,
        COPYRIGHT = 61,
        NOTICE = 62,
        FONT_NAME = 63,
        FAMILY_NAME = 64,
        FULL_NAME = 65,
        CAP_HEIGHT = 66,
        WM_CLASS = 67,
        WM_TRANSIENT_FOR = 68,
    }
}

/// 'Extended Window Manager Hints' atoms
pub mod wm {
    #![allow(non_snake_case)]

    use crate::replies::String8;

    macro_rules! define_atoms {
        ($($atom:tt,)*) => {
            $(
                pub fn $atom() -> String8 {
                    String8::from_str(stringify!($atom))
                }
            )*

        };
    }

    define_atoms! {
        _NET_WM_NAME,
        _NET_WM_VISIBLE_NAME,
        _NET_WM_ICON_NAME,
        _NET_WM_VISIBLE_ICON_NAME,
        _NET_WM_DESKTOP,
        _NET_WM_WINDOW_TYPE,
        _NET_WM_STATE,
        _NET_WM_ALLOWED_ACTIONS,
        _NET_WM_STRUT,
        _NET_WM_STRUT_PARTIAL,
        _NET_WM_ICON_GEOMETRY,
        _NET_WM_ICON,
        _NET_WM_PID,
        _NET_WM_HANDLED_ICONS,
        _NET_WM_USER_TIME,
        _NET_FRAME_EXTENTS,
    }
}
