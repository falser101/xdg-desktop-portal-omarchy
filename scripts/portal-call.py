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


def dynamic_launcher_icon_variant() -> GLib.Variant:
    """Valid GBytesIcon for PrepareInstall (xdg-desktop-portal validates PNG/JPEG/SVG)."""
    import struct
    import zlib

    # Prefer a real theme icon when available — always passes glycin validation.
    for path in (
        "/usr/share/icons/AdwaitaLegacy/48x48/legacy/utilities-terminal.png",
        "/usr/share/icons/hicolor/48x48/apps/utilities-terminal.png",
        "/usr/share/pixmaps/archlinux-logo.png",
    ):
        try:
            with open(path, "rb") as fh:
                data = fh.read()
            if data.startswith(b"\x89PNG"):
                return Gio.BytesIcon.new(GLib.Bytes.new(data)).serialize()
        except OSError:
            pass

    # Fallback: solid 64×64 RGB PNG with correct CRCs.
    width = height = 64
    raw = b"".join(b"\x00" + bytes([30, 144, 255]) * width for _ in range(height))

    def chunk(tag: bytes, payload: bytes) -> bytes:
        return (
            struct.pack(">I", len(payload))
            + tag
            + payload
            + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)
        )

    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    return Gio.BytesIcon.new(GLib.Bytes.new(png)).serialize()


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
            "notification",
            "notification-remove",
            "dynamic-launcher",
            "dynamic-launcher-webapp",
            "dynamic-launcher-token",
            "dynamic-launcher-uninstall",
            "email",
        ],
    )
    parser.add_argument("--timeout", type=int, default=20000)
    parser.add_argument(
        "--uri",
        default="https://omarchy.org",
        help="URI for open-uri / app-chooser / dynamic-launcher-webapp",
    )
    parser.add_argument(
        "--address",
        default="test@example.com",
        help="To: address for email (default: test@example.com)",
    )
    parser.add_argument(
        "--subject",
        default="Omarchy portal Email test",
        help="Subject for email",
    )
    parser.add_argument(
        "--body",
        default="Sent via xdg-desktop-portal-omarchy Email portal.",
        help="Body for email",
    )
    parser.add_argument(
        "--cc",
        action="append",
        default=[],
        help="CC address (repeatable)",
    )
    parser.add_argument(
        "--bcc",
        action="append",
        default=[],
        help="BCC address (repeatable)",
    )
    parser.add_argument(
        "--attach",
        action="append",
        default=[],
        metavar="PATH",
        help="Attachment file path (repeatable; sent as attachment_fds)",
    )
    parser.add_argument(
        "--folder",
        default="",
        help="current_folder for FileChooser open/save/open-dir (absolute path)",
    )
    parser.add_argument(
        "--file",
        default="",
        dest="current_file",
        help="current_file for FileChooser save (absolute path)",
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
    parser.add_argument(
        "--name",
        default="Omarchy Portal Test",
        help="Launcher name for dynamic-launcher (default: Omarchy Portal Test)",
    )
    parser.add_argument(
        "--install",
        action="store_true",
        help="After PrepareInstall succeeds, call Install with a test .desktop",
    )
    parser.add_argument(
        "--desktop-file-id",
        default="org.omarchy.portal.test.Launcher.desktop",
        help="desktop_file_id for Install / Uninstall",
    )
    parser.add_argument(
        "--app-id",
        default="org.omarchy.portal.test",
        help="app_id for RequestInstallToken / impl PrepareInstall",
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
        folder_path = args.folder or "/tmp/omarchy-portal-test"
        folder = (folder_path.encode() + b"\0")
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
            current_file = args.current_file or f"{folder_path.rstrip('/')}/draft.txt"
            options = variant_options(
                {
                    "current_folder": ("ay", list(folder)),
                    "current_file": ("ay", list(current_file.encode() + b"\0")),
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
        # Direct impl call (same as before). Expect a 3-button dialog:
        # Deny=0, Allow=1, Allow once=2 (close without choose → 2).
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
        unpacked = result.unpack()
        # (response, {result: u})
        label = {0: "Forbid", 1: "Allow", 2: "Allow once"}
        try:
            code = int(unpacked[1].get("result", unpacked[1]["result"]))
        except Exception:
            code = None
        print(
            json.dumps(
                {
                    "reply": unpacked,
                    "result": code,
                    "meaning": label.get(code, "unknown"),
                }
            )
        )
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

    if args.kind == "notification":
        # Frontend portal; exercises AddNotification with buttons + default-action
        # + persistent hint. Click the toast / button to fire ActionInvoked.
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Notification",
            None,
        )
        notif = {
            "title": GLib.Variant("s", "Portal notification test"),
            "body": GLib.Variant(
                "s", "Click the toast (default) or the Open button."
            ),
            "priority": GLib.Variant("s", "normal"),
            "icon": GLib.Variant("s", "dialog-information"),
            "default-action": GLib.Variant("s", "open-main"),
            "buttons": GLib.Variant(
                "aa{sv}",
                [
                    {
                        "label": GLib.Variant("s", "Open"),
                        "action": GLib.Variant("s", "open-main"),
                    },
                    {
                        "label": GLib.Variant("s", "Dismiss"),
                        "action": GLib.Variant("s", "dismiss"),
                    },
                ],
            ),
            "display-hint": GLib.Variant("as", ["persistent"]),
        }
        proxy.call_sync(
            "AddNotification",
            GLib.Variant("(sa{sv})", ("portal-call-test", notif)),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        print(
            json.dumps(
                {
                    "ok": True,
                    "id": "portal-call-test",
                    "hint": "Toast should appear; click it or Open. Then: portal-call.py notification-remove",
                }
            ),
            flush=True,
        )
        return 0

    if args.kind == "notification-remove":
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Notification",
            None,
        )
        proxy.call_sync(
            "RemoveNotification",
            GLib.Variant("(s)", ("portal-call-test",)),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        print(json.dumps({"ok": True, "removed": "portal-call-test"}), flush=True)
        return 0

    if args.kind in ("dynamic-launcher", "dynamic-launcher-webapp"):
        # Frontend PrepareInstall → Omarchy Confirm dialog.
        # Accept → results include short-lived token (+ name/icon).
        # Pass --install to complete Install with a test .desktop entry.
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.DynamicLauncher",
            None,
        )
        icon_v = dynamic_launcher_icon_variant()
        options: dict = {
            "modal": GLib.Variant("b", True),
            "editable_name": GLib.Variant("b", True),
        }
        if args.kind == "dynamic-launcher-webapp":
            options["launcher_type"] = GLib.Variant("u", 2)
            options["target"] = GLib.Variant("s", args.uri)
        else:
            options["launcher_type"] = GLib.Variant("u", 1)

        result = proxy.call_sync(
            "PrepareInstall",
            GLib.Variant("(ssva{sv})", ("", args.name, icon_v, options)),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        handle = result.unpack()[0]
        print(f"HANDLE {handle}", flush=True)
        print(
            json.dumps(
                {
                    "hint": "Confirm dialog should appear (Install / Cancel).",
                    "kind": args.kind,
                    "name": args.name,
                }
            ),
            flush=True,
        )
        box = wait_request(bus, handle, args.timeout)
        out = {
            "response": box["response"],
            "results": box["results"],
            "meaning": {0: "success", 1: "cancelled", 2: "other"}.get(
                box["response"], "unknown"
            ),
        }
        print(json.dumps(out, default=str), flush=True)
        if not box["done"]:
            return 2
        if box["response"] != 0 or not args.install:
            return 0 if box["response"] is not None else 2

        token = box["results"].get("token")
        if not token:
            print(json.dumps({"error": "no token in PrepareInstall results"}))
            return 1
        desktop_entry = "\n".join(
            [
                "[Desktop Entry]",
                "Type=Application",
                "Exec=true",
                "TryExec=true",
                "Terminal=false",
                "Categories=Utility;",
                "Comment=xdg-desktop-portal-omarchy DynamicLauncher test",
                "",
            ]
        )
        proxy.call_sync(
            "Install",
            GLib.Variant(
                "(sssa{sv})",
                (token, args.desktop_file_id, desktop_entry, {}),
            ),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        entry = proxy.call_sync(
            "GetDesktopEntry",
            GLib.Variant("(s)", (args.desktop_file_id,)),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        ).unpack()[0]
        print(
            json.dumps(
                {
                    "installed": True,
                    "desktop_file_id": args.desktop_file_id,
                    "desktop_entry": entry,
                    "hint": f"Remove with: portal-call.py dynamic-launcher-uninstall --desktop-file-id {args.desktop_file_id}",
                }
            ),
            flush=True,
        )
        return 0

    if args.kind == "dynamic-launcher-token":
        # Backend allowlist check (no dialog). Only Software/Discover/AppCenter
        # should get SUCCESS=0; random app ids get OTHER=2.
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.impl.portal.desktop.omarchy",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.impl.portal.DynamicLauncher",
            None,
        )
        code = proxy.call_sync(
            "RequestInstallToken",
            GLib.Variant("(sa{sv})", (args.app_id, {})),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        ).unpack()[0]
        print(
            json.dumps(
                {
                    "app_id": args.app_id,
                    "response": int(code),
                    "meaning": {0: "allowed (no dialog)", 2: "denied"}.get(
                        int(code), "other"
                    ),
                    "hint": "Try --app-id org.freedesktop.portal.test (or an allowed id) vs org.omarchy.portal.test (denied)",
                }
            )
        )
        return 0

    if args.kind == "dynamic-launcher-uninstall":
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.DynamicLauncher",
            None,
        )
        proxy.call_sync(
            "Uninstall",
            GLib.Variant("(sa{sv})", (args.desktop_file_id, {})),
            Gio.DBusCallFlags.NONE,
            args.timeout,
            None,
        )
        print(json.dumps({"ok": True, "uninstalled": args.desktop_file_id}))
        return 0

    if args.kind == "email":
        # Frontend ComposeEmail → omarchy impl → xdg-email.
        # Opens the default mailto: handler (no portal dialog).
        proxy = Gio.DBusProxy.new_sync(
            bus,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Email",
            None,
        )
        options: dict = {
            "address": GLib.Variant("s", args.address),
            "subject": GLib.Variant("s", args.subject),
            "body": GLib.Variant("s", args.body),
        }
        if args.cc:
            options["cc"] = GLib.Variant("as", list(args.cc))
        if args.bcc:
            options["bcc"] = GLib.Variant("as", list(args.bcc))

        fd_list = None
        keep_open = []
        if args.attach:
            fd_list = Gio.UnixFDList.new()
            handles = []
            for path in args.attach:
                fh = open(path, "rb")
                keep_open.append(fh)
                handles.append(fd_list.append(fh.fileno()))
            options["attachment_fds"] = GLib.Variant("ah", handles)

        print(
            json.dumps(
                {
                    "hint": "Default mailto: client should open; no portal dialog.",
                    "address": args.address,
                    "subject": args.subject,
                    "cc": args.cc,
                    "bcc": args.bcc,
                    "attach": args.attach,
                }
            ),
            flush=True,
        )
        if fd_list is not None:
            result = proxy.call_with_unix_fd_list_sync(
                "ComposeEmail",
                GLib.Variant("(sa{sv})", ("", options)),
                Gio.DBusCallFlags.NONE,
                args.timeout,
                fd_list,
                None,
            )[0]
        else:
            result = proxy.call_sync(
                "ComposeEmail",
                GLib.Variant("(sa{sv})", ("", options)),
                Gio.DBusCallFlags.NONE,
                args.timeout,
                None,
            )
        for fh in keep_open:
            fh.close()
        handle = result.unpack()[0]
        print(f"HANDLE {handle}", flush=True)
        box = wait_request(bus, handle, args.timeout)
        print(
            json.dumps(
                {
                    "response": box["response"],
                    "results": box["results"],
                    "meaning": {0: "success", 1: "cancelled", 2: "other"}.get(
                        box["response"], "unknown"
                    ),
                },
                default=str,
            )
        )
        return 0 if box["done"] and box["response"] == 0 else (2 if not box["done"] else 1)

    return 1


if __name__ == "__main__":
    sys.exit(main())
