"""
Tests for the demo module.
"""

import pytest
from demo import hello_world


def test_hello_world_exists():
    """Test that the hello_world function exists."""
    assert callable(hello_world)


def test_hello_world_returns_correct_message():
    """Test that hello_world returns 'Hello World'."""
    result = hello_world()
    assert result == "Hello World"


def test_hello_world_prints(capsys):
    """Test that hello_world prints the correct message."""
    hello_world()
    captured = capsys.readouterr()
    assert "Hello World" in captured.out
