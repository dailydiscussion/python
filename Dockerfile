FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

ENV PORT=8000
ENV PROXY_URL=http://lpmrbkqk:cpege5k4lr07@45.38.107.97:6014

EXPOSE 8000

CMD ["sh", "-c", "uvicorn moviebox_api_server:app --host 0.0.0.0 --port ${PORT:-8000}"]
