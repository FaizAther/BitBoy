#!/usr/bin/env python3
"""Unit tests for play_sessions (no softgbc-play binary required)."""

from __future__ import annotations

import subprocess
import unittest
from pathlib import Path
from unittest.mock import MagicMock

from play_sessions import (
    PlaySessionManager,
    new_session_id,
    parse_cookie_header,
    resolve_session_id,
    valid_session_id,
)


class FakeProc:
    def __init__(self) -> None:
        self._code = None
        self.stderr = None

    def poll(self):
        return self._code

    def terminate(self):
        self._code = 0

    def kill(self):
        self._code = -9

    def wait(self, timeout=None):
        return self._code


class SessionTests(unittest.TestCase):
    def test_valid_session_id(self):
        self.assertIsNone(valid_session_id(None))
        self.assertIsNone(valid_session_id("ab"))  # too short
        self.assertIsNone(valid_session_id("../evil"))
        self.assertEqual(valid_session_id("abc123"), "abc123")
        sid = new_session_id()
        self.assertTrue(valid_session_id(sid))

    def test_resolve_prefers_header(self):
        self.assertEqual(
            resolve_session_id(header="hdrAAAAA", query_s="qryBBBBB", cookie="cokCCCCC"),
            "hdrAAAAA",
        )
        self.assertEqual(
            resolve_session_id(header=None, query_s="qryBBBBB", cookie="cokCCCCC"),
            "qryBBBBB",
        )
        self.assertEqual(
            resolve_session_id(header=None, query_s=None, cookie="cokCCCCC"),
            "cokCCCCC",
        )

    def test_parse_cookie(self):
        self.assertEqual(
            parse_cookie_header("a=1; softgbc_play=sess1234; b=2"),
            "sess1234",
        )

    def test_same_sid_reuses_port(self):
        ports = iter([9101, 9102, 9103])
        procs: list[FakeProc] = []

        def popen(*_a, **_k):
            p = FakeProc()
            procs.append(p)
            return p

        mgr = PlaySessionManager(
            Path("/bin/true"),
            popen=popen,
            port_picker=lambda: next(ports),
            probe=lambda _p: True,
        )
        a, err = mgr.ensure("watchTEST01")
        self.assertIsNone(err)
        self.assertEqual(a.port, 9101)
        b, err = mgr.ensure("watchTEST01")
        self.assertIsNone(err)
        self.assertIs(a, b)
        self.assertEqual(len(procs), 1)

    def test_different_sids_get_different_ports(self):
        # Each spawn picks HTTP + GDB ports.
        ports = iter([9201, 9202, 9203, 9204])

        def popen(*_a, **_k):
            return FakeProc()

        mgr = PlaySessionManager(
            Path("/bin/true"),
            popen=popen,
            port_picker=lambda: next(ports),
            probe=lambda _p: True,
        )
        a, _ = mgr.ensure("userAAA1")
        b, _ = mgr.ensure("userBBB2")
        self.assertEqual(a.port, 9201)
        self.assertEqual(a.gdb_port, 9202)
        self.assertEqual(b.port, 9203)
        self.assertEqual(b.gdb_port, 9204)
        self.assertEqual(mgr.session_count(), 2)

    def test_list_live_includes_meta(self):
        def popen(*_a, **_k):
            return FakeProc()

        mgr = PlaySessionManager(
            Path("/bin/true"),
            popen=popen,
            port_picker=lambda: 9301,
            probe=lambda _p: True,
        )
        mgr.ensure("liveGame1")
        mgr.update_meta("liveGame1", title="TETRIS DX", frames=42, running=True)
        live = mgr.list_live()
        self.assertEqual(len(live), 1)
        self.assertEqual(live[0]["id"], "liveGame1")
        self.assertEqual(live[0]["title"], "TETRIS DX")
        self.assertEqual(live[0]["frames"], 42)
        self.assertTrue(live[0]["running"])
        self.assertIn("gdb_port", live[0])


if __name__ == "__main__":
    unittest.main()
