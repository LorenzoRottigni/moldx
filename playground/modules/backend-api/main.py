"""Entry point for the backend API service."""
from fastapi import FastAPI

app = FastAPI(title="backend-api", version="1.0.0")


@app.get("/health")
def health() -> dict:
    """Health check endpoint."""
    return {"status": "ok", "service": "backend-api"}


@app.get("/items")
def items() -> list[dict]:
    """Return a static list of demo items."""
    return [
        {"id": 1, "name": "alpha"},
        {"id": 2, "name": "beta"},
        {"id": 3, "name": "gamma"},
    ]


def run() -> None:
    """Run the uvicorn server."""
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)


if __name__ == "__main__":
    run()
