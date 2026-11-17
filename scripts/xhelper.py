#!/usr/bin/env python3
"""xhelper — the X11 questions the e2e needs and no installed tool answers.

  xhelper cm-owner            print the _NET_WM_CM_S<screen> owner id (0 = none)
  xhelper own-cm [seconds]    own _NET_WM_CM_S<screen> as a stand-in compositor
  xhelper children            count the root window's children
  xhelper dims                count root children carrying _NET_WM_WINDOW_OPACITY
  xhelper hints               count root children carrying _PERGOLA_HINT
  xhelper bars                geometry of root children carrying _PERGOLA_OVERLAY
                              (one "x,y w,h" line per divider, in creation order)

Why this exists: the overlay's dim layer is only honest when a compositor owns
the `_NET_WM_CM_S<screen>` selection — that is how xfwm4, picom, kwin and mutter
all advertise themselves — and the leak check needs to count windows that
`wmctrl -l` cannot see, because override-redirect windows are not in
_NET_CLIENT_LIST. No installed tool answers either question, so this talks to
libX11 through ctypes: no build step, no new package.

`own-cm` is not a compositor. It only owns the selection, which is the signal
the daemon reads. Whether the layer is actually *rendered* is the compositor's
job and was measured against a real one (see docs/UX-NOTES.md).
"""

import ctypes
import ctypes.util
import sys
import time

WINDOW = ctypes.c_ulong
ATOM = ctypes.c_ulong
CURRENT_TIME = 0


def _lib():
    name = ctypes.util.find_library("X11")
    if not name:
        sys.exit("xhelper: libX11 not found")
    lib = ctypes.CDLL(name)
    lib.XOpenDisplay.restype = ctypes.c_void_p
    lib.XOpenDisplay.argtypes = [ctypes.c_char_p]
    lib.XInternAtom.restype = ATOM
    lib.XInternAtom.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]
    lib.XDefaultScreen.restype = ctypes.c_int
    lib.XDefaultScreen.argtypes = [ctypes.c_void_p]
    lib.XRootWindow.restype = WINDOW
    lib.XRootWindow.argtypes = [ctypes.c_void_p, ctypes.c_int]
    lib.XSetSelectionOwner.restype = ctypes.c_int
    lib.XSetSelectionOwner.argtypes = [ctypes.c_void_p, ATOM, WINDOW, ctypes.c_ulong]
    lib.XGetSelectionOwner.restype = WINDOW
    lib.XGetSelectionOwner.argtypes = [ctypes.c_void_p, ATOM]
    lib.XQueryTree.restype = ctypes.c_int
    lib.XQueryTree.argtypes = [
        ctypes.c_void_p,
        WINDOW,
        ctypes.POINTER(WINDOW),
        ctypes.POINTER(WINDOW),
        ctypes.POINTER(ctypes.POINTER(WINDOW)),
        ctypes.POINTER(ctypes.c_uint),
    ]
    lib.XGetWindowProperty.restype = ctypes.c_int
    lib.XGetWindowProperty.argtypes = [
        ctypes.c_void_p,
        WINDOW,
        ATOM,
        ctypes.c_long,
        ctypes.c_long,
        ctypes.c_int,
        ATOM,
        ctypes.POINTER(ATOM),
        ctypes.POINTER(ctypes.c_int),
        ctypes.POINTER(ctypes.c_ulong),
        ctypes.POINTER(ctypes.c_ulong),
        ctypes.POINTER(ctypes.POINTER(ctypes.c_ubyte)),
    ]
    lib.XGetGeometry.restype = ctypes.c_int
    lib.XGetGeometry.argtypes = [
        ctypes.c_void_p,
        WINDOW,
        ctypes.POINTER(WINDOW),
        ctypes.POINTER(ctypes.c_int),
        ctypes.POINTER(ctypes.c_int),
        ctypes.POINTER(ctypes.c_uint),
        ctypes.POINTER(ctypes.c_uint),
        ctypes.POINTER(ctypes.c_uint),
        ctypes.POINTER(ctypes.c_uint),
    ]
    lib.XSync.restype = ctypes.c_int
    lib.XSync.argtypes = [ctypes.c_void_p, ctypes.c_int]
    lib.XFree.restype = ctypes.c_int
    lib.XFree.argtypes = [ctypes.c_void_p]
    return lib


def _open(lib):
    dpy = lib.XOpenDisplay(None)
    if not dpy:
        sys.exit("xhelper: cannot open DISPLAY")
    screen = lib.XDefaultScreen(dpy)
    cm = lib.XInternAtom(dpy, b"_NET_WM_CM_S%d" % screen, 0)
    return dpy, screen, cm


def _children(lib, dpy, root):
    root_ret = WINDOW()
    parent_ret = WINDOW()
    kids = ctypes.POINTER(WINDOW)()
    n = ctypes.c_uint()
    ok = lib.XQueryTree(
        dpy, root, ctypes.byref(root_ret), ctypes.byref(parent_ret),
        ctypes.byref(kids), ctypes.byref(n),
    )
    if not ok:
        sys.exit("xhelper: XQueryTree failed")
    out = [kids[i] for i in range(n.value)]
    if kids:
        lib.XFree(kids)
    return out


def _has_prop(lib, dpy, win, atom):
    actual_type = ATOM()
    actual_format = ctypes.c_int()
    nitems = ctypes.c_ulong()
    after = ctypes.c_ulong()
    data = ctypes.POINTER(ctypes.c_ubyte)()
    status = lib.XGetWindowProperty(
        dpy, win, atom, 0, 1, 0, 0,
        ctypes.byref(actual_type), ctypes.byref(actual_format),
        ctypes.byref(nitems), ctypes.byref(after), ctypes.byref(data),
    )
    if data:
        lib.XFree(data)
    return status == 0 and actual_type.value != 0


def _geometry(lib, dpy, win):
    """x, y, w, h of a window as the server has it (parent-relative: the root)."""
    root_ret = WINDOW()
    x = ctypes.c_int()
    y = ctypes.c_int()
    w = ctypes.c_uint()
    h = ctypes.c_uint()
    border = ctypes.c_uint()
    depth = ctypes.c_uint()
    ok = lib.XGetGeometry(
        dpy, win, ctypes.byref(root_ret), ctypes.byref(x), ctypes.byref(y),
        ctypes.byref(w), ctypes.byref(h), ctypes.byref(border), ctypes.byref(depth),
    )
    if not ok:
        sys.exit("xhelper: XGetGeometry failed")
    return x.value, y.value, w.value, h.value


def main(argv):
    if not argv:
        sys.exit(__doc__)
    lib = _lib()
    dpy, screen, cm = _open(lib)
    root = lib.XRootWindow(dpy, screen)
    cmd = argv[0]
    if cmd == "cm-owner":
        print(lib.XGetSelectionOwner(dpy, cm))
    elif cmd == "own-cm":
        seconds = float(argv[1]) if len(argv) > 1 else 0.0
        lib.XSetSelectionOwner(dpy, cm, root, CURRENT_TIME)
        lib.XSync(dpy, 0)
        if lib.XGetSelectionOwner(dpy, cm) != root:
            sys.exit("xhelper: could not own _NET_WM_CM_S%d" % screen)
        print("owning _NET_WM_CM_S%d" % screen, flush=True)
        deadline = time.time() + seconds if seconds else None
        while deadline is None or time.time() < deadline:
            time.sleep(0.1)
            lib.XSync(dpy, 0)
    elif cmd == "children":
        print(len(_children(lib, dpy, root)))
    elif cmd == "dims":
        opacity = lib.XInternAtom(dpy, b"_NET_WM_WINDOW_OPACITY", 0)
        print(sum(1 for w in _children(lib, dpy, root) if _has_prop(lib, dpy, w, opacity)))
    elif cmd == "hints":
        # A pointer hint (the hovered-gap line, the drop frame) is an
        # override-redirect window of ours, invisible to wmctrl. Counting them
        # is how the e2e proves a hint appeared and left nothing behind.
        atom = lib.XInternAtom(dpy, b"_PERGOLA_HINT", 0)
        print(sum(1 for w in _children(lib, dpy, root) if _has_prop(lib, dpy, w, atom)))
    elif cmd == "bars":
        # The overlay's split lines are real windows, so the line width is a
        # server fact, not a daemon claim: a 4 px fallback line measures 4.
        atom = lib.XInternAtom(dpy, b"_PERGOLA_OVERLAY", 0)
        for w in _children(lib, dpy, root):
            if _has_prop(lib, dpy, w, atom):
                x, y, bw, bh = _geometry(lib, dpy, w)
                print("%d,%d %dx%d" % (x, y, bw, bh))
    else:
        sys.exit("xhelper: unknown command %r\n%s" % (cmd, __doc__))


if __name__ == "__main__":
    main(sys.argv[1:])
