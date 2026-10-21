// Compile-only ABI assertions against the installed API 26 Native SDK.
#include <stddef.h>

#include "ace/xcomponent/native_interface_xcomponent.h"
#include "native_drawing/drawing_error_code.h"
#include "native_drawing/drawing_types.h"

_Static_assert(OH_NATIVE_XCOMPONENT_MAX_TOUCH_POINTS_NUMBER == 10,
               "touch capacity");
_Static_assert(OH_NATIVEXCOMPONENT_DOWN == 0, "touch down value");
_Static_assert(OH_NATIVEXCOMPONENT_CANCEL == 3, "touch cancel value");
_Static_assert(sizeof(OH_NativeXComponent_Callback) == 32, "callback size");
_Static_assert(_Alignof(OH_NativeXComponent_Callback) == 8,
               "callback alignment");
_Static_assert(sizeof(OH_NativeXComponent_TouchPoint) == 56, "point size");
_Static_assert(offsetof(OH_NativeXComponent_TouchPoint, size) == 24,
               "point size field");
_Static_assert(offsetof(OH_NativeXComponent_TouchPoint, timeStamp) == 40,
               "point timestamp");
_Static_assert(sizeof(OH_NativeXComponent_TouchEvent) == 624, "event size");
_Static_assert(offsetof(OH_NativeXComponent_TouchEvent, touchPoints) == 56,
               "event points");
_Static_assert(offsetof(OH_NativeXComponent_TouchEvent, numPoints) == 616,
               "event count");

_Static_assert(COLOR_FORMAT_RGBA_8888 == 4, "RGBA format value");
_Static_assert(ALPHA_FORMAT_PREMUL == 2, "premultiplied alpha value");
_Static_assert(sizeof(OH_Drawing_Image_Info) == 16, "image info size");
_Static_assert(_Alignof(OH_Drawing_Image_Info) == 4, "image info alignment");
_Static_assert(offsetof(OH_Drawing_Image_Info, colorType) == 8,
               "color type offset");
_Static_assert(sizeof(OH_Drawing_ErrorCode) == 4, "drawing error enum size");
