# SardineFish Push Service

A simple email notification push service for the comment system in my blog.

## Features
- Sending notification by email through SMTP with plain TCP or TLS connection
- User access controll
- Simple to configure & use
- Fast and reliable

[Service API Documentation](./docs/README.md)


### Build Requirement
- Rust 1.50-nightly (Not fully tested for other version)

### Runtime Requirement
- `libssl-dev` (For dynamic link with `openssl` crate)
- Mongodb

## Build

### Build Service
```shell
$ cargo +nightly build --release
```

### Build Manager Client
```shell
$ cd client
$ cargo +nightly build
```

### Build into Docker Image
```shell
$ docker build -t <tag-name> .
```

### Run Unit Tests
Make sure there is a mongodb instance running on localhost and listen to the default port.

Run the `init` test first to init database. The tests will insert some data into db named `sar-notify-test`. You can directly remove this db after running tests.
```shell
$ cargo test init
$ cargo test
```

## Start the Service
A mongodb is required for this service to work, start it before the service.

We recommend start with `--init` option to init a `Root` user, and revoke the `secret` immediately.
```shell
$ cargo run -- --db-addr=<address to mongodb> --db-name=<db name> --init
# Waiting for service init
$ cargo run -- --db-addr=<address to mongodb> --db-name=<db name>
```
The service will run on `localhost:5000` by default, you can change it by option `--listen=0.0.0.0:12345`

## Use the Manager Client
You can simply use the manager client to manage user access & service profiles.

i.e. Revoke the initial `secret` for `root` user
```shell
$ cd client
$ cargo run -- \
    --uid=root \
    --secret=secret_must_change \
    http://localhost:5000 \
    access revoke
```

Run `cargo run -- -- help` for help.