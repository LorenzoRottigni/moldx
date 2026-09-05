FROM python:3.12-slim

WORKDIR /app
COPY pyproject.toml ./
COPY __init__.py ./
COPY main.py ./
COPY tests ./tests

RUN python -m pip install fastapi uvicorn

EXPOSE 8000
CMD ["python", "main.py"]
