FROM python:3.8

WORKDIR /usr/app

COPY ./server.py .

EXPOSE 9292

CMD ["python", "./server.py"]

