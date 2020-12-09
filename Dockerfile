FROM debian:buster-slim
WORKDIR /app
COPY ./target/release/sar-notify /app/sar-notify
EXPOSE 5000
RUN apt-get update
RUN apt-get install -y libssl-dev ca-certificates
CMD /app/sar-notify --listen 0.0.0.0:5000