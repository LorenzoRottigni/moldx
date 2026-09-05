FROM python:3.12-slim

WORKDIR /app
COPY requirements.txt ./
RUN python -m pip install -r requirements.txt
COPY __init__.py ./
COPY main.py ./

CMD ["python", "main.py"]
