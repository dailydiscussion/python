FROM python:3.12-slim

RUN useradd -m -u 1000 user
USER user
ENV HOME=/home/user \
    PATH=/home/user/.local/bin:$PATH

WORKDIR $HOME/app

COPY --chown=user requirements.txt .
RUN pip install --no-cache-dir --upgrade -r requirements.txt

COPY --chown=user . .

ENV PORT=7860
ENV PROXY_URL=http://lpmrbkqk:cpege5k4lr07@45.38.107.97:6014

EXPOSE 7860

CMD ["sh", "-c", "uvicorn moviebox_api_server:app --host 0.0.0.0 --port ${PORT:-7860}"]
