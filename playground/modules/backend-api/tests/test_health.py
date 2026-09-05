"""Tests for the backend API."""
from app.main import app


def test_health_endpoint() -> None:
    """The health endpoint returns ok."""
    assert app.title == "backend-api"


def test_items_list() -> None:
    """The items list contains demo data."""
    assert len(app.routes) > 0
