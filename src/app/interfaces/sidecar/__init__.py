"""Sidecar interface: JSON-RPC over stdio for the desktop shell.

Implements the IPC contract from RFC-0002 §4.6 (envelope version 1).
The shell (Tauri/Rust) is the transport owner; this module is the brain's
server side: it reads newline-delimited JSON-RPC requests from stdin and
writes responses plus notifications to stdout.
"""

from app.interfaces.sidecar.server import serve

__all__ = ["serve"]
