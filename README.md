[![Build Status](https://travis-ci.org/chouette-mail/chouette.svg?branch=master)](https://travis-ci.org/chouette-mail/chouette) [![Gitter chat](https://badges.gitter.im/gitterHQ/gitter.png)](https://gitter.im/chouette-mail/community)

# Chouette Mail

*The fastest and smallest mail client in the universe*

Chouette mail is a mail client consisting in a backend written in
[rust](https://www.rust-lang.org/) and a front end in
[elm](https://elm-lang.org/).

This is a work in progress, and for the moment, not many features.

### Building

The following sections explain how to build the client and the server. If you
have any problem building anything, feel free to open an
[issue](https://github.com/chouette-mail/chouette/issues/new).

#### Client

You'll need to intall [elm](https://guide.elm-lang.org/install.html) in order
to be able to build the client.

We recommend that you install elm with
[nvm](https://github.com/creationix/nvm#installation).

Once `elm` is installed, you should be able to run `make client-dev` in the
root of the repository to build the client.

#### Server


##### Installation

You'll need to install [rust-nightly](https://www.rust-lang.org/tools/install)
in order to be able to build the server. We recommend that you install rust
with rustup, so you can install rust nightly by running
`rustup toolchain add nightly`.

You have three tways to build the server with nightly:
  - you can simple run `make server-dev` at the root of the directory,
  - if you feel more confortable with using cargo, you can go to the server
    directory, run `rustup override set nightly`, and then you'll be able to
    `cargo build` freely,
  - or you can `cargo +nightly build` in the server directory.

##### Database setup

The server requires a postgresql database. The best way is to create a postgres
user and a database for it. On most operating systems, you need to use the
postgres user:

```
sudo su postgres
```

Once that's done, you can create a user like this:

```
createuser -W <username>
```

and then, create a database for the user:

```
createdb <dbname> -O <username>
```

You can then run

```
exit
```

to get back to your normal user.

##### Configuration

Once you've created the database, you can run the `chouette-setup` tool that
will prompt help to configure the server:

```
cargo run --bin chouette-setup
```

It will help you use the right settings, and check that everything is correct.

The last thing that needs to be done is it initialize the database. For this,
you will need `diesel_cli`. You can install it like so:

```
cargo install diesel_cli --no-default-features --features postgres
```

Once `diesel_cli` is installed, you will be able to initialize the database by
running:

```
diesel migration run
```

### Running

Once you've built and configured everything, you just go to the server
directory, and you run `cargo run` or `cargo +nightly run` depending on whether
you overrode the toolchain.

### Developping

You can install `elm-live` and run `make client-watch` at the root of the
repository. When a change will be made to the client, `elm-live` will rebuild
the client, and it will serve a static server on port 7000.  However, this is
not what you want, you need to start the rust server by going in the server
directory and running `cargo build` or `cargo +nightly build`, and test on
[localhost:8000](localhost:8000).

The rust server will always serve the latest version of the static files, so
you don't need to restart the rust server everytime if you only modified the
client.

