run *ARGS:
    cargo run -- {{ ARGS }}

release LEVEL:
    cargo release -x {{ LEVEL }}
    git push
    git push --tags
