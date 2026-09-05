"""Python worker process for the MoldX playground.

Simulates a background worker that polls a queue and processes jobs.
"""

import json
import logging
import time
from dataclasses import dataclass, field

logging.basicConfig(level=logging.INFO, format="[%(levelname)s] %(message)s")
logger = logging.getLogger("worker")


@dataclass
class Queue:
    """An in-memory FIFO queue."""

    items: list = field(default_factory=list)

    def enqueue(self, item: dict) -> None:
        self.items.append(item)

    def dequeue(self) -> dict | None:
        return self.items.pop(0) if self.items else None


def process_job(job: dict) -> dict:
    """Simulate processing a single job."""
    logger.info("processing job %s", job.get("id"))
    time.sleep(0.1)
    job["status"] = "processed"
    return job


def main() -> None:
    """Run the worker for a fixed number of jobs."""
    queue = Queue()
    for i in range(3):
        queue.enqueue({"id": i + 1, "payload": f"job-{i + 1}"})

    logger.info("worker started with %d jobs", len(queue.items))
    results = []
    while job := queue.dequeue():
        results.append(process_job(job))

    logger.info("processed %d jobs", len(results))
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
