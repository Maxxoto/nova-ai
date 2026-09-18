"""``python -m app.interfaces.sidecar`` — run the brain's stdio JSON-RPC server."""

import asyncio

from app.interfaces.sidecar.server import serve


def main() -> None:
    asyncio.run(serve())


if __name__ == "__main__":
    main()
