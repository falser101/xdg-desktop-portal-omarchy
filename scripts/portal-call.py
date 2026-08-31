#!/usr/bin/env python3
"""Call an xdg-desktop-portal method and print the Response."""
from __future__ import annotations

import argparse
import json
import sys
from gi.repository import Gio, GLib


def variant_options(items: dict) -> dict:
    packed = {}
    for key, (sig, value) in items.items():
        packed[key] = GLib.Variant(sig, value)
    return packed


def wait_request(bus: Gio.DBusConnection, handle: str, timeout_ms: int):
    loop = GLib.MainLoop()
    box = {"done": False, "response": None, "results": None}

    def on_signal(_conn, _sender, _path, _iface, signal, params):
        if signal != "Response":
            return
        response, results = params.unpack()
        box["done"] = True
        box["response"] = int(response)
        box["results"] = results
        loop.quit()

    bus.signal_subscribe(
        "org.freedesktop.portal.Desktop",
        "org.freedesktop.portal.Request",
        "Response",
        handle,
        None,
        Gio.DBusSignalFlags.NONE,
        on_signal,
    )

    GLib.timeout_add(timeout_ms, loop.quit)
    loop.run()
    return box


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "kind",
        choices=[
            "open",
            "save",
            "open-dir",
            "account",
            "settings",
            "open-uri",
            "app-chooser",
            "access",
            "screenshot",
            "pick-color",
            "background",
        ],
    )
    parser.add_argument("--timeout", type=int, default=20000)
    parser.add_argument(
        "--uri",
        default="https://omarchy.org",
        help="URI for open-uri / app-chooser (default: https://omarchy.org)",
    )
    parser.add_argument(
        "--mime",
        default="",
        help="Unused by OpenURI itself; documented for app-chooser scenarios",
    )
    parser.add_argument(
        "--ask",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Pass ask=true to OpenURI so AppChooser appears (default: true)",
    )
    args = parser.parse_args()

    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)

    if args.kind == "settings":
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
            None,
        )
        color = proxy.call_sync(
            "Read",
            GLib.Variant("(ss)", ("org.freedesktop.appearance", "color-scheme")),
            Gio.DBusCallFlags.NONE,
            5000,
            None,
        )
        accent = proxy.call_sync(
            "Read",
            GLib.Variant("(ss)", ("org.freedesktop.appearance", "accent-color")),
            Gio.DBusCallFlags.NONE,
            5000,
            None,
        )
        print(json.dumps({"color-scheme": color.unpack()[0], "accent-color": accent.unpack()[0]}))
        return 0

    if args.kind in ("open", "save", "open-dir"):
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.FileChooser",
            None,
        )
        folder = b"/tmp/omarchy-portal-test\0"
        choices = (
            "a(ssa(ss)s)",
            [
                (
                    "encoding",
                    "Encoding",
                    [("utf8", "UTF-8"), ("latin1", "Latin-1")],
                    "utf8",
                ),
                ("remember", "Remember this folder", [], "false"),
            ],
        )
        if args.kind == "open":
            method = "OpenFile"
            title = "Portal test: Open"
            options = variant_options(
                {
                    "current_folder": ("ay", list(folder)),
                    "choices": choices,
                }
            )
        elif args.kind == "open-dir":
            method = "OpenFile"
            title = "Portal test: Open folder"
            options = variant_options(
                {
                    "current_folder": ("ay", list(folder)),
                    "directory": ("b", True),
                }
            )
        else:
            method = "SaveFile"
            title = "Portal test: Save"
            options = variant_options(
                {
                    "current_folder": ("ay", list(folder)),
                    "current_file": ("ay", list(b"/tmp/omarchy-portal-test/draft.txt\0")),
                    "choices": choices,
                }
            )
        result = proxy.call_sync(
            method,
            GLib.Variant("(ssa{sv})", ("", title, options)),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        handle = result.unpack()[0]
        print(f"HANDLE {handle}", flush=True)
        box = wait_request(bus, handle, args.timeout)
        print(json.dumps({"response": box["response"], "results": box["results"]}))
        return 0 if box["done"] else 2

    if args.kind == "account":
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Account",
            None,
        )
        result = proxy.call_sync(
            "GetUserInformation",
            GLib.Variant(
                "(sa{sv})",
                (
                    "",
                    {"reason": GLib.Variant("s", "Omarchy portal automated test.")},
                ),
            ),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        handle = result.unpack()[0]
        print(f"HANDLE {handle}", flush=True)
        box = wait_request(bus, handle, args.timeout)
        print(json.dumps({"response": box["response"], "results": box["results"]}))
        return 0 if box["done"] else 2

    if args.kind == "access":
        # Call the omarchy impl directly so we can pass choices/icon without
        # going through a frontend wrapper that strips options.
        #
        # PyGObject's GLib.Variant("(ossssa{sv})", tuple) miscounts elements for
        # this signature; build the tuple with new_tuple instead.
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.impl.portal.desktop.omarchy",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.impl.portal.Access",
            None,
        )
        # choices: a(ssa(ss)s) = id, label, [(option_id, option_label)...], selected
        options = variant_options(
            {
                "icon": ("s", "dialog-password"),
                "deny_label": ("s", "Deny"),
                "grant_label": ("s", "Allow"),
                "choices": (
                    "a(ssa(ss)s)",
                    [
                        ("remember", "Remember this decision", [], "false"),
                        (
                            "scope",
                            "Access scope",
                            [("read", "Read only"), ("write", "Read and write")],
                            "read",
                        ),
                    ],
                ),
            }
        )
        args_variant = GLib.Variant.new_tuple(
            GLib.Variant("o", "/org/freedesktop/portal/desktop/request/t/accesstest"),
            GLib.Variant("s", "org.omarchy.portal.test"),
            GLib.Variant("s", ""),
            GLib.Variant("s", "Allow access?"),
            GLib.Variant("s", "xdg-desktop-portal-omarchy test"),
            GLib.Variant(
                "s",
                "This is an Access portal test.\nToggle and dropdown should appear below.",
            ),
            GLib.Variant("a{sv}", options),
        )
        result = proxy.call_sync(
            "AccessDialog",
            args_variant,
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        print(json.dumps({"reply": result.unpack()}, default=str))
        return 0

    if args.kind == "screenshot":
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Screenshot",
            None,
        )
        result = proxy.call_sync(
            "Screenshot",
            GLib.Variant(
                "(sa{sv})",
                ("", {"interactive": GLib.Variant("b", True)}),
            ),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        handle = result.unpack()[0]
        print(f"HANDLE {handle}", flush=True)
        box = wait_request(bus, handle, args.timeout)
        print(json.dumps({"response": box["response"], "results": box["results"]}))
        return 0 if box["done"] else 2

    if args.kind == "pick-color":
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Screenshot",
            None,
        )
        result = proxy.call_sync(
            "PickColor",
            GLib.Variant("(sa{sv})", ("", {})),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        handle = result.unpack()[0]
        print(f"HANDLE {handle}", flush=True)
        box = wait_request(bus, handle, args.timeout)
        print(json.dumps({"response": box["response"], "results": box["results"]}))
        return 0 if box["done"] else 2

    if args.kind == "background":
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.impl.portal.desktop.omarchy",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.impl.portal.Background",
            None,
        )
        result = proxy.call_sync(
            "NotifyBackground",
            GLib.Variant(
                "(oss)",
                (
                    "/org/freedesktop/portal/desktop/request/t/backgroundtest",
                    "org.omarchy.portal.test",
                    "Portal test",
                ),
            ),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        print(json.dumps({"reply": result.unpack()}))
        return 0

    if args.kind in ("open-uri", "app-chooser"):
        # OpenURI with ask=true is the usual path into impl AppChooser.
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.OpenURI",
            None,
        )
        options = {"ask": GLib.Variant("b", bool(args.ask))}
        result = proxy.call_sync(
            "OpenURI",
            GLib.Variant(
                "(ssa{sv})",
                (
                    "",
                    args.uri,
                    options,
                ),
            ),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        handle = result.unpack()[0]
        print(f"HANDLE {handle}", flush=True)
        if args.kind == "app-chooser":
            print(
                json.dumps(
                    {
                        "hint": "Pick an app in the Omarchy dialog. Check 'Set as default' to write mimeapps.list.",
                        "uri": args.uri,
                        "mime_hint": args.mime or None,
                    }
                ),
                flush=True,
            )
        box = wait_request(bus, handle, args.timeout)
        print(json.dumps({"response": box["response"], "results": box["results"]}))
        return 0 if box["done"] else 2

    return 1


if __name__ == "__main__":
    sys.exit(main())
