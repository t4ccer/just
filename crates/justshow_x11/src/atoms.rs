use crate::utils::impl_resource_id;

impl_resource_id!(AtomId);

pub mod predefined {
    use crate::{atoms::AtomId, ResourceId};

    pub const PRIMARY: AtomId = AtomId(ResourceId { value: 1 });
    pub const SECONDARY: AtomId = AtomId(ResourceId { value: 2 });
    pub const ARC: AtomId = AtomId(ResourceId { value: 3 });
    pub const ATOM: AtomId = AtomId(ResourceId { value: 4 });
    pub const BITMAP: AtomId = AtomId(ResourceId { value: 5 });
    pub const CARDINAL: AtomId = AtomId(ResourceId { value: 6 });
    pub const COLORMAP: AtomId = AtomId(ResourceId { value: 7 });
    pub const CURSOR: AtomId = AtomId(ResourceId { value: 8 });
    pub const CUT_BUFFER0: AtomId = AtomId(ResourceId { value: 9 });
    pub const CUT_BUFFER1: AtomId = AtomId(ResourceId { value: 10 });
    pub const CUT_BUFFER2: AtomId = AtomId(ResourceId { value: 11 });
    pub const CUT_BUFFER3: AtomId = AtomId(ResourceId { value: 12 });
    pub const CUT_BUFFER4: AtomId = AtomId(ResourceId { value: 13 });
    pub const CUT_BUFFER5: AtomId = AtomId(ResourceId { value: 14 });
    pub const CUT_BUFFER6: AtomId = AtomId(ResourceId { value: 15 });
    pub const CUT_BUFFER7: AtomId = AtomId(ResourceId { value: 16 });
    pub const DRAWABLE: AtomId = AtomId(ResourceId { value: 17 });
    pub const FONT: AtomId = AtomId(ResourceId { value: 18 });
    pub const INTEGER: AtomId = AtomId(ResourceId { value: 19 });
    pub const PIXMAP: AtomId = AtomId(ResourceId { value: 20 });
    pub const POINT: AtomId = AtomId(ResourceId { value: 21 });
    pub const RECTANGLE: AtomId = AtomId(ResourceId { value: 22 });
    pub const RESOURCE_MANAGER: AtomId = AtomId(ResourceId { value: 23 });
    pub const RGB_COLOR_MAP: AtomId = AtomId(ResourceId { value: 24 });
    pub const RGB_BEST_MAP: AtomId = AtomId(ResourceId { value: 25 });
    pub const RGB_BLUE_MAP: AtomId = AtomId(ResourceId { value: 26 });
    pub const RGB_DEFAULT_MAP: AtomId = AtomId(ResourceId { value: 27 });
    pub const RGB_GRAY_MAP: AtomId = AtomId(ResourceId { value: 28 });
    pub const RGB_GREEN_MAP: AtomId = AtomId(ResourceId { value: 29 });
    pub const RGB_RED_MAP: AtomId = AtomId(ResourceId { value: 30 });
    pub const STRING: AtomId = AtomId(ResourceId { value: 31 });
    pub const VISUALID: AtomId = AtomId(ResourceId { value: 32 });
    pub const WINDOW: AtomId = AtomId(ResourceId { value: 33 });
    pub const WM_COMMAND: AtomId = AtomId(ResourceId { value: 34 });
    pub const WM_HINTS: AtomId = AtomId(ResourceId { value: 35 });
    pub const WM_CLIENT_MACHINE: AtomId = AtomId(ResourceId { value: 36 });
    pub const WM_ICON_NAME: AtomId = AtomId(ResourceId { value: 37 });
    pub const WM_ICON_SIZE: AtomId = AtomId(ResourceId { value: 38 });
    pub const WM_NAME: AtomId = AtomId(ResourceId { value: 39 });
    pub const WM_NORMAL_HINTS: AtomId = AtomId(ResourceId { value: 40 });
    pub const WM_SIZE_HINTS: AtomId = AtomId(ResourceId { value: 41 });
    pub const WM_ZOOM_HINTS: AtomId = AtomId(ResourceId { value: 42 });
    pub const MIN_SPACE: AtomId = AtomId(ResourceId { value: 43 });
    pub const NORM_SPACE: AtomId = AtomId(ResourceId { value: 44 });
    pub const MAX_SPACE: AtomId = AtomId(ResourceId { value: 45 });
    pub const END_SPACE: AtomId = AtomId(ResourceId { value: 46 });
    pub const SUPERSCRIPT_X: AtomId = AtomId(ResourceId { value: 47 });
    pub const SUPERSCRIPT_Y: AtomId = AtomId(ResourceId { value: 48 });
    pub const SUBSCRIPT_X: AtomId = AtomId(ResourceId { value: 49 });
    pub const SUBSCRIPT_Y: AtomId = AtomId(ResourceId { value: 50 });
    pub const UNDERLINE_POSITION: AtomId = AtomId(ResourceId { value: 51 });
    pub const UNDERLINE_THICKNESS: AtomId = AtomId(ResourceId { value: 52 });
    pub const STRIKEOUT_ASCENT: AtomId = AtomId(ResourceId { value: 53 });
    pub const STRIKEOUT_DESCENT: AtomId = AtomId(ResourceId { value: 54 });
    pub const ITALIC_ANGLE: AtomId = AtomId(ResourceId { value: 55 });
    pub const X_HEIGHT: AtomId = AtomId(ResourceId { value: 56 });
    pub const QUAD_WIDTH: AtomId = AtomId(ResourceId { value: 57 });
    pub const WEIGHT: AtomId = AtomId(ResourceId { value: 58 });
    pub const POINT_SIZE: AtomId = AtomId(ResourceId { value: 59 });
    pub const RESOLUTION: AtomId = AtomId(ResourceId { value: 60 });
    pub const COPYRIGHT: AtomId = AtomId(ResourceId { value: 61 });
    pub const NOTICE: AtomId = AtomId(ResourceId { value: 62 });
    pub const FONT_NAME: AtomId = AtomId(ResourceId { value: 63 });
    pub const FAMILY_NAME: AtomId = AtomId(ResourceId { value: 64 });
    pub const FULL_NAME: AtomId = AtomId(ResourceId { value: 65 });
    pub const CAP_HEIGHT: AtomId = AtomId(ResourceId { value: 66 });
    pub const WM_CLASS: AtomId = AtomId(ResourceId { value: 67 });
    pub const WM_TRANSIENT_FOR: AtomId = AtomId(ResourceId { value: 68 });
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
