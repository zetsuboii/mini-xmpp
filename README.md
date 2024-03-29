# mini-jabber

A simple implementation of [XMPP](https://xmpp.org/) (Jabber with its old name) messaging protocol.

I'm following [RFC 6120](https://datatracker.ietf.org/doc/rfc6120/) directly without any external
resources to get used to the RFC syntax.

## Demo
https://github.com/zetsuboii/mini-jabber/assets/12009698/b0302f1f-2556-4390-bc82-9cf401dd29c1

## Building
```bash
git clone https://github.com/zetsuboii/mini-jabber
cd mini-jabber
cargo build
```

## Running
```bash
cargo run --bin client
cargo run --bin server
```

## SQLX Cook Book
```bash
# Install sqlx-cli
cargo install sqlx-cli

echo "DATABASE_URL=sqlite:jabber.sqlite" > .env

# Create a database
sqlx database create

# Run migrations
sqlx migrate run
```

## Roadmap
- [X] XMPP handshake
- [X] Switch to minidom crate for valid XML (used quick-xml instead)
- [X] XMPP Messaging
- [X] Authentication
- [X] Resource binding
- [X] Message delivery
- [X] Friends list
- [ ] P2P connections with [XEP 1074](https://xmpp.org/extensions/xep-0174.html)
- [ ] Companion mobile and CLI apps
