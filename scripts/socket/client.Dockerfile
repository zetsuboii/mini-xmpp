FROM python:3.8

WORKDIR /usr/app

COPY ./client.py .

EXPOSE 9292

CMD ["python", "./client.py"]

