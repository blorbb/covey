_:
    @just --list --justfile={{ justfile() }}

install-covey:
    cargo install --path ./covey-egui

install-plugins mode='release':
    just --justfile='../covey-plugins/justfile' install-all {{ mode }}

install-plugin plugin mode='release':
    just --justfile='../covey-plugins/justfile' install {{ plugin }} {{ mode }}
