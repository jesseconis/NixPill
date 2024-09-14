import pytest
import websockets
import asyncio

BASE_WS_URL = "ws://127.0.0.1:8000/ws"

# Test WebSocket connection and message exchange
@pytest.mark.asyncio
async def test_websocket():
    async with websockets.connect(BASE_WS_URL) as websocket:
        await websocket.send("Hello WebSocket")
        response = await websocket.recv()
        assert response == "Message received: Hello WebSocket"

