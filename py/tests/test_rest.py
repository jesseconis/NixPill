import httpx
import pytest

BASE_URL = "http://127.0.0.1:8000"

# Test GET request
@pytest.mark.asyncio
async def test_get_item():
    async with httpx.AsyncClient() as client:
        response = await client.get(f"{BASE_URL}/items/1")
        assert response.status_code == 200
        data = response.json()
        assert data["item_id"] == 1
        assert data["name"] == "Item 1"

# Test POST request
@pytest.mark.asyncio
async def test_create_item():
    async with httpx.AsyncClient() as client:
        response = await client.post(f"{BASE_URL}/items/", json={"name": "TestItem", "description": "A test item"})
        assert response.status_code == 200
        data = response.json()
        assert data["message"] == "Item TestItem created"

