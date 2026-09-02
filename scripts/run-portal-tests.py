#!/usr/bin/env python3
"""Full portal regression harness for xdg-desktop-portal-omarchy.

Produces a Markdown report under /tmp/omarchy-portal-test-report.md
and JSON under /tmp/omarchy-portal-test-report.json.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from dataclasses import asdict, dataclass, field
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PORTAL_CALL = ROOT / "scripts" / "portal-call.py"
REPORT_MD = Path("/tmp/omarchy-portal-test-report.md")
REPORT_JSON = Path("/tmp/omarchy-portal-test-report.json")
SHOT_DIR = Path("/tmp/omarchy-portal-test-shots")


@dataclass
class CaseResult:
    id: str
    name: str
    category: str
    expected: str
    status: str  # pass | fail | skip | warn
    detail: str = ""
    duration_ms: int = 0
    artifacts: list[str] = field(default_factory=list)


def run(cmd: list[str] | str, timeout: float | None = 60, env=None) -> subprocess.CompletedProcess:
    return subprocess.run(
        cmd,
        shell=isinstance(cmd, str),
        capture_output=True,
        text=True,
        timeout=timeout,
        env=env,
        cwd=str(ROOT),
    )


def hypr_clients() -> list[dict]:
    try:
        out = run(["hyprctl", "-j", "clients"], timeout=5)
        return json.loads(out.stdout or "[]")
    except Exception:
        return []


def find_portal_window() -> dict | None:
    for c in hypr_clients():
        if (c.get("class") or "") == "xdg-desktop-portal-omarchy":
            return c
    for c in hypr_clients():
        title = c.get("title") or ""
        cls = c.get("class") or ""
        if cls == "org.quickshell" and (
            "Portal" in title
            or "User Information" in title
            or "Background" in title
            or "Launcher" in title
            or "Access" in title
            or title in ("Open", "Save", "Select folder")
        ):
            return c
    return None


def focus_and_escape(win: dict | None) -> None:
    if win and win.get("address"):
        run(["hyprctl", "dispatch", "focuswindow", f"address:{win['address']}"], timeout=5)
        time.sleep(0.15)
    run(["wtype", "-k", "Escape"], timeout=5)


def shot_window(win: dict | None, name: str) -> str | None:
    SHOT_DIR.mkdir(parents=True, exist_ok=True)
    path = SHOT_DIR / f"{name}.png"
    if win and win.get("at") and win.get("size"):
        x, y = win["at"]
        w, h = win["size"]
        r = run(["grim", "-g", f"{x},{y} {w}x{h}", str(path)], timeout=10)
        if r.returncode == 0 and path.exists():
            return str(path)
    r = run(["grim", str(path)], timeout=10)
    return str(path) if r.returncode == 0 and path.exists() else None


def interactive_cancel(kind: str, extra: list[str] | None = None, timeout_ms: int = 25000) -> CaseResult:
    cid = f"ui.{kind}"
    name = f"Interactive {kind} opens then cancels (Esc)"
    t0 = time.time()
    cmd = [sys.executable, str(PORTAL_CALL), kind, "--timeout", str(timeout_ms)]
    if extra:
        cmd.extend(extra)
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, cwd=str(ROOT))
    win = None
    for _ in range(80):
        win = find_portal_window()
        if win:
            break
        if proc.poll() is not None:
            break
        time.sleep(0.1)
    artifacts = []
    if win:
        shot = shot_window(win, f"{kind}-dialog")
        if shot:
            artifacts.append(shot)
        focus_and_escape(win)
    else:
        # Still try Escape in case window class differs
        run(["wtype", "-k", "Escape"], timeout=5)

    try:
        out, _ = proc.communicate(timeout=max(5, timeout_ms / 1000 + 5))
    except subprocess.TimeoutExpired:
        proc.kill()
        out, _ = proc.communicate()
        return CaseResult(
            cid,
            name,
            "interactive",
            "dialog appears; Response cancelled (1) or closed",
            "fail",
            f"process hung; window={'yes' if win else 'no'}; out={out[-500:]}",
            int((time.time() - t0) * 1000),
            artifacts,
        )

    duration = int((time.time() - t0) * 1000)
    parsed = parse_portal_output(out or "", kind)

    if not win:
        if parsed.ok is None:
            return CaseResult(
                cid,
                name,
                "interactive",
                "Omarchy Portal / egui window appears",
                "fail",
                f"no portal window; exit={proc.returncode}; out={(out or '')[-800:]}",
                duration,
                artifacts,
            )
        return CaseResult(
            cid,
            name,
            "interactive",
            "window appears (egui picker)",
            "warn",
            f"no egui window matched, but {parsed.summary}; out={(out or '')[-400:]}",
            duration,
            artifacts,
        )

    if parsed.ok is True:
        status = "pass"
        detail = f"window={win.get('title')!r} class={win.get('class')!r} {parsed.summary}"
    elif parsed.ok is False:
        status = "fail"
        detail = f"window shown; unexpected outcome {parsed.summary}; out={(out or '')[-500:]}"
    else:
        status = "fail"
        detail = f"window shown but no parseable reply; out={(out or '')[-500:]}"

    return CaseResult(cid, name, "interactive", parsed.expected, status, detail, duration, artifacts)


@dataclass
class ParsedReply:
    ok: bool | None
    summary: str
    expected: str


def parse_portal_output(out: str, kind: str) -> ParsedReply:
    """Interpret portal-call.py stdout for Esc-cancel tests.

    Frontend portals print {"response": 0|1, ...}.
    Access/Background call the impl directly:
      Access → {"reply": [code, dict]}  (1 = cancelled)
      Background → {"reply": [...], "result": 0|1|2}  (Esc → Deny = 0)
    """
    data = None
    for line in reversed(out.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                data = json.loads(line)
            except json.JSONDecodeError:
                continue
            break
    if not isinstance(data, dict):
        return ParsedReply(None, "no JSON", "dialog + cancelled/deny reply")

    if kind == "access":
        reply = data.get("reply")
        code = reply[0] if isinstance(reply, (list, tuple)) and reply else data.get("response")
        try:
            code = int(code)
        except (TypeError, ValueError):
            return ParsedReply(None, f"raw={data}", "Access reply 1 (deny/cancel)")
        return ParsedReply(
            code == 1,
            f"reply={code}",
            "Access impl reply 1 (cancelled)",
        )

    if kind == "background":
        result = data.get("result")
        if result is None:
            reply = data.get("reply")
            if isinstance(reply, (list, tuple)) and len(reply) > 1 and isinstance(reply[1], dict):
                result = reply[1].get("result")
        try:
            result = int(result)
        except (TypeError, ValueError):
            return ParsedReply(None, f"raw={data}", "Background result 0 (Deny)")
        meaning = {0: "Forbid", 1: "Allow", 2: "Allow once"}.get(result, "unknown")
        return ParsedReply(
            result == 0,
            f"result={result} ({meaning})",
            "Background result 0 (Deny on Esc)",
        )

    response = data.get("response")
    try:
        response = int(response)
    except (TypeError, ValueError):
        return ParsedReply(None, f"raw={data}", "dialog + Response 0/1")
    return ParsedReply(
        response in (0, 1),
        f"response={response}",
        "dialog + Response 0/1",
    )


def noninteractive(kind: str, extra: list[str] | None = None, timeout_ms: int = 15000) -> CaseResult:
    cid = f"api.{kind}"
    name = f"API {kind}"
    t0 = time.time()
    cmd = [sys.executable, str(PORTAL_CALL), kind, "--timeout", str(timeout_ms)]
    if extra:
        cmd.extend(extra)
    try:
        r = run(cmd, timeout=timeout_ms / 1000 + 10)
    except subprocess.TimeoutExpired as e:
        return CaseResult(cid, name, "api", "returns promptly", "fail", f"timeout: {e}", int((time.time() - t0) * 1000))
    duration = int((time.time() - t0) * 1000)
    out = (r.stdout or "") + (r.stderr or "")
    if r.returncode != 0 and kind not in ("dynamic-launcher-uninstall",):
        # Some kinds print JSON even on cancel
        if '"response": 1' in out or '"response":1' in out:
            return CaseResult(cid, name, "api", "completes", "pass", f"cancelled ok; {out[-300:]}", duration)
        return CaseResult(cid, name, "api", "exit 0 or valid JSON", "fail", f"rc={r.returncode} out={out[-600:]}", duration)
    return CaseResult(cid, name, "api", "exit 0 + payload", "pass", out.strip()[-500:], duration)


def unit_tests() -> CaseResult:
    t0 = time.time()
    r = run(["cargo", "test", "--lib", "--", "--nocapture"], timeout=180)
    duration = int((time.time() - t0) * 1000)
    out = (r.stdout or "") + (r.stderr or "")
    m = re.search(r"(\d+) passed.*?(\d+) failed", out)
    if r.returncode == 0:
        return CaseResult("unit.cargo", "cargo test --lib", "unit", "all pass", "pass", out.strip().splitlines()[-5:] and "\n".join(out.strip().splitlines()[-8:]), duration)
    return CaseResult(
        "unit.cargo",
        "cargo test --lib",
        "unit",
        "all pass",
        "fail",
        out[-1500:],
        duration,
    )


def check_env() -> list[CaseResult]:
    cases = []
    # services
    for unit in (
        "xdg-desktop-portal-omarchy.service",
        "xdg-desktop-portal.service",
        "xdg-desktop-portal-hyprland.service",
    ):
        r = run(["systemctl", "--user", "is-active", unit], timeout=5)
        active = (r.stdout or "").strip() == "active"
        cases.append(
            CaseResult(
                f"env.{unit}",
                f"service {unit}",
                "env",
                "active",
                "pass" if active else "fail",
                (r.stdout or r.stderr or "").strip(),
            )
        )

    # routing
    conf = Path.home() / ".config/xdg-desktop-portal/hyprland-portals.conf"
    text = conf.read_text() if conf.exists() else ""
    for key, want in [
        ("FileChooser", "omarchy"),
        ("Settings", "omarchy"),
        ("ScreenCast", "hyprland"),
        ("Secret", "gnome-keyring"),
    ]:
        line = next((ln for ln in text.splitlines() if f"portal.{key}" in ln), "")
        ok = want in line or (key == "FileChooser" and "default=omarchy" in text and not line)
        # FileChooser may come from default=
        if key == "FileChooser":
            ok = "omarchy" in text and ("FileChooser=omarchy" in text or "default=omarchy" in text)
        cases.append(
            CaseResult(
                f"env.route.{key}",
                f"route {key} → {want}",
                "env",
                f"config contains {want}",
                "pass" if ok else "fail",
                line or text[:200],
            )
        )

    plugin = Path.home() / ".config/omarchy/plugins/omarchy-portal"
    cases.append(
        CaseResult(
            "env.no_user_plugin",
            "No Quickshell plugin in ~/.config/omarchy/plugins",
            "env",
            "omarchy-portal plugin absent",
            "pass" if not plugin.exists() else "warn",
            str(plugin),
        )
    )

    # xdph picker
    xdph = Path.home() / ".config/hypr/xdph.conf"
    xdph_text = xdph.read_text() if xdph.exists() else ""
    picker_ok = "omarchy-share-picker" in xdph_text
    cases.append(
        CaseResult(
            "env.xdph_picker",
            "xdph.conf uses omarchy-share-picker",
            "env",
            "configured",
            "pass" if picker_ok else "warn",
            xdph_text.strip() or "(missing xdph.conf)",
        )
    )

    # package vs git HEAD
    pkg = run(["pacman", "-Q", "xdg-desktop-portal-omarchy-git"], timeout=5)
    head = run(["git", "rev-parse", "--short", "HEAD"], timeout=5)
    cases.append(
        CaseResult(
            "env.package_vs_git",
            "AUR package vs git HEAD",
            "env",
            "informational",
            "warn",
            f"pkg={(pkg.stdout or '').strip()} HEAD={(head.stdout or '').strip()}",
        )
    )

    # dbus name owned
    r = run(["busctl", "--user", "status", "org.freedesktop.impl.portal.desktop.omarchy"], timeout=5)
    cases.append(
        CaseResult(
            "env.dbus_name",
            "D-Bus name owned",
            "env",
            "status succeeds",
            "pass" if r.returncode == 0 else "fail",
            (r.stdout or r.stderr or "")[:300],
        )
    )
    return cases


def check_settings_payload(res: CaseResult) -> CaseResult:
    if res.status != "pass":
        return res
    try:
        data = json.loads(res.detail.strip().splitlines()[-1])
        cs = data.get("color-scheme")
        ac = data.get("accent-color")
        if cs is None:
            res.status = "fail"
            res.detail = f"missing color-scheme: {data}"
        elif ac is None:
            res.status = "warn"
            res.detail = f"color-scheme={cs} but accent-color missing: {data}"
        else:
            res.detail = f"color-scheme={cs} accent-color={ac}"
    except Exception as e:
        res.status = "warn"
        res.detail = f"could not parse settings JSON: {e}; raw={res.detail[-200:]}"
    return res


def journal_snippet(since: str = "5 min ago") -> str:
    r = run(
        [
            "journalctl",
            "--user",
            "-u",
            "xdg-desktop-portal-omarchy.service",
            "--since",
            since,
            "--no-pager",
            "-n",
            "80",
        ],
        timeout=10,
    )
    return (r.stdout or "")[-4000:]


def main() -> int:
    results: list[CaseResult] = []
    print("=== env ===", flush=True)
    results.extend(check_env())

    print("=== unit ===", flush=True)
    results.append(unit_tests())

    print("=== api non-interactive ===", flush=True)
    # settings
    results.append(check_settings_payload(noninteractive("settings")))
    # notification add + remove
    results.append(noninteractive("notification"))
    results.append(noninteractive("notification-remove"))

    # email: may open client — use short timeout; cancel if window; treat spawn without crash as pass
    print("=== email ===", flush=True)
    t0 = time.time()
    attach = ROOT / "LICENSE"
    try:
        r = run(
            [
                sys.executable,
                str(PORTAL_CALL),
                "email",
                "--timeout",
                "8000",
                "--attach",
                str(attach),
            ],
            timeout=20,
        )
        out = (r.stdout or "") + (r.stderr or "")
        # Email often returns quickly with response 0 after spawning client
        ok = '"response": 0' in out or '"response":0' in out or r.returncode == 0
        results.append(
            CaseResult(
                "api.email",
                "Email Compose (with attachment path)",
                "api",
                "Response success or client spawned",
                "pass" if ok else "fail",
                out[-500:],
                int((time.time() - t0) * 1000),
            )
        )
    except subprocess.TimeoutExpired:
        results.append(
            CaseResult(
                "api.email",
                "Email Compose (with attachment path)",
                "api",
                "returns",
                "warn",
                "timed out waiting for Response (client may be open)",
                int((time.time() - t0) * 1000),
            )
        )

    # dynamic launcher prepare (UI) — cancel
    print("=== interactive dialogs ===", flush=True)
    deep = str(Path.home() / "Projects" / "omarchy" / "xdg-desktop-portal-omarchy")
    results.append(interactive_cancel("open", ["--folder", deep]))
    results.append(interactive_cancel("save", ["--folder", "/tmp", "--file", "/tmp/omarchy-portal-draft.txt"]))
    results.append(interactive_cancel("open-dir", ["--folder", str(Path.home())]))
    results.append(interactive_cancel("account"))
    results.append(interactive_cancel("access"))
    results.append(interactive_cancel("background"))
    results.append(interactive_cancel("app-chooser", ["--uri", "https://example.com", "--ask"]))
    results.append(interactive_cancel("screenshot"))
    results.append(interactive_cancel("dynamic-launcher"))

    # pick-color is interactive but different UI (hyprpicker) — short timeout warn
    print("=== pick-color (hyprpicker) ===", flush=True)
    t0 = time.time()
    proc = subprocess.Popen(
        [sys.executable, str(PORTAL_CALL), "pick-color", "--timeout", "5000"],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    time.sleep(1.5)
    # kill picker if present
    run(["pkill", "-x", "hyprpicker"], timeout=5)
    run(["wtype", "-k", "Escape"], timeout=5)
    try:
        out, _ = proc.communicate(timeout=10)
    except subprocess.TimeoutExpired:
        proc.kill()
        out, _ = proc.communicate()
    results.append(
        CaseResult(
            "ui.pick-color",
            "PickColor invokes picker",
            "interactive",
            "hyprpicker/portal path runs without daemon crash",
            "pass" if "HANDLE" in (out or "") or "response" in (out or "") else "warn",
            (out or "")[-400:],
            int((time.time() - t0) * 1000),
        )
    )

    # ScreenCast picker binary smoke (no full OBS flow)
    print("=== screencast picker smoke ===", flush=True)
    picker = Path.home() / ".local/bin/omarchy-share-picker"
    fallback = Path("/usr/bin/omarchy-share-picker")
    picker_path = picker if picker.exists() else fallback
    if picker_path.exists():
        env = os.environ.copy()
        env["XDPH_WINDOW_SHARING_LIST"] = ""
        # share picker without XDPH env may exit quickly
        try:
            r = run([str(picker_path)], timeout=3, env=env)
            results.append(
                CaseResult(
                    "ui.share_picker_bin",
                    "omarchy-share-picker executable",
                    "interactive",
                    "runs (may exit without XDPH env)",
                    "pass",
                    f"path={picker_path} rc={r.returncode} out={(r.stdout or r.stderr or '')[-200:]}",
                )
            )
        except subprocess.TimeoutExpired:
            run(["wtype", "-k", "Escape"], timeout=5)
            run(["pkill", "-f", "omarchy-share-picker"], timeout=5)
            results.append(
                CaseResult(
                    "ui.share_picker_bin",
                    "omarchy-share-picker executable",
                    "interactive",
                    "opens UI when invoked",
                    "pass",
                    f"path={picker_path} stayed running >3s (UI likely open); sent Escape",
                )
            )
    else:
        results.append(
            CaseResult(
                "ui.share_picker_bin",
                "omarchy-share-picker executable",
                "interactive",
                "binary exists",
                "fail",
                "not found in ~/.local/bin or /usr/bin",
            )
        )

    # Inhibit: call via gdbus CreateMonitor / Inhibit if possible — soft check journal
    print("=== inhibit journal ===", flush=True)
    j = journal_snippet("30 min ago")
    has_inhibit = "Inhibit.Inhibit" in j or "inhibit" in j.lower()
    results.append(
        CaseResult(
            "api.inhibit_seen",
            "Inhibit activity observed in journal (session apps)",
            "api",
            "optional — session may inhibit idle",
            "pass" if has_inhibit else "warn",
            "seen in journal" if has_inhibit else "no Inhibit lines in last 30m (may be idle)",
        )
    )

    # Summarize
    counts = {k: 0 for k in ("pass", "fail", "warn", "skip")}
    for c in results:
        counts[c.status] = counts.get(c.status, 0) + 1

    payload = {
        "generated_at": datetime.now().isoformat(timespec="seconds"),
        "repo": str(ROOT),
        "counts": counts,
        "cases": [asdict(c) for c in results],
        "journal_tail": journal_snippet("15 min ago")[-2000:],
    }
    REPORT_JSON.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")

    lines = [
        "# xdg-desktop-portal-omarchy test report",
        "",
        f"- Generated: `{payload['generated_at']}`",
        f"- Repo: `{ROOT}`",
        f"- Summary: **{counts['pass']} pass**, **{counts['fail']} fail**, **{counts['warn']} warn**, **{counts['skip']} skip**",
        "",
        "## Results",
        "",
        "| ID | Status | Name | Detail |",
        "|----|--------|------|--------|",
    ]
    for c in results:
        detail = c.detail.replace("|", "\\|").replace("\n", " ")[:180]
        lines.append(f"| `{c.id}` | **{c.status}** | {c.name} | {detail} |")

    lines += [
        "",
        "## Notes",
        "",
        "- Interactive cases expect the egui picker (`class=xdg-desktop-portal-omarchy`), then Esc.",
        "- Screenshots (if any): `/tmp/omarchy-portal-test-shots/`",
        "- JSON: `/tmp/omarchy-portal-test-report.json`",
        "",
        "## Failures / warnings to investigate",
        "",
    ]
    issues = [c for c in results if c.status in ("fail", "warn")]
    if not issues:
        lines.append("_None._")
    else:
        for c in issues:
            lines.append(f"### `{c.id}` — {c.status}")
            lines.append("")
            lines.append(f"**{c.name}**")
            lines.append("")
            lines.append("```")
            lines.append(c.detail[:1500])
            lines.append("```")
            lines.append("")

    REPORT_MD.write_text("\n".join(lines) + "\n")
    print(f"\nReport: {REPORT_MD}", flush=True)
    print(f"JSON:   {REPORT_JSON}", flush=True)
    print(json.dumps(counts), flush=True)
    return 1 if counts["fail"] else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
