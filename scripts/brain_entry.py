"""Frozen-brain entry point.

Points tiktoken at the encoding cache bundled next to the executable before
litellm ever asks for an encoding — in a PyInstaller app tiktoken's plugin
discovery finds nothing ("Plugins found: []"), and without a user cache the
first token count dies with `Unknown encoding cl100k_base`. Warming the
encoding here also makes it a startup-time check: if the bundle is broken the
sidecar says so immediately instead of failing on the first ask.
"""

import asyncio
import os
import sys

_BUNDLED = os.path.join(os.path.dirname(sys.executable), "tiktoken-cache")
if os.path.isdir(_BUNDLED):
    os.environ["TIKTOKEN_CACHE_DIR"] = _BUNDLED

import tiktoken

tiktoken.get_encoding("cl100k_base")

from app.interfaces.sidecar.server import serve

if __name__ == "__main__":
    asyncio.run(serve())
