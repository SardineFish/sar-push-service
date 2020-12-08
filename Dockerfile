FROM debian:buster-slim
WORKDIR /app
COPY ./bin/sar-notify /app/sar-push
EXPOSE 5000
RUN apt-get update
RUN apt-get install -y libssl-dev
CMD /app/sar-push