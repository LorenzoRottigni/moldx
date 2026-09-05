"""Tests for the worker."""
from main import Queue, process_job


def test_queue_fifo() -> None:
    q = Queue()
    q.enqueue({"id": 1})
    q.enqueue({"id": 2})
    assert q.dequeue() == {"id": 1}
    assert q.dequeue() == {"id": 2}
    assert q.dequeue() is None


def test_process_job() -> None:
    job = {"id": 1}
    processed = process_job(job)
    assert processed["status"] == "processed"
