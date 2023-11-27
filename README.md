# mini-jabber

A simple implementation of [XMPP](https://xmpp.org/) (Jabber with its old name) messaging protocol.

I'm following [RFC 6120](https://datatracker.ietf.org/doc/rfc6120/) directly without any external
resources to get used to the RFC syntax.

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

## Roadmap
- [X] XMPP handshake
- [ ] Switch to minidom crate for valid XML
- [ ] XMPP Messaging
- [ ] Friends list
- [ ] P2P connections with [XEP 1074](https://xmpp.org/extensions/xep-0174.html)
- [ ] Companion mobile and CLI apps
