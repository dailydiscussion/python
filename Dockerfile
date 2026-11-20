# Python Production Container for MovieBox API on Render / Cloud
FROM python:3.11-slim

WORKDIR /app

# Install dependencies
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# Copy backend application and permanent account configuration
COPY moviebox_api/ ./moviebox_api/
COPY moviebox_api_server.py ./
COPY saved_account.json ./

ENV PORT=8000 \
    PYTHONUNBUFFERED=1

EXPOSE 8000

CMD ["sh", "-c", "uvicorn moviebox_api_server:app --host 0.0.0.0 --port ${PORT:-8000}"]
